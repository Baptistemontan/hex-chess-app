use std::collections::HashSet;

use hex_chess_core::{board::Board, hex_coord::HexVector, mov::MaybePromoteMove, piece::PieceKind};
use leptos::*;

// use leptos_meta::*;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Color {
    Black,
    Grey,
    White,
}

struct GridIterator {
    q: isize,
    r: isize,
    line: isize,
    colors: [Color; 3],
}

impl GridIterator {
    pub fn new() -> Self {
        GridIterator {
            q: -5,
            r: -2,
            line: 0,
            colors: [Color::Black, Color::Grey, Color::White],
        }
    }
}

impl Iterator for GridIterator {
    type Item = (HexVector, Color);

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

fn hexagon<F>(
    cx: Scope,
    vector: HexVector,
    color: Color,
    board: ReadSignal<Board>,
    selected: ReadSignal<Option<HexVector>>,
    on_select: F,
    legal_moves: Memo<Option<HashSet<MaybePromoteMove>>>,
) -> impl IntoView
where
    F: Fn(HexVector) + 'static,
{
    let hide = vector.mag() > 5;

    let piece = create_memo(cx, move |_| board.get().get_piece_at(vector));

    let piece_image_url = create_memo(cx, move |_| {
        piece.get().map(|piece| {
            use hex_chess_core::piece::Color;
            let color = match piece.color {
                Color::Black => 'b',
                Color::White => 'w',
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
        })
    });

    let selected = move || selected.get().is_some_and(|pos| pos == vector);

    let is_move_dest = create_memo(cx, move |_| {
        legal_moves
            .get()
            .is_some_and(|moves| moves.iter().any(|mov| mov.to() == vector))
    });
    let is_piece_and_dest = move || piece.get().is_some() && is_move_dest.get();

    let on_click = move |_| {
        on_select(vector);
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
                class=("hex-grid__content__black", move || color == Color::Black && !selected())
                class=("hex-grid__content__grey", move || color == Color::Grey && !selected())
                class=("hex-grid__content__white", move || color == Color::White && !selected())
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
        </li>
    }
}

#[component]
pub fn DrawBoard(
    cx: Scope,
    board: ReadSignal<Board>,
    update_board: WriteSignal<Board>,
) -> impl IntoView {
    let (selected, set_selected) = create_signal(cx, None);
    let (_can_promote, set_can_promote) = create_signal(cx, None);

    let play_move = move |from: HexVector, to: HexVector, promote: Option<PieceKind>| {
        update_board.update(|board| match board.play_move(from, to, promote) {
            Ok(None) => set_selected.set(None),
            Ok(Some(promote)) => set_can_promote.set(Some(promote)),
            Err(err) => log!("invalid move: {:?}", err),
        })
    };

    let on_select = move |pos: HexVector| {
        let (target_piece, turn) =
            board.with(|board| (board.get_piece_at(pos), board.get_player_turn()));
        match (selected.get(), target_piece) {
            (_, Some(piece)) if piece.color == turn => set_selected(Some(pos)),
            (Some(selected), _) => play_move(selected, pos, None),
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
        {GridIterator::new().map(|(vector, color)| hexagon(cx, vector, color, board, selected, on_select, current_legal_moves)).collect_view(cx)}
        </ul>
    }
}
