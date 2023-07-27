use std::fmt::Display;

use actix_session::{Session, SessionInsertError};
use actix_web::{
    cookie::{
        time::{self, Duration},
        Cookie, CookieBuilder, SameSite,
    },
    http::header,
    *,
};
use oauth2::{AuthorizationCode, CsrfToken, RedirectUrl, TokenResponse};
pub mod auth_client;
pub mod user_id;
use auth_client::AuthClient;
use reqwest::header::HeaderValue;
use serde::Deserialize;

use crate::server::BaseUrl;

use self::user_id::MaybeUserId;

const PCKE_VERIFIER_SESSION_KEY: &str = "__pcke_code_verifier";
const CSRF_TOKEN_SESSION_KEY: &str = "__csrf_token";
const COOKIE_TOKEN_KEY: &str = "access_token";

#[derive(Debug, Deserialize)]
struct LoginParams {
    origin: Option<String>,
}

fn get_redirect_uri(origin: Option<&str>, base_url: &str) -> RedirectUrl {
    let login_callback_url =
        reqwest::Url::parse(&format!("{}/api/auth/callback", base_url)).unwrap();

    let redirect_url = origin
        .and_then(|url| login_callback_url.join(&format!("?origin={}", url)).ok())
        .unwrap_or(login_callback_url);

    RedirectUrl::from_url(redirect_url)
}

#[get("/login")]
async fn login(
    client: web::Data<AuthClient>,
    base_url: web::Data<BaseUrl>,
    session: Session,
    origin: web::Query<LoginParams>,
    user_id: MaybeUserId,
) -> Result<HttpResponse, SessionInsertError> {
    if user_id.0.is_some() {
        return Ok(HttpResponse::Found()
            .append_header((header::LOCATION, origin.origin.as_deref().unwrap_or("/")))
            .finish());
    }

    let redirect_url = get_redirect_uri(origin.origin.as_deref(), &base_url.0);
    let (authorize_url, csrf_token, pkce_verifier) =
        client.get_auth_url(redirect_url, ["openid".into()]);
    session.insert(PCKE_VERIFIER_SESSION_KEY, pkce_verifier)?;
    session.insert(CSRF_TOKEN_SESSION_KEY, csrf_token)?;

    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, authorize_url.to_string()))
        .finish())
    // Ok(HttpResponse::Ok().finish())
}

fn create_session_token_cookie(value: &str, max_age: Duration) -> Cookie {
    CookieBuilder::new(COOKIE_TOKEN_KEY, value)
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(max_age)
        .path("/")
        .finish()
}

fn append_session_token_cookie<'b>(
    res: &'b mut HttpResponseBuilder,
    value: &str,
    max_age: Duration,
) -> Result<&'b mut HttpResponseBuilder, header::InvalidHeaderValue> {
    let cookie = create_session_token_cookie(value, max_age)
        .encoded()
        .to_string();
    let header_val = HeaderValue::from_str(&cookie)?;

    res.append_header((header::SET_COOKIE, header_val));
    Ok(res)
}

#[get("/logout")]
async fn logout(
    client: web::Data<AuthClient>,
    base_url: web::Data<BaseUrl>,
    session: Session,
) -> Result<HttpResponse, header::InvalidHeaderValue> {
    session.purge();

    let mut builder = client.revoke(Some(&base_url.0));

    Ok(append_session_token_cookie(&mut builder, "", Duration::seconds(-1))?.finish())
}

#[derive(Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}

#[derive(Debug)]
enum CallbackError {
    PckeVerifierMissing,
    CsrfTokenMissing,
    InvalidCsrfToken,
}

impl Display for CallbackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallbackError::PckeVerifierMissing => {
                write!(f, "pcke code verifier missing from session.")
            }
            CallbackError::CsrfTokenMissing => write!(f, "csrf token missing from session."),
            CallbackError::InvalidCsrfToken => write!(f, "invalid CSRF token."),
        }
    }
}

impl std::error::Error for CallbackError {}

#[get("/callback")]
async fn login_callback(
    session: Session,
    client: web::Data<AuthClient>,
    base_url: web::Data<BaseUrl>,
    params: web::Query<AuthRequest>,
    origin: web::Query<LoginParams>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let AuthRequest { code, state } = params.into_inner();
    let pkce_verifier = session
        .remove_as(PCKE_VERIFIER_SESSION_KEY)
        .and_then(Result::ok)
        .ok_or(CallbackError::PckeVerifierMissing)?;

    let csrf_token: CsrfToken = session
        .remove_as(CSRF_TOKEN_SESSION_KEY)
        .and_then(Result::ok)
        .ok_or(CallbackError::CsrfTokenMissing)?;
    let code = AuthorizationCode::new(code);
    let state = CsrfToken::new(state);

    if csrf_token.secret() != state.secret() {
        return Err(CallbackError::InvalidCsrfToken.into());
    }

    let redirect_url = get_redirect_uri(origin.origin.as_deref(), &base_url.0);

    let token = client
        .get_token_from_code(code, pkce_verifier, redirect_url)
        .await?;

    // let id = client.get_id(token.access_token().secret()).await?;

    let max_age = token
        .expires_in()
        .map(|duration| duration.as_secs_f64())
        .map(time::Duration::seconds_f64)
        .unwrap_or(time::Duration::DAY);

    let return_to = origin.origin.as_deref().unwrap_or("/");

    let mut builder = HttpResponse::Found();

    Ok(
        append_session_token_cookie(&mut builder, token.access_token().secret(), max_age)?
            .append_header((header::LOCATION, return_to))
            .finish(),
    )
}

#[get("/test")]
pub async fn test(user_id: user_id::UserId) -> impl Responder {
    HttpResponse::Ok().body(user_id.0)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(login)
        .service(login_callback)
        .service(logout)
        .service(test);
}
