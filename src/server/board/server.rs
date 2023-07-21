use actix_web::*;

use futures::lock::Mutex;

use hex_chess_core::board::Board;
use hex_chess_core::{
    hex_coord::HexVector,
    mov::{CanPromoteMove, IllegalMove},
    piece::{Color, PieceKind},
};

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use super::GameEvent;

use actix_web_lab::sse;

pub struct Game {
    white_broadcast: sse::Sender,
    black_broadcast: sse::Sender,
    game_id: Uuid,
    white_id: Uuid,
    black_id: Uuid,
    board: Board,
}

impl Game {
    pub fn get_player_color(&self, player_id: Uuid) -> Result<Color, GameError> {
        if self.white_id == player_id {
            Ok(Color::White)
        } else if self.black_id == player_id {
            Ok(Color::Black)
        } else {
            Err(GameError::InvalidPlayerId {
                game_id: self.game_id,
                player_id,
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
            let _ = opponent.send(event).await;
        }
        Ok(res)
    }

    pub async fn play_move(
        &mut self,
        player_id: Uuid,
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
        game_id: Uuid,
    ) -> Result<Self, sse::Sender> {
        let (player1_id, player1_color) = (Uuid::new_v4(), Color::White);
        let (player2_id, player2_color) = (Uuid::new_v4(), Color::Black);
        let player1_ds = player1
            .send(GameEvent::start(game_id, player1_id, player1_color))
            .await
            .is_err();

        if player1_ds {
            return Err(player2);
        }
        let p2_ds = player2
            .send(GameEvent::start(game_id, player2_id, player2_color))
            .await
            .is_err();

        if p2_ds {
            let _ = player1.send(GameEvent::OpponentDisconnected).await;
            return Err(player1);
        }

        Ok(Game {
            white_broadcast: player1,
            black_broadcast: player2,
            game_id,
            white_id: player1_id,
            black_id: player2_id,
            board: Board::new(),
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
                    .send(GameEvent::OpponentDisconnected)
                    .await
            }
            (false, true) => {
                self.black_broadcast
                    .send(GameEvent::OpponentDisconnected)
                    .await
            }
            _ => Ok(()),
        };

        black_disconnected || white_disconnected
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameError {
    InvalidGameId(Uuid),
    InvalidPlayerId { game_id: Uuid, player_id: Uuid },
}

impl Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::InvalidGameId(game_id) => write!(f, "game with id {} don't exist.", game_id),
            GameError::InvalidPlayerId { game_id, player_id } => write!(
                f,
                "game with id {} don't have a player with id {}",
                game_id, player_id
            ),
        }
    }
}

impl std::error::Error for GameError {}

pub struct Games {
    games: Mutex<HashMap<Uuid, Arc<Mutex<Game>>>>,
    waiting_room: Mutex<Vec<sse::Sender>>,
}

impl Default for Games {
    fn default() -> Self {
        Self::new()
    }
}

impl Games {
    pub fn new() -> Self {
        actix_web::rt::spawn(remove_stale_games());

        Games {
            games: Mutex::new(HashMap::new()),
            waiting_room: Mutex::new(vec![]),
        }
    }

    async fn find_waiting_game(&self) -> Option<sse::Sender> {
        self.waiting_room.lock().await.pop()
    }

    pub async fn start_new_game(&self) -> sse::Sse<sse::ChannelStream> {
        let (sender, stream) = sse::channel(10);
        if let Some(opponent) = self.find_waiting_game().await {
            println!("start game");
            let game_id = Uuid::new_v4();
            match Game::new(sender, opponent, game_id).await {
                Ok(game) => {
                    let game = Arc::new(Mutex::new(game));
                    self.games.lock().await.insert(game_id, game);
                }
                Err(sender) => {
                    self.waiting_room.lock().await.push(sender);
                }
            }
        } else {
            println!("waiting for opponent");
            self.waiting_room.lock().await.push(sender);
        }
        stream
    }

    pub async fn get_game_with_id(&self, id: Uuid) -> Result<Arc<Mutex<Game>>, GameError> {
        let games = self.games.lock().await;
        games.get(&id).cloned().ok_or(GameError::InvalidGameId(id))
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

        println!("removed games {:?}", to_remove);

        games.retain(|id, _| !to_remove.contains(id));
    }
}

lazy_static::lazy_static! {
    pub static ref GAMES: Games = Games::new();
}

async fn remove_stale_games() {
    let mut interval = actix_web::rt::time::interval(Duration::from_secs(10));
    loop {
        interval.tick().await;
        GAMES.remove_stale_games().await
    }
}

#[get("/api/start_game")]
pub async fn start_game() -> sse::Sse<sse::ChannelStream> {
    GAMES.start_new_game().await
}
