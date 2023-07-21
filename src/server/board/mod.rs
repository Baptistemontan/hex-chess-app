use hex_chess_core::{
    hex_coord::HexVector,
    mov::{CanPromoteMove, IllegalMove},
    piece::{Color, PieceKind},
};
use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ssr")]
pub mod server;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum GameEvent {
    WaitingForOpponent,
    CustomCreated {
        game_id: Uuid,
    },
    GameStart {
        game_id: Uuid,
        player_id: Uuid,
        player_color: Color,
    },
    OpponentPlayedMove {
        from: HexVector,
        to: HexVector,
        promote_to: Option<PieceKind>,
    },
    OpponentDisconnected,
}

impl GameEvent {
    pub fn start(game_id: Uuid, player_id: Uuid, player_color: Color) -> Self {
        GameEvent::GameStart {
            game_id,
            player_id,
            player_color,
        }
    }
}

#[cfg(feature = "ssr")]
impl From<GameEvent> for actix_web_lab::sse::Event {
    fn from(val: GameEvent) -> Self {
        actix_web_lab::sse::Data::new_json(val).unwrap().into()
    }
}

#[server(PlayMove, "/api")]
pub async fn play_move(
    game_id: Uuid,
    player_id: Uuid,
    from: HexVector,
    to: HexVector,
    promote_to: Option<PieceKind>,
) -> Result<Result<Option<CanPromoteMove>, IllegalMove>, ServerFnError> {
    let game = server::GAMES.get_game_with_id(game_id).await?;

    let mut game = game.lock().await;
    game.play_move(player_id, from, to, promote_to)
        .await
        .map_err(Into::into)
}
