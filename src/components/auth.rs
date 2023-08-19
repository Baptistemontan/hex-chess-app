use std::rc::Rc;

use crate::i18n::i18n_context;
use leptos::*;
use leptos_i18n::t;

#[derive(Clone, Copy, Debug)]
pub struct IsLoggedIn(pub Resource<(), bool>);

#[cfg(feature = "ssr")]
use crate::server::auth::user_id::MaybeUserId;

#[cfg(feature = "ssr")]
async fn is_user_logged_in_inner(user_id: MaybeUserId) -> bool {
    user_id.0.is_some()
}

#[server(FetchIsLoggedIn, "/api")]
async fn fetch_user_logged_in(cx: Scope) -> Result<bool, ServerFnError> {
    use leptos_actix::extract;
    extract(cx, is_user_logged_in_inner).await
}

#[component]
pub fn AuthentificationContext(cx: Scope, children: ChildrenFn) -> impl IntoView {
    let is_logged_in = create_blocking_resource(
        cx,
        || (),
        move |_| async move { fetch_user_logged_in(cx).await.unwrap_or(false) },
    );

    provide_context(cx, IsLoggedIn(is_logged_in));

    let children = store_value(cx, Rc::new(children));

    let render = move || children.get_value()(cx);

    view! { cx,
        <Suspense fallback=render >
            {render}
        </Suspense>
    }
}

fn render_inner(cx: Scope, should_render: fn(bool) -> bool, children: ChildrenFn) -> impl IntoView {
    let is_logged_in = use_context::<IsLoggedIn>(cx)
        .expect("Auth Components can only be used inside a AuthContextProvider");

    let is_logged_in = move || is_logged_in.0.read(cx).unwrap_or(false);

    let should_render = move || should_render(is_logged_in());
    // let should_render = move || should_render(true);

    let render = move || should_render().then(|| children(cx));

    view! { cx,
        {}
        {render}
    }
}

#[component]
pub fn LoggedIn(cx: Scope, children: ChildrenFn) -> impl IntoView {
    render_inner(cx, |logged_in| logged_in, children)
}

#[component]
pub fn NotLoggedIn(cx: Scope, children: ChildrenFn) -> impl IntoView {
    render_inner(cx, |logged_in| !logged_in, children)
}

#[component]
pub fn CheckLoggedIn(cx: Scope, children: ChildrenFn) -> impl IntoView {
    let i18n = i18n_context(cx);

    view! { cx,
        <LoggedIn>
            {children(cx)}
        </LoggedIn>
        <NotLoggedIn>
            <h1>{t!(i18n, not_logged_in)}</h1>
        </NotLoggedIn>
    }
}
