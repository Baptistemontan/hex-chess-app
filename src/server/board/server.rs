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

use actix_web_lab::sse::{self, SendError};

const TIMEOUT_DURATION: Duration = Duration::from_secs(2);

pub struct Player {
    sender: sse::Sender,
    player_id: String,
}

async fn test_connection(sender: &sse::Sender) -> bool {
    let future = async { sender.send(sse::Event::Comment("ping".into())).await };

    actix_web::rt::time::timeout(TIMEOUT_DURATION, future)
        .await
        .is_ok_and(|r| r.is_ok())
}

impl Player {
    pub fn new(sender: sse::Sender, player_id: String) -> Self {
        Player { sender, player_id }
    }

    pub fn new_with_stream(player_id: String) -> (Self, sse::Sse<sse::ChannelStream>) {
        let (sender, stream) = sse::channel(10);
        (Self::new(sender, player_id), stream)
    }

    pub async fn send<E: Into<sse::Event>>(&self, event: E) -> Result<(), SendError> {
        self.sender.send(event).await
    }

    pub async fn is_connected(&self) -> bool {
        test_connection(&self.sender).await
    }

    pub fn has_id(&self, id: &str) -> bool {
        self.player_id == id
    }
}

pub struct Game {
    white_player: Player,
    black_player: Player,
    game_id: Uuid,
    board: Board,
    spectators: Vec<sse::Sender>,
}

