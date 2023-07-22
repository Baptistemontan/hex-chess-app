use crate::components::board::{GameKind, MultiBoard, SoloBoard};
use leptos::*;
use leptos_meta::*;

/// Renders the home page of your application.
#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    let (game_kind, set_game_kind) = create_signal(cx, GameKind::Solo);
    let input_element: NodeRef<html::Input> = create_node_ref(cx);

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let value = input_element.get().expect("<input> to exist").value();
        set_game_kind.set(GameKind::JoinCustom(value));
    };

    view! { cx,
        <Title text="Hex Chess | Home"/>
        <h1>"Welcome to Hex chess!"</h1>
        <div class="board">
            {move || match game_kind.get() {
                GameKind::Solo => view! { cx, <SoloBoard/> },
                other => view! { cx, <MultiBoard game_kind=other/> }
            }}
        </div>
        <div>
            <button on:click=move |_| set_game_kind.set(GameKind::Solo)>"Solo Game"</button>
            <button on:click=move |_| set_game_kind.set(GameKind::Random)>"Random Game"</button>
            <button on:click=move |_| set_game_kind.set(GameKind::Custom)>"Create Custom Game"</button>
            <div>
                <p>"Join game : "</p>
                <form on:submit=on_submit>
                    <input type="text" node_ref=input_element/>
                    <input type="submit" value="Join"/>
                </form>
            </div>
        </div>
    }
}
