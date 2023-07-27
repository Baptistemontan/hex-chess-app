use actix_web::*;

use futures::lock::Mutex;

use hex_chess_core::board::Board;
use hex_chess_core::{
    hex_coord::HexVector,
    mov::{CanPromoteMove, IllegalMove},
    piece::{Color, PieceKind},
};
use leptos::ServerFnError;

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::server::auth::user_id::MaybeUserId;

use super::GameEvent;

use actix_web_lab::sse;

pub struct Game {
    white_broadcast: sse::Sender,
    black_broadcast: sse::Sender,
    game_id: Uuid,
    white_id: String,
    black_id: String,
    board: Board,
    spectators: Vec<sse::Sender>,
}

impl Game {
    pub fn get_player_color(&self, player_id: String) -> Result<Color, GameError> {
        if self.white_id == player_id {
            Ok(Color::White)
        } else if self.black_id == player_id {
            Ok(Color::Black)
        } else {
            Err(GameError::InvalidPlayerId {
                game_id: self.game_id,
            })
        }
    }

    async fn play_move_inner(
        &mut self,
        color: Color,
        from: HexVector,
        to: HexVector,
        promote_to: Option<PieceKind>,
    ) -> Result<Option<CanPromoteMove>, IllegalMove> {
        let res = self.board.play_move(from, to, promote_to)?;
        if res.is_none() {
            let event = GameEvent::OpponentPlayedMove {
                from,
                to,
                promote_to,
            };
            let opponent = match color {
                Color::Black => &self.white_broadcast,
                Color::White => &self.black_broadcast,
            };

            let futs = self
                .spectators
                .iter()
                .chain(Some(opponent))
                .map(|sender| sender.send(&event));

            let _ = futures::future::join_all(futs).await; // don't care about result
        }
        Ok(res)
    }

    pub async fn play_move(
        &mut self,
        player_id: String,
        from: HexVector,
        to: HexVector,
        promote_to: Option<PieceKind>,
    ) -> Result<Result<Option<CanPromoteMove>, IllegalMove>, GameError> {
        let color = self.get_player_color(player_id)?;
        if color != self.board.get_player_turn() {
            return Ok(Err(IllegalMove::NotYourTurn));
        }
        Ok(self.play_move_inner(color, from, to, promote_to).await)
    }

    pub async fn new(
        player1: sse::Sender,
        player2: sse::Sender,
        player1_id: String,
        player2_id: String,
        game_id: Uuid,
    ) -> Result<Self, (sse::Sender, String)> {
        let player1_color = Color::White;
        let player2_color = Color::Black;
        let player1_ds = player1
            .send(&GameEvent::start(game_id, player1_color))
            .await
            .is_err();

        if player1_ds {
            return Err((player2, player2_id));
        }
        let p2_ds = player2
            .send(&GameEvent::start(game_id, player2_color))
            .await
            .is_err();

        if p2_ds {
            let _ = player1.send(&GameEvent::OpponentDisconnected).await;
            return Err((player1, player1_id));
        }

        Ok(Game {
            white_broadcast: player1,
            black_broadcast: player2,
            game_id,
            white_id: player1_id,
            black_id: player2_id,
            board: Board::new(),
            spectators: vec![],
        })
    }

    pub async fn is_stale(&self) -> bool {
        let black_disconnected = self
            .black_broadcast
            .send(sse::Event::Comment("ping".into()))
            .await
            .is_err();
        let white_disconnected = self
            .white_broadcast
            .send(sse::Event::Comment("ping".into()))
            .await
            .is_err();

        let _ = match (black_disconnected, white_disconnected) {
            (true, false) => {
                self.white_broadcast
                    .send(&GameEvent::OpponentDisconnected)
                    .await
            }
            (false, true) => {
                self.black_broadcast
                    .send(&GameEvent::OpponentDisconnected)
                    .await
            }
            _ => Ok(()),
        };

        black_disconnected && white_disconnected
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    InvalidGameId(Uuid),
    InvalidPlayerId { game_id: Uuid },
    PlayerNotLoggedIn,
}

impl Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::PlayerNotLoggedIn => write!(f, "player must be logged in."),
            GameError::InvalidGameId(game_id) => write!(f, "game with id {} don't exist.", game_id),
            GameError::InvalidPlayerId { game_id } => {
                write!(f, "youare not a player of game {}", game_id)
            }
        }
    }
}

