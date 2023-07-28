mod home_page;
mod not_found;
mod play;

use home_page::HomePage;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use not_found::NotFound;
use play::{Custom, Join, Random};

use crate::components::auth::AuthentificationContext;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! { cx,
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/hex-chess-app.css"/>
        <AuthentificationContext>
            <Router>
                <main>
                    <Routes>
                        <Route path="/" view=HomePage />
                        <Route path="/play/:game_id" view=Join />
                        <Route path="/play/random" view=Random />
                        <Route path="/play/custom" view=Custom />
                        <Route path="/*any" view=NotFound />
                    </Routes>
                </main>
            </Router>
        </AuthentificationContext>
    }
}
