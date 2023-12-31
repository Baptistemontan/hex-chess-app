// use gloo_net::eventsource::EventSourceError;
use hex_chess_core::{
    board::Board,
    hex_coord::HexVector,
    mov::{CanPromoteMove, MaybePromoteMove},
    piece::{Color as PieceColor, Piece, PieceKind},
};
use leptos::*;
use std::collections::HashSet;

use crate::{
    pages::play::GameEventStream,
    server::board::{GameEvent, PlayMove},
};

// use leptos_meta::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HexColor {
    Black,
    Grey,
    White,
    Selected,
    LastMove,
}

impl HexColor {
    pub const COLORS: [Self; 3] = [HexColor::Black, HexColor::Grey, HexColor::White];

    fn reverse(self) -> Self {
        match self {
            HexColor::Black => HexColor::White,
            HexColor::White => HexColor::Black,
            _ => self,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Orientation {
    Normal,
    Reversed,
}

impl Orientation {
    pub fn reverse_assign(&mut self) {
        *self = match self {
            Orientation::Normal => Orientation::Reversed,
            Orientation::Reversed => Orientation::Normal,
        }
    }
}

struct GridIterator {
    q: isize,
    r: isize,
    line: isize,
    colors: [HexColor; 3],
}

impl GridIterator {
    pub fn new() -> Self {
        GridIterator {
            q: -5,
            r: -2,
            line: 0,
            colors: HexColor::COLORS,
        }
    }
}

impl Iterator for GridIterator {
    type Item = (HexVector, HexColor);

    fn next(&mut self) -> Option<Self::Item> {
        if self.line >= 11 {
            return None;
        }
        let vector = HexVector::new_axial(self.q, self.r);
        let is_even = self.q.unsigned_abs() % 2;
        let color = self.colors[is_even];

        self.q += 1;

        if self.q == 6 {
            self.colors.rotate_right(1);
            self.q = -5;
            self.line += 1;
            self.r = self.line - 2;
        } else if is_even == 1 {
            self.r -= 1;
        }

        Some((vector, color))
    }
}

fn get_piece_url_and_alt(piece: Piece) -> (String, String) {
    let (color_url, color) = match piece.color {
        PieceColor::Black => ('b', "black"),
        PieceColor::White => ('w', "white"),
    };
    let (kind_url, kind) = match piece.kind {
        PieceKind::OriginalPawn | PieceKind::Pawn => ('p', "pawn"),
        PieceKind::Knight => ('n', "knight"),
        PieceKind::Bishop => ('b', "bishop"),
        PieceKind::Rook => ('r', "rook"),
        PieceKind::King => ('k', "king"),
        PieceKind::Queen => ('q', "queen"),
    };
    let url = format!("/assets/pieces/{}{}.png", color_url, kind_url);
    let alt = format!("{} {}", color, kind);
    (url, alt)
}

#[allow(clippy::too_many_arguments)]
fn hexagon<F>(
    cx: Scope,
    vector: HexVector,
    color: HexColor,
    board: ReadSignal<Board>,
    selected: ReadSignal<Option<HexVector>>,
    on_select: F,
    legal_moves: Memo<Option<HashSet<HexVector>>>,
    orientation: ReadSignal<Orientation>,
    can_promote: ReadSignal<Option<CanPromoteMove>>,
    last_move: ReadSignal<Option<(HexVector, HexVector)>>,
) -> impl IntoView
where
    F: Fn(HexVector, Option<PieceKind>) + Copy + 'static,
{
    let vector = move || {
        if orientation.get() == Orientation::Normal {
            vector
        } else {
            -vector
        }
    };

    let color = move || {
        let vector = vector();
        let selected = selected.get();
        let orientation = orientation.get();
        let last_move = last_move.get();
        if selected.is_some_and(|pos| pos == vector) {
            HexColor::Selected
        } else if last_move.is_some_and(|(from, to)| from == vector || to == vector) {
            HexColor::LastMove
        } else if orientation == Orientation::Normal {
            color
        } else {
            color.reverse()
        }
    };

    let hide = move || vector().mag() > 5;

    let piece = create_memo(cx, move |_| board.get().get_piece_at(vector()));

    let piece_image_url = create_memo(cx, move |_| piece.get().map(get_piece_url_and_alt));

    let is_move_dest = create_memo(cx, move |_| {
        legal_moves.with(|moves| {
            moves
                .as_ref()
                .is_some_and(|moves| moves.contains(&vector()))
        })
    });

    let is_piece_and_dest = move || piece.get().is_some() && is_move_dest.get();

    let on_click = move |_| {
        on_select(vector(), None);
    };

    let is_promote = move || {
        can_promote.get().and_then(|promote_move| {
            if promote_move.to != vector() {
                None
            } else {
                Some(promote_move)
            }
        })
    };

    view! { cx,
        <li
            class="hex-grid__item"
            class=("hex-grid__item__hide", hide)
            class=("hex-grid__item__is_piece", move || piece.get().is_some())
            on:click=on_click
        >
            <div
                class="hex-grid__content"
                class=("hex-grid__content__black", move || color() == HexColor::Black)
                class=("hex-grid__content__grey", move || color() == HexColor::Grey)
                class=("hex-grid__content__white", move || color() == HexColor::White)
                class=("hex-grid__content__selected", move || color() == HexColor::Selected)
                class=("hex-grid__content__last_move", move || color() == HexColor::LastMove)
                class=("hex-grid__content__is_dest", move || is_move_dest.get() && !is_piece_and_dest())
                class=("hex-grid__content__is_piece_and_dest", is_piece_and_dest)
            >
                {move || piece_image_url.get().map(|(url, alt)| {
                    let alt = format!("icon for a {}", alt);
                    view! { cx,
                        <img class="piece_image" src=url alt=alt />
                    }
                })}
            </div>
            {move || is_promote().map(move |promote_move| {
                let promote_fn = move |piece_kind| on_select(vector(), Some(piece_kind));
                promote(cx, orientation, promote_move.color, promote_fn)
            })}
        </li>
    }
}

fn promote_to<F>(cx: Scope, piece: Piece, promote_fn: F) -> impl IntoView
where
    F: Fn(PieceKind) + Copy + 'static,
{
    let on_click = move |_| promote_fn(piece.kind);
    let (piece_url, alt) = get_piece_url_and_alt(piece);
    let title = format!("Promote to a {}", alt);
    let alt = format!("button icon to promote to a {}", alt);
    view! { cx,
        <img on:click=on_click class="piece_image" src=piece_url alt=alt title=title />
    }
}

fn promote<F>(
    cx: Scope,
    orientation: ReadSignal<Orientation>,
    piece_color: PieceColor,
    promote_fn: F,
) -> impl IntoView
where
    F: Fn(PieceKind) + Copy + 'static,
{
    let is_reversed =
        move || match (orientation.get(), piece_color) {
            (Orientation::Normal, PieceColor::White)
            | (Orientation::Reversed, PieceColor::Black) => false,
            (Orientation::Normal, PieceColor::Black)
            | (Orientation::Reversed, PieceColor::White) => true,
        };
    view! { cx,
        <div class="promote" class=("promote__reversed", is_reversed)>
        {
            [PieceKind::Queen, PieceKind::Rook, PieceKind::Knight, PieceKind::Bishop]
            .into_iter()
            .map(|kind| Piece::new(piece_color, kind))
            .map(|piece| promote_to(cx, piece, promote_fn))
            .collect_view(cx)
        }
        </div>
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_hex_board<OS>(
    cx: Scope,
    board: ReadSignal<Board>,
    orientation: ReadSignal<Orientation>,
    selected: ReadSignal<Option<HexVector>>,
    can_promote: ReadSignal<Option<CanPromoteMove>>,
    last_move: ReadSignal<Option<(HexVector, HexVector)>>,
    color: impl Fn() -> PieceColor + 'static,
    on_select: OS,
) -> impl IntoView
where
    OS: Fn(HexVector, Option<PieceKind>) + Copy + 'static,
{
    let legal_moves = create_memo(cx, move |_| board.get().get_legal_moves_for(color()));

    let current_legal_moves = create_memo(cx, move |_| {
        legal_moves.track();
        selected.get().and_then(|selected| {
            legal_moves.with(|map| {
                Some(
                    map.get(&selected)?
                        .iter()
                        .copied()
                        .map(MaybePromoteMove::to)
                        .collect(),
                )
            })
        })
    });

    view! { cx,
        <ul class="hex-grid__list">
        {move || GridIterator::new().map(|(vector, color)| hexagon(cx, vector, color, board, selected, on_select, current_legal_moves, orientation, can_promote, last_move)).collect_view(cx)}
        </ul>
    }
}

#[allow(clippy::too_many_arguments)]
fn orientation_manager(
    cx: Scope,
    board: ReadSignal<Board>,
    selected: ReadSignal<Option<HexVector>>,
    player_color: impl Fn() -> PieceColor + Copy + 'static,
    can_promote: ReadSignal<Option<CanPromoteMove>>,
    last_move: ReadSignal<Option<(HexVector, HexVector)>>,
    is_solo: bool,
    on_select: impl Fn(HexVector, Option<PieceKind>) + Copy + 'static,
    back_one_turn: impl Fn(ev::MouseEvent) + 'static,
    advance_history: impl Fn(ev::MouseEvent) + 'static,
    unwind_history: impl Fn(ev::MouseEvent) + 'static,
) -> impl IntoView {
    let (orientation, set_orientation) = create_signal(cx, Orientation::Normal);
    create_effect(cx, move |_| {
        let color = player_color();
        if !is_solo {
            let orientation = match color {
                PieceColor::Black => Orientation::Reversed,
                PieceColor::White => Orientation::Normal,
            };
            set_orientation.set(orientation);
        }
    });
    let on_switch = move |_| set_orientation.update(Orientation::reverse_assign);

    view! { cx,
        <div>
            {draw_hex_board(cx, board, orientation, selected, can_promote, last_move, player_color, on_select)}
            <div class="under_board">
                <div class="history_movement">
                    <img on:click=back_one_turn class="board_button" src="/assets/icons/backward.svg" alt="button icon for going back one turn" title="Go back one turn"/>
                    <img on:click=advance_history class="board_button" src="/assets/icons/forward.svg" alt="button icon for advancing one turn in the history" title="Advance by one turn"/>
                    <img on:click=unwind_history class="board_button" src="/assets/icons/forward_double.svg" alt="button icon for going back to last turn in history" title="Go to last turn"/>
                </div>
                <img on:click=on_switch class="board_button" src="/assets/icons/switch_side.svg" alt="button icon for switching the orientation of the board" title="Switch board orientation"/>
            </div>
        </div>
    }
}

fn get_board_buttons_func(
    set_selected: WriteSignal<Option<HexVector>>,
    set_board: WriteSignal<Board>,
    set_last_move: WriteSignal<Option<(HexVector, HexVector)>>,
) -> (
    impl Fn(ev::MouseEvent) + 'static,
    impl Fn(ev::MouseEvent) + 'static,
    impl Fn(ev::MouseEvent) + 'static,
) {
    fn inner(
        func: impl Fn(&mut Board),
        set_selected: WriteSignal<Option<HexVector>>,
        set_board: WriteSignal<Board>,
        set_last_move: WriteSignal<Option<(HexVector, HexVector)>>,
    ) {
        set_selected.set(None);
        set_board.update(|board| {
            func(board);
            let last_move = board.get_last_played_move().map(|mov| (mov.from, mov.to));
            set_last_move.set(last_move);
        })
    }

    let back_one_turn =
        move |_| inner(Board::back_one_turn, set_selected, set_board, set_last_move);
    let advance_history = move |_| {
        inner(
            Board::advance_history,
            set_selected,
            set_board,
            set_last_move,
        )
    };
    let unwind_history = move |_| {
        inner(
            Board::unwind_history,
            set_selected,
            set_board,
            set_last_move,
        )
    };

    (back_one_turn, advance_history, unwind_history)
}

#[component]
pub fn MultiBoard(cx: Scope, events: GameEventStream) -> impl IntoView {
    let (selected, set_selected) = create_signal(cx, None);
    let (dest, set_dest) = create_signal(cx, None);
    let (can_promote, set_can_promote) = create_signal(cx, None);
    let (board, set_board) = create_signal(cx, Board::new());
    let (player_infos, set_player_infos) = create_signal(cx, (PieceColor::White, None));

    let (last_move, set_last_move) = create_signal(cx, None);

    let is_end = create_memo(cx, move |_| board.get().is_end());

    create_effect(cx, move |_| {
        if is_end.get().is_some() {
            set_selected.set(None);
            set_dest.set(None);
        }
    });

    events.listen(cx, move |event| match event {
        GameEvent::GameStart {
            game_id,
            player_color,
        } => {
            set_player_infos.set((player_color, Some(game_id)));
        }
        GameEvent::OpponentPlayedMove {
            from,
            to,
            promote_to,
        } => {
            let is_in_history = board.with_untracked(|board| !board.get_next_moves().is_empty());
            set_board.update(|board| {
                if is_in_history {
                    board.unwind_history();
                }
                board.play_move(from, to, promote_to).unwrap();
            });
            set_last_move.set(Some((from, to)));
            if is_in_history || selected.get_untracked().is_some_and(|pos| pos == to) {
                set_selected.set(None);
            }
        }
        GameEvent::RejoinedGame {
            game_id,
            player_color,
            board,
        } => {
            if let Some(board) = board {
                if let Some(last_move) = board.get_last_played_move() {
                    set_last_move.set(Some((last_move.from, last_move.to)));
                }
                set_board.set(board);
            }
            set_player_infos.set((player_color, Some(game_id)))
        }
        _ => (),
    });

    let player_color = move || player_infos.get().0;

    let play_server_move = create_server_action::<PlayMove>(cx);

    let on_select = move |pos: HexVector, promote_to: Option<PieceKind>| {
        let (color, ids) = player_infos.get();

        let selected = selected.get();
        let (target_piece, is_turn) =
            board.with(|board| (board.get_piece_at(pos), board.get_player_turn() == color));
        if is_end.get().is_some() || ids.is_none() {
            return;
        }
        match (selected, target_piece) {
            (_, Some(piece)) if piece.color == color => {
                set_selected.set(Some(pos));
                set_dest.set(None);
            }
            (Some(_), _) if is_turn => {
                set_dest.set(Some((pos, promote_to)));
            }
            _ => (),
        }
    };

    let (back_one_turn, advance_history, unwind_history) =
        get_board_buttons_func(set_selected, set_board, set_last_move);

    create_effect(cx, move |_| {
        let (color, ids) = player_infos.get();
        let from = selected.get();
        let to = dest.get();
        if board
            .with(|board| board.get_player_turn() != color || !board.get_next_moves().is_empty())
        {
            return;
        }
        if let (Some(game_id), Some(from), Some((to, promote_to))) = (ids, from, to) {
            let mut move_res = Ok(None);
            set_board.update(|board| {
                move_res = board.play_move(from, to, promote_to);
            });
            match move_res {
                Ok(Some(promote_move)) => set_can_promote.set(Some(promote_move)),
                Ok(None) => {
                    set_can_promote.set(None);
                    set_dest.set(None);
                    set_selected.set(None);
                    set_last_move.set(Some((from, to)));
                    play_server_move.dispatch(PlayMove {
                        game_id,
                        from,
                        to,
                        promote_to,
                    });
                }
                _ => (),
            }
        }
    });

    view! { cx,

        <div>
            {orientation_manager(cx, board, selected, player_color, can_promote, last_move, false, on_select, back_one_turn, advance_history, unwind_history)}
            {move || is_end.get().map(|end| {
                match end {
                    hex_chess_core::board::GameEnd::Win(color) => {
                        view! { cx,
                            <p>{format!("{:?} wins!", color)}</p>
                        }
                    },
                    hex_chess_core::board::GameEnd::Draw => {
                        view! { cx,
                            <p>"Draw"</p>
                        }
                    },
                    hex_chess_core::board::GameEnd::Stalemate { winner } => {
                        view! { cx,
                            <p>{format!("Stalemate, {:?} wins 3/4 of the points", winner)}</p>
                        }
                    },
                }
            })}
        </div>

    }
}

#[component]
pub fn SoloBoard(cx: Scope) -> impl IntoView {
    let (board, set_board) = create_signal(cx, Board::new());
    let (selected, set_selected) = create_signal(cx, None);
    let (can_promote, set_can_promote) = create_signal(cx, None);
    let (dest, set_dest) = create_signal(cx, None);
    let (last_move, set_last_move) = create_signal(cx, None);

    let is_end = create_memo(cx, move |_| board.get().is_end());

    create_effect(cx, move |_| {
        if is_end.get().is_some() {
            set_selected.set(None);
            set_dest.set(None);
        }
    });

    let on_select = move |pos: HexVector, promote_to: Option<PieceKind>| {
        let (target_piece, color) =
            board.with(|board| (board.get_piece_at(pos), board.get_player_turn()));
        let selected = selected.get();
        if is_end.get().is_some() {
            return;
        }
        match (selected, target_piece) {
            (_, Some(piece)) if piece.color == color => {
                set_selected.set(Some(pos));
                set_dest.set(None);
            }
            (Some(_), _) => {
                set_dest.set(Some((pos, promote_to)));
            }
            (None, _) => (),
        }
    };

    create_effect(cx, move |_| {
        if let (Some(from), Some((to, promote_to))) = (selected.get(), dest.get()) {
            let mut res = Ok(None);
            set_board.update(|board| res = board.play_move(from, to, promote_to));
            if let Ok(Some(promote_move)) = res {
                set_can_promote.set(Some(promote_move))
            } else {
                if res.is_ok() {
                    set_last_move.set(Some((from, to)));
                }
                set_selected.set(None);
                set_dest.set(None);
                set_can_promote.set(None);
            }
        }
    });

    let color = move || board.with(Board::get_player_turn);

    let (back_one_turn, advance_history, unwind_history) =
        get_board_buttons_func(set_selected, set_board, set_last_move);

    view! { cx,
        <div>
            {orientation_manager(
                cx,
                board,
                selected,
                color,
                can_promote,
                last_move,
                true,
                on_select,
                back_one_turn,
                advance_history,
                unwind_history
            )}
            {move || is_end.get().map(|end| {
                match end {
                    hex_chess_core::board::GameEnd::Win(color) => {
                        view! { cx,
                            <p>{format!("{:?} wins!", color)}</p>
                        }
                    },
                    hex_chess_core::board::GameEnd::Draw => {
                        view! { cx,
                            <p>"Draw"</p>
                        }
                    },
                    hex_chess_core::board::GameEnd::Stalemate { winner } => {
                        view! { cx,
                            <p>{format!("Stalemate, {:?} wins 3/4 of the points", winner)}</p>
                        }
                    },
                }
            })}
        </div>

    }
}
