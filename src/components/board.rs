use hex_chess_core::{board::Board, hex_coord::HexVector};
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

fn hexagon(cx: Scope, vector: HexVector, color: Color, board: ReadSignal<Board>) -> impl IntoView {
    let hide = vector.mag() > 5;
    let piece = board.get().get_piece_at(vector);
    let piece_asset_url = piece.map(|piece| {
        use hex_chess_core::piece::{Color, PieceKind};
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
        format!("assets/pieces/{color}{kind}.png")
    });

    let image = piece_asset_url.map(|url| {
        view! { cx,
            <image class="piece_image" src=url />
        }
    });
    view! { cx,
        <li
            class="hex-grid__item"
            class=("hex-grid__item__hide", hide)
        >
            <div
                class="hex-grid__content"
                class=("hex-grid__content__black", color == Color::Black)
                class=("hex-grid__content__grey", color == Color::Grey)
                class=("hex-grid__content__white", color == Color::White)
            >
                {image}
            </div>
        </li>
    }
}

#[component]
pub fn DrawBoard(cx: Scope, board: ReadSignal<Board>) -> impl IntoView {
    let _ = board;
    view! { cx,
        <ul class="hex-grid__list">
        {GridIterator::new().map(|(vector, color)| hexagon(cx, vector, color, board)).collect_view(cx)}
        </ul>
    }
}
