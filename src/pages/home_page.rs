use crate::components::board::DrawBoard;
use hex_chess_core::board::Board;
use leptos::*;
use leptos_meta::*;

/// Renders the home page of your application.
#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(cx, 0);
    let (board, _set_board) = create_signal(cx, Board::new());
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! { cx,
        <Title text="Hex Chess | Home"/>
        <h1>"Welcome to Hex chess!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
        <div class="board">
            <DrawBoard board/>
        </div>
    }
}
