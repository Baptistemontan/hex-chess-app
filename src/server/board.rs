use hex_chess_core::{
    board::Board,
    hex_coord::HexVector,
    mov::{CanPromoteMove, IllegalMove},
    piece::PieceKind,
};
use leptos::*;

#[server(GetBoard, "/api")]
pub async fn get_board() -> Result<Board, ServerFnError> {
    Ok(Board::new())
}

#[server(PlayMove, "/api")]
pub async fn play_move(
    from: HexVector,
    to: HexVector,
    promote_to: Option<PieceKind>,
) -> Result<Result<Option<CanPromoteMove>, IllegalMove>, ServerFnError> {
    Ok(Board::new().play_move(from, to, promote_to))
}
