use crate::components::auth::CheckLoggedIn;
use crate::components::board::{GameKind, MultiBoard};
use leptos::*;
use leptos_router::*;

#[component]
pub fn Join(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);

    let game_id = move || params.with(|p| p.get("game_id").cloned());

    view! { cx,
        <CheckLoggedIn>
            <div class="board">
            {move || game_id().map(|game_id| {
                view! { cx, <MultiBoard game_kind=GameKind::Join(game_id)/> }
            })}
            </div>
        </CheckLoggedIn>
    }
}

#[component]
pub fn Random(cx: Scope) -> impl IntoView {
    view! { cx,
        <CheckLoggedIn>
            <div class="board">
                <MultiBoard game_kind=GameKind::Random/>
            </div>
        </CheckLoggedIn>
    }
}

#[component]
pub fn Custom(cx: Scope) -> impl IntoView {
    view! { cx,
        <CheckLoggedIn>
            <div class="board">
                <MultiBoard game_kind=GameKind::Custom/>
            </div>
        </CheckLoggedIn>
    }
}
