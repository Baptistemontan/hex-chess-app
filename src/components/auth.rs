use leptos::*;
use leptos_router::use_location;

#[derive(Clone, Copy, Debug)]
pub struct IsLoggedIn(pub Resource<(), bool>);

impl IsLoggedIn {
    pub fn is_logged_in(self, cx: Scope) -> bool {
        self.0.read(cx).is_some_and(|v| v)
    }
}

#[cfg(feature = "ssr")]
use crate::server::auth::user_id::MaybeUserId;

// #[cfg(feature = "ssr")]
// pub async fn get_user_id_or_redirect_login(cx: Scope) -> Option<String> {
//     use actix_web::FromRequest;
//     let req = use_context::<actix_web::HttpRequest>(cx)
//         .expect("HttpRequest should have been provided via context");

//     let user_id = MaybeUserId::extract(&req).await.unwrap();

//     if user_id.0.is_none() {
//         leptos_actix::redirect(cx, &format!("/api/auth/login?origin={}", req.uri()));
//     }

//     user_id.0
// }

#[cfg(feature = "ssr")]
async fn is_user_logged_in_inner(user_id: MaybeUserId) -> bool {
    user_id.0.is_some()
}

#[server(FetchIsLoggedIn, "/api")]
async fn fetch_user_logged_in(cx: Scope) -> Result<bool, ServerFnError> {
    use leptos_actix::extract;
    extract(cx, is_user_logged_in_inner).await

    // Ok(get_user_id_or_redirect_login(cx).await.is_some())
}

#[component]
pub fn AuthentificationContext(cx: Scope, children: Children) -> impl IntoView {
    let is_logged_in = create_local_resource(
        cx,
        || (),
        move |_| async move { fetch_user_logged_in(cx).await.unwrap_or(false) },
    );

    provide_context(cx, IsLoggedIn(is_logged_in));

    children(cx)
}

fn render_inner<F: Fn(bool) -> bool>(
    cx: Scope,
    logged_in: Option<IsLoggedIn>,
    should_render: F,
    children: &ChildrenFn,
) -> impl IntoView {
    logged_in
        .map(|is_logged_in| is_logged_in.is_logged_in(cx))
        .and_then(|logged_in| should_render(logged_in).then(|| children(cx)))
}

#[component]
pub fn LoggedIn(cx: Scope, children: ChildrenFn) -> impl IntoView {
    let logged_in = use_context::<IsLoggedIn>(cx);

    move || render_inner(cx, logged_in, |logged_in| logged_in, &children)
}

#[component]
pub fn NotLoggedIn(cx: Scope, children: ChildrenFn) -> impl IntoView {
    let logged_in = use_context::<IsLoggedIn>(cx);

    move || render_inner(cx, logged_in, |logged_in| !logged_in, &children)
}

#[component]
pub fn CheckLoggedIn(cx: Scope, children: ChildrenFn) -> impl IntoView {
    fn render_children(cx: Scope, children: &ChildrenFn) -> impl IntoView {
        children(cx)
    }

    let render = move |cx| render_children(cx, &children);

    let location = use_location(cx);

    let url = move || format!("/api/auth/login?origin={}", location.pathname.get());

    view! { cx,
        <LoggedIn>
            {render(cx)}
        </LoggedIn>
        <NotLoggedIn>
            <h1>"You must be logged to play online"</h1>
            <a href=url rel="external">"login"</a>
        </NotLoggedIn>
    }
}
