pub mod addons;
pub mod debrid;
pub mod mdblist;
pub mod simkl;
pub mod telemetry;
pub mod tmdb;
pub mod trakt;

#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("API error ({status}): {body}")]
    Api { status: u16, body: String },
    #[error("Manifest error: {0}")]
    Manifest(String),
}

pub type Result<T> = std::result::Result<T, IntegrationError>;

pub(crate) fn default_http_client() -> Result<reqwest::Client> {
    Ok(nuvio_storage::dns::client_builder()
        .user_agent("NuvioDesktop-Rust/0.1.0")
        .timeout(std::time::Duration::from_secs(8))
        .build()?)
}
