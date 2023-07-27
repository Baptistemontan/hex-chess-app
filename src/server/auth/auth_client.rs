use actix_web::ResponseError;
use actix_web::{http::header, HttpResponse, HttpResponseBuilder};
use oauth2::{
    basic::BasicClient, url::ParseError, AuthUrl, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenUrl,
};
use reqwest::header::{HeaderMap, InvalidHeaderValue};
use reqwest::Url;
use std::{
    env::{self, VarError},
    fmt::Display,
    ops::Deref,
};

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct AuthClient {
    client: BasicClient,
    authority: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthClientInitError {
    InvalidAuthUrl(ParseError),
    InvalidTokenUrl(ParseError),
    InvalidRedirectUrl(ParseError),
    InvalidRevocationUrl(ParseError),
    MissingClientId(VarError),
    MissingClientSecret(VarError),
    MissingAuthority(VarError),
}

impl Display for AuthClientInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthClientInitError::InvalidAuthUrl(err) => write!(f, "invalid auth url: {}", err),
            AuthClientInitError::InvalidTokenUrl(err) => write!(f, "invalid token url: {}", err),
            AuthClientInitError::InvalidRedirectUrl(err) => {
                write!(f, "invalid redirect url: {}", err)
            }
            AuthClientInitError::InvalidRevocationUrl(err) => {
                write!(f, "invalid revocation url: {}", err)
            }
            AuthClientInitError::MissingClientId(err) => write!(f, "missing client id: {}", err),
            AuthClientInitError::MissingClientSecret(err) => {
                write!(f, "missing client secret: {}", err)
            }
            AuthClientInitError::MissingAuthority(err) => write!(f, "missing authority: {}", err),
        }
    }
}

impl std::error::Error for AuthClientInitError {}

#[derive(Debug)]
pub enum UserIdError {
    ReqwestError(reqwest::Error),
    InvalidHeaderValue(InvalidHeaderValue),
}

impl From<reqwest::Error> for UserIdError {
    fn from(value: reqwest::Error) -> Self {
        UserIdError::ReqwestError(value)
    }
}

impl From<InvalidHeaderValue> for UserIdError {
    fn from(value: InvalidHeaderValue) -> Self {
        UserIdError::InvalidHeaderValue(value)
    }
}

impl Display for UserIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserIdError::InvalidHeaderValue(err) => err.fmt(f),
            UserIdError::ReqwestError(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for UserIdError {}

impl ResponseError for UserIdError {}

#[derive(Deserialize)]
struct UserInfoResponse {
    sub: String,
}

impl AuthClient {
    pub fn into_inner(self) -> BasicClient {
        self.client
    }

    pub fn new() -> Result<Self, AuthClientInitError> {
        let authority = env::var("AUTHORITY").map_err(AuthClientInitError::MissingAuthority)?;
        let client = create_client(&authority)?;

        Ok(AuthClient { client, authority })
    }

    pub fn revoke(&self, return_to: Option<&str>) -> HttpResponseBuilder {
        let logout_url = match return_to {
            Some(return_to) => format!(
                "{}/v2/logout?client_id={}&returnTo={}",
                self.authority,
                self.client.client_id().as_str(),
                return_to
            ),
            None => format!(
                "{}/v2/logout?client_id={}",
                self.authority,
                self.client.client_id().as_str()
            ),
        };

        let mut res = HttpResponse::Found();
        res.append_header((header::LOCATION, logout_url));
        res
    }

    pub async fn get_id(&self, token: &str) -> Result<String, UserIdError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", token))?,
        );
        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()?;

        let url = format!("{}/userinfo", self.authority);

        let res: UserInfoResponse = client.get(url).send().await?.json().await?;

        Ok(res.sub)
    }

    pub fn get_auth_url(
        &self,
        redirect_url: RedirectUrl,
        scopes: impl IntoIterator<Item = String>,
    ) -> (Url, CsrfToken, PkceCodeVerifier) {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (authorize_url, csrf_token) = self
            .client
            .clone()
            .set_redirect_uri(redirect_url)
            .authorize_url(CsrfToken::new_random)
            .add_scopes(scopes.into_iter().map(Scope::new))
            .set_pkce_challenge(pkce_challenge)
            .url();

        (authorize_url, csrf_token, pkce_verifier)
    }

    pub async fn get_token_from_code(
        &self,
        code: oauth2::AuthorizationCode,
        pkce_verifier: PkceCodeVerifier,
        redirect_url: RedirectUrl,
    ) -> std::result::Result<
        oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
        oauth2::RequestTokenError<
            oauth2::reqwest::Error<reqwest::Error>,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    > {
        self.client
            .clone()
            .set_redirect_uri(redirect_url)
            .exchange_code(code)
            .set_pkce_verifier(pkce_verifier)
            .request_async(oauth2::reqwest::async_http_client)
            .await
    }
}

impl AsRef<BasicClient> for AuthClient {
    fn as_ref(&self) -> &BasicClient {
        &self.client
    }
}

impl Deref for AuthClient {
    type Target = BasicClient;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

fn create_client(authority: &str) -> Result<BasicClient, AuthClientInitError> {
    let client_id =
        ClientId::new(env::var("AUTH0_CLIENT_ID").map_err(AuthClientInitError::MissingClientId)?);
    let client_secret = ClientSecret::new(
        env::var("AUTH0_CLIENT_SECRET").map_err(AuthClientInitError::MissingClientSecret)?,
    );

    let auth_url = AuthUrl::new(format!("{}/authorize", authority))
        .map_err(AuthClientInitError::InvalidAuthUrl)?;
    let token_url = TokenUrl::new(format!("{}/oauth/token", authority))
        .map_err(AuthClientInitError::InvalidTokenUrl)?;

    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url));

    Ok(client)
}
