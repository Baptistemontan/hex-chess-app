use crate::components::board::Board;
use hex_chess_core::board::Board;
use leptos::*;
use leptos_meta::*;

/// Renders the home page of your application.
#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    // Creates a reactive value to update the button
    let board = create_rw_signal(cx, Board::new());

    view! { cx,
        <Title text="Hex Chess | Home"/>
        <h1>"Welcome to Hex chess!"</h1>
        <div class="board">
            <Board board/>
        </div>
    }
}
