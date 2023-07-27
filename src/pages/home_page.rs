use crate::components::auth::{AuthentificationContext, LoggedIn, NotLoggedIn};
use crate::components::board::SoloBoard;
use leptos::*;
use leptos_meta::*;

/// Renders the home page of your application.
#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    let input_element: NodeRef<html::Input> = create_node_ref(cx);

    let navigate = leptos_router::use_navigate(cx);

    let (redirect, set_redirect) = create_signal(cx, None);

    create_effect(cx, move |_| {
        if let Some(value) = redirect.get() {
            navigate(&format!("/play/{}", value), Default::default()).unwrap();
        }
    });

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let value = input_element.get().expect("<input> to exist").value();
        set_redirect.set(Some(value))
    };

    view! { cx,
        <AuthentificationContext>
            <Title text="Hex Chess | Home"/>
            <h1>"Welcome to Hex chess!"</h1>
            <LoggedIn>
                <a href="/api/auth/logout" rel="external">"logout"</a>
            </LoggedIn>
            <NotLoggedIn>
                <a href="/api/auth/login" rel="external">"login"</a>
            </NotLoggedIn>
            <div class="board">
                <SoloBoard/>
            </div>
            <div>
                <LoggedIn>
                    <button on:click=move |_| set_redirect.set(Some("random".into()))>"Random Game"</button>
                    <button on:click=move |_| set_redirect.set(Some("custom".into()))>"Create Custom Game"</button>
                    <div>
                        <p>"Join game : "</p>
                        <form on:submit=on_submit>
                            <input type="text" node_ref=input_element/>
                            <input type="submit" value="Join"/>
                        </form>
                    </div>
                </LoggedIn>
            </div>
        </AuthentificationContext>
    }
}