impl Game {
    pub fn get_player_color(&self, player_id: String) -> Result<Color, GameError> {
        if self.white_player.player_id == player_id {
            Ok(Color::White)
        } else if self.black_player.player_id == player_id {
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
                Color::Black => &self.white_player.sender,
                Color::White => &self.black_player.sender,
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
        player1: Player,
        player2: Player,
        game_id: Uuid,
    ) -> Result<Self, Option<Player>> {
        let players = [player1, player2];

        // TODO: shuffle players

        let [player1, player2] = players;

        let player1_color = Color::White;
        let player2_color = Color::Black;

        let res = futures::join!(player1.is_connected(), player2.is_connected());

        match res {
            (true, true) => {}
            (true, false) => return Err(Some(player1)),
            (false, true) => return Err(Some(player2)),
            (false, false) => return Err(None),
        }

        let event_p1 = GameEvent::GameStart {
            game_id,
            player_color: player1_color,
        };
        let event_p2 = GameEvent::GameStart {
            game_id,
            player_color: player2_color,
        };

        let _ = futures::join!(player1.send(&event_p1), player2.send(&event_p2));

        Ok(Game {
            white_player: player1,
            black_player: player2,
            game_id,
            board: Board::new(),
            spectators: vec![],
        })
    }

    async fn remove_stale_specs(&mut self) {
        let futs = self.spectators.iter().map(test_connection);
        let res = futures::future::join_all(futs).await;

        let mut iter = res.into_iter();
        self.spectators.retain(|_| iter.next().unwrap());
    }

    pub async fn is_stale(&mut self) -> bool {
        // check if ended
        if self.board.is_end().is_some() {
            return true;
        }

        // remove stale specs
        self.remove_stale_specs().await;

        // check if all players are connected
        let res = futures::join!(
            self.white_player.is_connected(),
            self.black_player.is_connected()
        );

        match res {
            (true, false) => {
                let _ = self
                    .white_player
                    .send(&GameEvent::OpponentDisconnected)
                    .await;
                false
            }
            (false, true) => {
                let _ = self
                    .black_player
                    .send(&GameEvent::OpponentDisconnected)
                    .await;
                false
            }
            (true, true) => false,
            (false, false) => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    InvalidGameId(Uuid),
    InvalidPlayerId { game_id: Uuid },
    PlayerNotLoggedIn,
    AllPlayerDisconnected,
}

impl Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::PlayerNotLoggedIn => write!(f, "player must be logged in."),
            GameError::InvalidGameId(game_id) => write!(f, "game with id {} don't exist.", game_id),
            GameError::InvalidPlayerId { game_id } => {
                write!(f, "you are not a player of game {}", game_id)
            }
            GameError::AllPlayerDisconnected => f.write_str("All player disconnected."),
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
    custom_games: Mutex<HashMap<Uuid, Player>>,
    waiting_room: Mutex<Vec<Player>>,
}

impl Default for Games {
    fn default() -> Self {
        Self::new()
    }
}

pub enum GameState {
    Started(Arc<Mutex<Game>>),
    Waiting(Player),
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

    async fn find_waiting_game(&self) -> Option<Player> {
        self.waiting_room.lock().await.pop()
    }

    pub async fn start_new_game(
        &self,
        player1: Player,
        player2: Player,
        game_id: Uuid,
    ) -> Result<(), Option<Player>> {
        let game = Game::new(player1, player2, game_id).await?;
        println!("start game {}", game_id);
        let game = Arc::new(Mutex::new(game));
        self.games.lock().await.insert(game_id, game);
        Ok(())
    }

    pub async fn start_new_random_game(&self, player_id: String) -> sse::Sse<sse::ChannelStream> {
        let (player1, stream) = Player::new_with_stream(player_id);
        if let Some(player2) = self.find_waiting_game().await {
            if player2.player_id == player1.player_id {
                self.waiting_room.lock().await.push(player1);
            } else {
                let game_id = Uuid::new_v4();
                println!("start game {}", game_id);
                match Game::new(player1, player2, game_id).await {
                    Ok(game) => {
                        let game = Arc::new(Mutex::new(game));
                        self.games.lock().await.insert(game_id, game);
                    }
                    Err(Some(player)) => {
                        self.waiting_room.lock().await.push(player);
                    }
                    Err(None) => {}
                }
            }
        } else {
            println!("waiting for opponent");
            self.waiting_room.lock().await.push(player1);
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

        if let Some(player) = custom_games.remove(&id) {
            return Ok(GameState::Waiting(player));
        }

        Err(GameError::InvalidGameId(id))
    }

    pub async fn remove_stale_games(&self) {
        let mut games = self.games.lock().await;

        let mut to_remove = HashSet::new();

        for (game_id, game) in games.iter() {
            let mut game = game.lock().await;
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
        let (player, stream) = Player::new_with_stream(player_id);
        let _ = player.send(&GameEvent::CustomCreated { game_id }).await;
        let mut games = self.custom_games.lock().await;
        games.insert(game_id, player);
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
            Color::Black => game.black_player.sender = player,
            Color::White => game.white_player.sender = player,
        }

        stream
    }

    async fn join_started_game(
        game: Arc<Mutex<Game>>,
        player_id: &str,
    ) -> Result<sse::Sse<sse::ChannelStream>, GameError> {
        let mut game = game.lock().await;

        let is_black = game.black_player.has_id(player_id);
        let is_white = game.white_player.has_id(player_id);

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
        player1: Player,
        player2_id: String,
    ) -> Option<sse::Sse<sse::ChannelStream>> {
        let (player2, stream) = Player::new_with_stream(player2_id);
        let res = self.start_new_game(player1, player2, game_id).await;
        match res {
            Ok(()) => Some(stream),
            Err(Some(player)) => {
                let _ = player.send(&GameEvent::OpponentDisconnected).await;
                Some(stream)
            }
            Err(None) => None,
        }
    }

    pub async fn join_game(
        &self,
        game_id: Uuid,
        player_id: String,
    ) -> Result<sse::Sse<sse::ChannelStream>, GameError> {
        let game = self.get_all_game_with_id(game_id).await?;
        match game {
            GameState::Started(game) => Self::join_started_game(game, &player_id).await,
            GameState::Waiting(player1) => {
                if let Some(stream) = self.join_waiting_game(game_id, player1, player_id).await {
                    Ok(stream)
                } else {
                    Err(GameError::AllPlayerDisconnected)
                }
            }
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
