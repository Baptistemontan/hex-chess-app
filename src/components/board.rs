use hex_chess_core::{
    board::Board as HexBoard,
    hex_coord::HexVector,
    mov::{CanPromoteMove, MaybePromoteMove},
    piece::{Color as PieceColor, Piece, PieceKind},
};
use leptos::*;
use std::collections::HashSet;

use crate::server::{GameEvent, PlayMove};

// use leptos_meta::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HexColor {
    Black,
    Grey,
    White,
    Selected,
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

    let color = move || {
        if selected.get().is_some_and(|pos| pos == vector()) {
            HexColor::Selected
        } else if orientation.get() == Orientation::Normal {
            color
        } else {
            color.reverse()
        }
    };

    let hide = move || vector().mag() > 5;

    let piece = create_memo(cx, move |_| board.get().get_piece_at(vector()));

    let piece_image_url = create_memo(cx, move |_| piece.get().map(get_piece_url));

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
                class=("hex-grid__content__black", move || color() == HexColor::Black)
                class=("hex-grid__content__grey", move || color() == HexColor::Grey)
                class=("hex-grid__content__white", move || color() == HexColor::White)
                class=("hex-grid__content__selected", move || color() == HexColor::Selected)
                class=("hex-grid__content__is_dest", move || is_move_dest.get() && !is_piece_and_dest())
                class=("hex-grid__content__is_piece_and_dest", is_piece_and_dest)
            >
                {move || piece_image_url.get().map(|url| {
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

fn draw_hex_board<OS>(
    cx: Scope,
    board: ReadSignal<HexBoard>,
    orientation: ReadSignal<Orientation>,
    selected: ReadSignal<Option<HexVector>>,
    can_promote: ReadSignal<Option<CanPromoteMove>>,
    on_select: OS,
) -> impl IntoView
where
    OS: Fn(HexVector, Option<PieceKind>) + Copy + 'static,
{
    let legal_moves = create_memo(cx, move |_| board.get().get_legal_moves());

    let current_legal_moves = create_memo(cx, move |_| {
        selected
            .get()
            .and_then(|selected| legal_moves.get().get(&selected).cloned())
    });

    view! { cx,
        <ul class="hex-grid__list">
        {move || GridIterator::new().map(|(vector, color)| hexagon(cx, vector, color, board, selected, on_select, current_legal_moves, orientation, can_promote)).collect_view(cx)}
        </ul>
    }
}

fn orientation_manager<OS>(
    cx: Scope,
    board: ReadSignal<HexBoard>,
    selected: ReadSignal<Option<HexVector>>,
    player_color: impl Fn() -> PieceColor + 'static,
    can_promote: ReadSignal<Option<CanPromoteMove>>,
    on_select: OS,
) -> impl IntoView
where
    OS: Fn(HexVector, Option<PieceKind>) + Copy + 'static,
{
    let (orientation, set_orientation) = create_signal(cx, Orientation::Normal);
    create_effect(cx, move |_| {
        let orientation = match player_color() {
            PieceColor::Black => Orientation::Reversed,
            PieceColor::White => Orientation::Normal,
        };
        set_orientation.set(orientation);
    });
    let on_switch = move |_| set_orientation.update(Orientation::reverse_assign);

    view! { cx,
        <div>
            {draw_hex_board(cx, board, orientation, selected, can_promote, on_select)}
            <button on:click=on_switch>"switch side"</button>
        </div>
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GameKind {
    Custom,
    JoinCustom(String),
    Random,
    Solo,
}

#[cfg(all(feature = "hydrate", not(feature = "ssr")))]
fn subscribe_to_events(cx: Scope, game_kind: GameKind) -> ReadSignal<Option<GameEvent>> {
    use futures::StreamExt;
    let url = match game_kind {
        GameKind::Custom => "/api/custom_game".into(),
        GameKind::Random => "/api/random_game".into(),
        GameKind::JoinCustom(id) => format!("/api/join_game/{}", id),
        GameKind::Solo => "".into(),
    };
    let mut source = gloo_net::eventsource::futures::EventSource::new(&url).unwrap();
    let stream = source.subscribe("message").unwrap().map(|value| {
        let (_, event) = value.unwrap();
        let data = event.data().as_string().unwrap();
        let event: GameEvent = serde_json::from_str(&data).unwrap();
        event
    });
    let s = create_signal_from_stream(cx, stream);
    on_cleanup(cx, move || source.close());
    s
}

#[cfg(feature = "ssr")]
fn subscribe_to_events(cx: Scope, _game_kind: GameKind) -> ReadSignal<Option<GameEvent>> {
    create_signal(cx, None).0
}

#[component]
pub fn Board(cx: Scope, game_kind: GameKind) -> impl IntoView {
    let events = subscribe_to_events(cx, game_kind);
    create_effect(cx, move |_| {
        let event = events.get();
        log!("event: {:?}", event);
    });

    let (selected, set_selected) = create_signal(cx, None);
    let (dest, set_dest) = create_signal(cx, None);
    let (can_promote, set_can_promote) = create_signal(cx, None);
    let (board, set_board) = create_signal(cx, HexBoard::new());
    let (player_infos, set_player_infos) = create_signal(cx, (PieceColor::White, None));
    let (custom_game_id, set_custom_game_id) = create_signal(cx, None);

    create_effect(cx, move |_: Option<Option<()>>| {
        let event = events.get()?;
        match event {
            GameEvent::GameStart {
                game_id,
                player_id,
                player_color,
            } => set_player_infos.set((player_color, Some((game_id, player_id)))),
            GameEvent::CustomCreated { game_id } => set_custom_game_id.set(Some(game_id)),
            GameEvent::OpponentPlayedMove {
                from,
                to,
                promote_to,
            } => set_board.update(|board| {
                board.play_move(from, to, promote_to).unwrap();
            }),
            _ => (),
        }
        None
    });

    let player_color = move || player_infos.get().0;

    let play_server_move = create_server_action::<PlayMove>(cx);

    let on_select = move |pos: HexVector, promote_to: Option<PieceKind>| {
        let (color, ids) = player_infos.get();
        if ids.is_none() {
            return;
        }
        let (target_piece, is_turn) =
            board.with(|board| (board.get_piece_at(pos), board.get_player_turn() == color));
        if !is_turn {
            return;
        }
        match (selected.get(), target_piece) {
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
        let (color, ids) = player_infos.get();
        let from = selected.get();
        let to = dest.get();
        if board.with(|board| board.get_player_turn() != color) {
            return;
        }
        if let (Some((game_id, player_id)), Some(from), Some((to, promote_to))) = (ids, from, to) {
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
                    play_server_move.dispatch(PlayMove {
                        game_id,
                        player_id,
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
            {orientation_manager(cx, board, selected, player_color, can_promote, on_select)}
            {move || custom_game_id.get().map(|game_id| view! { cx,
                <p>
                    {format!("Custom game created with id: {}", game_id)}
                </p>
            })}
        </div>

    }
}

#[component]
pub fn SoloBoard(cx: Scope) -> impl IntoView {
    let (board, set_board) = create_signal(cx, HexBoard::new());
    let (selected, set_selected) = create_signal(cx, None);
    let (can_promote, set_can_promote) = create_signal(cx, None);
    let (dest, set_dest) = create_signal(cx, None);

    let player_color = move || board.with(|board| board.get_player_turn());

    let on_select = move |pos: HexVector, promote_to: Option<PieceKind>| {
        let (target_piece, color) =
            board.with(|board| (board.get_piece_at(pos), board.get_player_turn()));
        match (selected.get(), target_piece) {
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
            if let Ok(promote_move) = res {
                set_can_promote.set(promote_move)
            }
        }
    });

    orientation_manager(cx, board, selected, player_color, can_promote, on_select)
}
