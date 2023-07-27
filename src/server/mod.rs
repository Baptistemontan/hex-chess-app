pub mod board;

#[cfg(feature = "ssr")]
pub mod auth;

#[cfg(feature = "ssr")]
#[derive(Debug, Clone)]
pub struct BaseUrl(pub String);

#[cfg(feature = "ssr")]
impl BaseUrl {
    pub fn new() -> Self {
        let base_url = std::env::var("BASE_URL").expect("BASE URL must be set.");
        BaseUrl(base_url)
    }
}
#[cfg(feature = "ssr")]
impl Default for BaseUrl {
    fn default() -> Self {
        Self::new()
    }
}
