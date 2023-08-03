use std::fmt::Display;

use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

use actix_session::Session;
use actix_web::{http::header, web, FromRequest, HttpResponse, ResponseError};

use crate::server::auth::COOKIE_TOKEN_KEY;

use super::auth_client::AuthClient;

const SESSION_USER_ID: &str = "session_user_id_key";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaybeUserId(pub Option<String>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserId(pub String);

#[derive(Debug)]
pub enum Impossible {}

impl Display for Impossible {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}

impl ResponseError for Impossible {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        unreachable!()
    }

    fn status_code(&self) -> reqwest::StatusCode {
        unreachable!()
    }
}

type BoxedFut<O> = Pin<Box<dyn Future<Output = O>>>;

pub enum MaybeFuture<O> {
    Future(BoxedFut<O>),
    Value(Option<O>),
}

impl<O: Unpin> Future for MaybeFuture<O> {
    type Output = O;
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.get_mut() {
            MaybeFuture::Future(fut) => fut.as_mut().poll(cx),
            MaybeFuture::Value(val) => Poll::Ready(val.take().expect("Polled a finished future.")),
        }
    }
}

impl<O> MaybeFuture<O> {
    pub fn new_future<F: Future<Output = O> + 'static>(fut: F) -> Self {
        MaybeFuture::Future(Box::pin(fut))
    }

    pub fn new_value(value: O) -> Self {
        MaybeFuture::Value(Some(value))
    }
}

impl FromRequest for MaybeUserId {
    type Error = actix_web::Error;

    type Future = MaybeFuture<Result<Self, Self::Error>>;

    #[inline]
    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let Some(cookie) = req.cookie(COOKIE_TOKEN_KEY) else {
            return MaybeFuture::new_value(Ok(MaybeUserId(None)));
        };

        let session = Session::extract(req).into_inner().unwrap();

        if let Some(user_id) = session.get::<String>(SESSION_USER_ID).ok().flatten() {
            return MaybeFuture::new_value(Ok(MaybeUserId(Some(user_id))));
        }

        let client = match web::Data::<AuthClient>::from_request(req, payload).into_inner() {
            Ok(client) => client,
            Err(err) => return MaybeFuture::new_value(Err(err)),
        };

        let token = cookie.value().to_owned();

        let fut = async move {
            let id = client.get_id(&token).await?;
            session.insert(SESSION_USER_ID, &id).ok();
            Ok(MaybeUserId(Some(id)))
        };

        MaybeFuture::new_future(fut)
    }
}

impl FromRequest for UserId {
    type Error = actix_web::Error;

    type Future = MaybeFuture<Result<Self, Self::Error>>;

    #[inline]
    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let redirect_to = RedirectToLogin {
            redirect_to: req.uri().to_string(),
        };

        let Some(cookie) = req.cookie(COOKIE_TOKEN_KEY) else {
            return MaybeFuture::new_value(Err(redirect_to.into()));
        };

        let client = match web::Data::<AuthClient>::from_request(req, payload).into_inner() {
            Ok(client) => client,
            Err(err) => return MaybeFuture::new_value(Err(err)),
        };

        let token = cookie.value().to_owned();

        let fut = async move {
            let id = client.get_id(&token).await?;
            Ok(UserId(id))
        };

        MaybeFuture::new_future(fut)
    }
}

#[derive(Debug)]
pub struct RedirectToLogin {
    redirect_to: String,
}

impl Display for RedirectToLogin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not logged in.")
    }
}

impl ResponseError for RedirectToLogin {
    fn status_code(&self) -> reqwest::StatusCode {
        reqwest::StatusCode::FOUND
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::Found()
            .append_header((
                header::LOCATION,
                format!("/api/auth/login?origin={}", self.redirect_to),
            ))
            .finish()
    }
}
