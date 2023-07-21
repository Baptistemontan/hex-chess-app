use crate::components::board::Board;
use leptos::*;
use leptos_meta::*;

/// Renders the home page of your application.
#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    view! { cx,
        <Title text="Hex Chess | Home"/>
        <h1>"Welcome to Hex chess!"</h1>
        <div class="board">
            <Board/>
        </div>
    }
}
