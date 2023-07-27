use leptos::*;

#[derive(Clone, Copy)]
struct IsLoggedIn(ReadSignal<Option<bool>>);

impl IsLoggedIn {
    pub fn is_logged_in(self) -> bool {
        self.0.get().unwrap_or(false)
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
    let is_logged_in = create_signal_from_stream(
        cx,
        Box::pin(futures::stream::once(async move {
            fetch_user_logged_in(cx).await.unwrap_or(false)
        })),
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
        .map(IsLoggedIn::is_logged_in)
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
