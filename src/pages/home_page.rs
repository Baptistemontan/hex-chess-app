use crate::components::auth::LoggedIn;
use crate::components::board::SoloBoard;
use leptos::*;
use leptos_meta::*;

/// Renders the home page of your application.
#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    let navigate = leptos_router::use_navigate(cx);

    let (redirect, set_redirect) = create_signal(cx, None::<&str>);

    create_effect(cx, move |_| {
        if let Some(value) = redirect.get() {
            navigate(&format!("/play/{}", value), Default::default()).unwrap();
        }
    });

    view! { cx,

        <Title text="Hex Chess | Home"/>
        <h1 class="title">"Welcome to Hex chess!"</h1>
        <div class="board">
            <SoloBoard/>
        </div>
        <LoggedIn>
            <div class="link_to_games">
                <div on:click=move |_| set_redirect.set(Some("random")) class="big_button">
                    <p>"Random Game"</p>
                </div>
                <div on:click=move |_| set_redirect.set(Some("custom")) class="big_button">
                    <p>"Invite Friend"</p>
                </div>
            </div>
        </LoggedIn>
    }
}