impl actix_web::ResponseError for GameError {
    fn status_code(&self) -> http::StatusCode {
        http::StatusCode::BAD_REQUEST
    }
}

impl std::error::Error for GameError {}

pub struct Games {
    games: Mutex<HashMap<Uuid, Arc<Mutex<Game>>>>,
    custom_games: Mutex<HashMap<Uuid, (sse::Sender, String)>>,
    waiting_room: Mutex<Vec<(sse::Sender, String)>>,
}

impl Default for Games {
    fn default() -> Self {
        Self::new()
    }
}

pub enum GameState {
    Started(Arc<Mutex<Game>>),
    Waiting(sse::Sender, String),
}

impl Games {
    pub fn new() -> Self {
        actix_web::rt::spawn(remove_stale_games());

        Games {
            games: Mutex::new(HashMap::new()),
            custom_games: Mutex::new(HashMap::new()),
            waiting_room: Mutex::new(vec![]),
        }
    }

    async fn find_waiting_game(&self) -> Option<(sse::Sender, String)> {
        self.waiting_room.lock().await.pop()
    }

    pub async fn start_new_game(
        &self,
        player1: sse::Sender,
        player2: sse::Sender,
        player1_id: String,
        player2_id: String,
        game_id: Uuid,
    ) -> Result<(), (sse::Sender, String)> {
        match Game::new(player1, player2, player1_id, player2_id, game_id).await {
            Ok(game) => {
                println!("start game {}", game_id);
                let game = Arc::new(Mutex::new(game));
                self.games.lock().await.insert(game_id, game);
                Ok(())
            }
            Err(sender) => Err(sender),
        }
    }

    pub async fn start_new_random_game(&self, player_id: String) -> sse::Sse<sse::ChannelStream> {
        let (sender, stream) = sse::channel(10);
        if let Some((opponent, opponent_id)) = self.find_waiting_game().await {
            let game_id = Uuid::new_v4();
            println!("start game {}", game_id);
            match Game::new(sender, opponent, player_id, opponent_id, game_id).await {
                Ok(game) => {
                    let game = Arc::new(Mutex::new(game));
                    self.games.lock().await.insert(game_id, game);
                }
                Err((sender, player_id)) => {
                    self.waiting_room.lock().await.push((sender, player_id));
                }
            }
        } else {
            println!("waiting for opponent");
            self.waiting_room.lock().await.push((sender, player_id));
        }
        stream
    }

    pub async fn get_game_with_id(&self, id: Uuid) -> Result<Arc<Mutex<Game>>, GameError> {
        let games = self.games.lock().await;
        if let Some(game) = games.get(&id) {
            return Ok(game.clone());
        }
        Err(GameError::InvalidGameId(id))
    }

    pub async fn get_all_game_with_id(&self, id: Uuid) -> Result<GameState, GameError> {
        if let Ok(game) = self.get_game_with_id(id).await {
            return Ok(GameState::Started(game));
        }
        let mut custom_games = self.custom_games.lock().await;

        if let Some((player, player_id)) = custom_games.remove(&id) {
            return Ok(GameState::Waiting(player, player_id));
        }

        Err(GameError::InvalidGameId(id))
    }

    pub async fn remove_stale_games(&self) {
        let mut games = self.games.lock().await;

        let mut to_remove = HashSet::new();

        for (game_id, game) in games.iter() {
            let game = game.lock().await;
            if game.is_stale().await {
                to_remove.insert(*game_id);
            }
        }

        if !to_remove.is_empty() {
            println!("removed games {:?}", to_remove);
        }

        games.retain(|id, _| !to_remove.contains(id));
    }

