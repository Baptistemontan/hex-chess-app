use leptos::*;
use leptos_meta::*;

use crate::i18n::i18n_context;
use leptos_i18n::t;

/// 404 - Not Found
#[component]
pub fn NotFound(cx: Scope) -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>(cx);
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    let i18n = i18n_context(cx);

    view! { cx,
        <Title text="Hex Chess | Not Found"/>
        <h1>{t!(i18n, not_found)}</h1>
    }
}
