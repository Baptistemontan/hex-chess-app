mod home_page;
mod not_found;
pub mod play;

use home_page::HomePage;
use leptos::*;
use leptos_i18n::I18nContextProvider;
use leptos_meta::*;
use leptos_router::*;
use not_found::NotFound;
use play::{Play, WaitingCustom, WaitingCustomWithId, WaitingRandom};

use crate::components::auth::AuthentificationContext;
use crate::components::layout::Layout;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! { cx,
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/hex-chess-app.css"/>
        <Link rel="manifest" href="/manifest.json"/>
        <Meta name="description" content="Website to play hexagonal chess, solo or with friends." />
        <I18nContextProvider>
        <AuthentificationContext>
            <Router>
                <Layout>
                        <Routes>
                            <Route path="/" view=HomePage />
                            <Route path="/play/random" view=WaitingRandom/>
                            <Route path="/play/custom" view=WaitingCustom/>
                            <Route path="/play/custom/:game_id" view=WaitingCustomWithId/>
                            <Route path="/play/:game_id" view=Play/>
                            <Route path="/*any" view=NotFound />
                        </Routes>
                    </Layout>
            </Router>
        </AuthentificationContext>
        </I18nContextProvider>
    }
}
