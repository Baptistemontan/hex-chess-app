use std::collections::HashSet;

use hex_chess_core::{
    board::Board as HexBoard,
    hex_coord::HexVector,
    mov::{CanPromoteMove, MaybePromoteMove},
    piece::{Color as PieceColor, Piece, PieceKind},
};
use leptos::*;

// use leptos_meta::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HexColor {
    Black,
    Grey,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Orientation {
    Normal,
    Reversed,
}

impl Orientation {
    pub fn reverse_assign(&mut self) {
        match self {
            Orientation::Normal => *self = Orientation::Reversed,
            Orientation::Reversed => *self = Orientation::Normal,
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
            colors: [HexColor::Black, HexColor::Grey, HexColor::White],
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

fn get_piece_url(piece: Piece) -> String {
    let color = match piece.color {
        PieceColor::Black => 'b',
        PieceColor::White => 'w',
    };
    let kind = match piece.kind {
        PieceKind::OriginalPawn | PieceKind::Pawn => 'p',
        PieceKind::Knight => 'n',
        PieceKind::Bishop => 'b',
        PieceKind::Rook => 'r',
        PieceKind::King => 'k',
        PieceKind::Queen => 'q',
    };
    format!("/assets/pieces/{color}{kind}.png")
}

#[allow(clippy::too_many_arguments)]
fn hexagon<F>(
    cx: Scope,
    vector: HexVector,
    color: HexColor,
    board: ReadSignal<HexBoard>,
    selected: ReadSignal<Option<HexVector>>,
    on_select: F,
    legal_moves: Memo<Option<HashSet<MaybePromoteMove>>>,
    orientation: ReadSignal<Orientation>,
    can_promote: ReadSignal<Option<CanPromoteMove>>,
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
    let hide = move || vector().mag() > 5;

    let piece = create_memo(cx, move |_| board.get().get_piece_at(vector()));

    let piece_image_url = create_memo(cx, move |_| piece.get().map(get_piece_url));

    let selected = move || selected.get().is_some_and(|pos| pos == vector());

    let is_move_dest = create_memo(cx, move |_| {
        legal_moves
            .get()
            .is_some_and(|moves| moves.iter().any(|mov| mov.to() == vector()))
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
                class=("hex-grid__content__black", move || color == HexColor::Black && !selected())
                class=("hex-grid__content__grey", move || color == HexColor::Grey && !selected())
                class=("hex-grid__content__white", move || color == HexColor::White && !selected())
                class=("hex-grid__content__selected", selected)
                class=("hex-grid__content__is_dest", move || is_move_dest.get() && !is_piece_and_dest())
                class=("hex-grid__content__is_piece_and_dest", is_piece_and_dest)
            >
                {move || piece_image_url.get().map(|url| {
                    // println!("{}", url);
                    view! { cx,
                        <img class="piece_image" src=url />
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
    let piece_url = get_piece_url(piece);
    view! { cx,
        <img on:click=on_click class="piece_image" src=piece_url />
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

#[component]
fn DrawHexBoard(
    cx: Scope,
    board: RwSignal<HexBoard>,
    orientation: ReadSignal<Orientation>,
) -> impl IntoView {
    let (selected, set_selected) = create_signal(cx, None);
    let (can_promote, set_can_promote) = create_signal(cx, None);

    let play_move = move |from: HexVector, to: HexVector, promote: Option<PieceKind>| {
        board.update(|board| match board.play_move(from, to, promote) {
            Ok(None) => set_selected.set(None),
            Ok(Some(promote)) => set_can_promote.set(Some(promote)),
            Err(err) => log!("invalid move: {:?}", err),
        })
    };

    let on_select = move |pos: HexVector, promote_to: Option<PieceKind>| {
        let (target_piece, turn) =
            board.with(|board| (board.get_piece_at(pos), board.get_player_turn()));
        set_can_promote.set(None);
        match (selected.get(), target_piece) {
            (_, Some(piece)) if piece.color == turn => set_selected(Some(pos)),
            (Some(selected), _) => play_move(selected, pos, promote_to),
            (None, _) => (),
        }
    };

    let legal_moves = create_memo(cx, move |_| board.get().get_legal_moves());

    let current_legal_moves = create_memo(cx, move |_| {
        selected
            .get()
            .and_then(|selected| legal_moves.get().get(&selected).cloned())
    });

    view! { cx,
        <ul class="hex-grid__list">
        {GridIterator::new().map(|(vector, color)| hexagon(cx, vector, color, board.read_only(), selected, on_select, current_legal_moves, orientation, can_promote)).collect_view(cx)}
        </ul>
    }
}

#[component]
pub fn Board(cx: Scope, board: RwSignal<HexBoard>) -> impl IntoView {
    let (orientation, set_orientation) = create_signal(cx, Orientation::Normal);

    let on_switch = move |_| set_orientation.update(Orientation::reverse_assign);

    view! { cx,
        <div>
            <DrawHexBoard board orientation/>
            <button on:click=on_switch>"switch side"</button>
        </div>
    }
}