    pub async fn create_custom_game(&self, player_id: String) -> sse::Sse<sse::ChannelStream> {
        let game_id = Uuid::new_v4();
        let (player, stream) = sse::channel(10);
        let _ = player.send(&GameEvent::CustomCreated { game_id }).await;
        let mut games = self.custom_games.lock().await;
        games.insert(game_id, (player, player_id));
        println!("created custom game {}", game_id);
        stream
    }

    async fn join_started_game_inner(
        game: &mut Game,
        player_color: Color,
    ) -> sse::Sse<sse::ChannelStream> {
        let (player, stream) = sse::channel(10);

        let event = GameEvent::RejoinedGame {
            game_id: game.game_id,
            player_color,
            board: game.board.clone(),
        };

        let _ = player.send(&event).await;

        match player_color {
            Color::Black => game.black_broadcast = player,
            Color::White => game.white_broadcast = player,
        }

        stream
    }

    async fn join_started_game(
        game: Arc<Mutex<Game>>,
        player_id: String,
    ) -> Result<sse::Sse<sse::ChannelStream>, GameError> {
        let mut game = game.lock().await;

        let is_black = game.black_id == player_id;
        let is_white = game.white_id == player_id;

        match (is_black, is_white) {
            (true, false) => Ok(Self::join_started_game_inner(&mut game, Color::Black).await),
            (false, true) => Ok(Self::join_started_game_inner(&mut game, Color::White).await),
            _ => Err(GameError::InvalidPlayerId {
                game_id: game.game_id,
            }),
        }
    }

    async fn join_waiting_game(
        &self,
        game_id: Uuid,
        player1: sse::Sender,
        player1_id: String,
        player2_id: String,
    ) -> sse::Sse<sse::ChannelStream> {
        let (player2, stream) = sse::channel(10);
        let res = self
            .start_new_game(player1, player2, player1_id, player2_id, game_id)
            .await;
        if let Err((sender, _)) = res {
            let _ = sender.send(&GameEvent::OpponentDisconnected).await;
        }
        stream
    }

    pub async fn join_game(
        &self,
        game_id: Uuid,
        player_id: String,
    ) -> Result<sse::Sse<sse::ChannelStream>, GameError> {
        let game = self.get_all_game_with_id(game_id).await?;
        match game {
            GameState::Started(game) => Self::join_started_game(game, player_id).await,
            GameState::Waiting(player1, player1_id) => Ok(self
                .join_waiting_game(game_id, player1, player1_id, player_id)
                .await),
        }
    }
}

lazy_static::lazy_static! {
    pub static ref GAMES: Games = Games::new();
}

async fn remove_stale_games() {
    let mut interval = actix_web::rt::time::interval(Duration::from_secs(10));
    loop {
        interval.tick().await;
        GAMES.remove_stale_games().await;
    }
}

pub fn check_id(id: MaybeUserId) -> Result<String, GameError> {
    id.0.ok_or(GameError::PlayerNotLoggedIn)
}

pub async fn get_player_id(cx: leptos::Scope) -> Result<String, ServerFnError> {
    use leptos_actix::extract;
    async fn inner(id: MaybeUserId) -> Result<String, GameError> {
        check_id(id)
    }
    let result = extract(cx, inner).await?;
    result.map_err(|_| ServerFnError::ServerError("not logged in.".into()))
}

#[get("new_random_game")]
async fn random_game(player_id: MaybeUserId) -> Result<sse::Sse<sse::ChannelStream>, GameError> {
    let id = check_id(player_id)?;
    Ok(GAMES.start_new_random_game(id).await)
}

#[get("new_custom_game")]
async fn custom_game(player_id: MaybeUserId) -> Result<sse::Sse<sse::ChannelStream>, GameError> {
    let id = check_id(player_id)?;
    Ok(GAMES.create_custom_game(id).await)
}

#[get("join_game/{game_id}")]
async fn join_game(
    game_id: web::Path<Uuid>,
    player_id: MaybeUserId,
) -> Result<sse::Sse<sse::ChannelStream>, GameError> {
    let id = check_id(player_id)?;
    let game_id = game_id.into_inner();
    println!("try joining game {}", game_id);
    GAMES.join_game(game_id, id).await
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(random_game)
        .service(custom_game)
        .service(join_game);
}
