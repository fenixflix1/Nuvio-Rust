use crate::{IntegrationError, Result, default_http_client};
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant, sleep_until};

const SIMKL_API_URL: &str = "https://api.simkl.com";
// Simkl spaces reads and account mutations separately across all active clients.
static NEXT_READ: Mutex<Option<Instant>> = Mutex::const_new(None);
static NEXT_WRITE: Mutex<Option<Instant>> = Mutex::const_new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimklPinResponse {
    pub result: String,
    pub device_code: Option<String>,
    pub user_code: Option<String>,
    pub verification_uri: Option<String>,
    pub verification_url: Option<String>,
    pub expires_in: Option<u64>,
    pub interval: Option<u64>,
}

#[derive(Clone)]
pub struct SimklClient {
    http: Client,
    client_id: String,
}

impl SimklClient {
    pub fn new(client_id: impl Into<String>) -> Result<Self> {
        Ok(Self {
            http: default_http_client()?,
            client_id: client_id.into().trim().to_string(),
        })
    }

    pub async fn request(
        &self,
        method: Method,
        path: &str,
        token: Option<&str>,
        query: &[(&str, &str)],
        body: Option<&Value>,
    ) -> Result<Value> {
        let (limit, interval) = if method == Method::GET {
            (&NEXT_READ, Duration::from_millis(100))
        } else {
            (&NEXT_WRITE, Duration::from_secs(1))
        };
        let mut next_request = limit.lock().await;
        if let Some(time) = *next_request {
            sleep_until(time).await;
        }
        let mut req = self
            .http
            .request(method, format!("{SIMKL_API_URL}{path}"))
            .header("simkl-api-key", &self.client_id)
            .query(&[
                ("client_id", self.client_id.as_str()),
                ("app-name", "NuvioDesktop-Rust"),
                ("app-version", env!("CARGO_PKG_VERSION")),
            ])
            .query(query);
        if let Some(token) = token {
            req = req.bearer_auth(token);
        }
        if let Some(body) = body {
            req = req.json(body);
        }
        let resp = req.send().await.map_err(reqwest::Error::without_url);
        *next_request = Some(Instant::now() + interval);
        drop(next_request);
        let resp = resp?;
        let status = resp.status();
        if status.as_u16() == 409 && path == "/scrobble/stop" {
            return Ok(Value::Null);
        }
        if !status.is_success() {
            return Err(IntegrationError::Api {
                status: status.as_u16(),
                body: "Simkl".into(),
            });
        }
        let bytes = resp.bytes().await?;
        if bytes.is_empty() {
            Ok(Value::Null)
        } else {
            Ok(serde_json::from_slice(&bytes)?)
        }
    }

    pub async fn request_pin(&self) -> Result<SimklPinResponse> {
        let value = self
            .request(Method::GET, "/oauth/pin", None, &[], None)
            .await?;
        Ok(serde_json::from_value(value)?)
    }

    pub async fn get_all_items(&self, access_token: &str) -> Result<Value> {
        self.request(
            Method::GET,
            "/sync/all-items",
            Some(access_token),
            &[
                ("extended", "full_anime_seasons"),
                ("episode_watched_at", "yes"),
                ("episode_tvdb_id", "yes"),
                ("include_all_episodes", "yes"),
            ],
            None,
        )
        .await
    }
}
