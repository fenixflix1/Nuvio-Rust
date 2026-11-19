use crate::{IntegrationError, Result, default_http_client};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const TRAKT_API_URL: &str = "https://api.trakt.tv";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraktCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraktToken {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraktPlaybackItem {
    pub id: u64,
    pub progress: f64,
    pub paused_at: Option<String>,
    pub r#type: String,
    pub movie: Option<Value>,
    pub episode: Option<Value>,
    pub show: Option<Value>,
}

#[derive(Clone)]
pub struct TraktClient {
    http: Client,
    creds: TraktCredentials,
}

impl TraktClient {
    pub fn new(creds: TraktCredentials) -> Result<Self> {
        Ok(Self {
            http: default_http_client()?,
            creds,
        })
    }

    pub async fn exchange_code(&self, code: &str) -> Result<TraktToken> {
        let resp = self
            .http
            .post(format!("{TRAKT_API_URL}/oauth/token"))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "code": code,
                "client_id": self.creds.client_id,
                "client_secret": self.creds.client_secret,
                "redirect_uri": self.creds.redirect_uri,
                "grant_type": "authorization_code"
            }))
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(IntegrationError::Api {
                status: status.as_u16(),
                body,
            });
        }
        Ok(serde_json::from_str(&body)?)
    }

    pub async fn get_playback(&self, access_token: &str) -> Result<Vec<TraktPlaybackItem>> {
        let resp = self
            .http
            .get(format!("{TRAKT_API_URL}/sync/playback"))
            .header("trakt-api-version", "2")
            .header("trakt-api-key", &self.creds.client_id)
            .bearer_auth(access_token)
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(IntegrationError::Api {
                status: status.as_u16(),
                body,
            });
        }
        Ok(serde_json::from_str(&body)?)
    }

    pub async fn scrobble(
        &self,
        access_token: &str,
        action: &str,
        payload: &Value,
    ) -> Result<Value> {
        let resp = self
            .http
            .post(format!("{TRAKT_API_URL}/scrobble/{action}"))
            .header("trakt-api-version", "2")
            .header("trakt-api-key", &self.creds.client_id)
            .bearer_auth(access_token)
            .json(payload)
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(IntegrationError::Api {
                status: status.as_u16(),
                body,
            });
        }
        Ok(serde_json::from_str(&body)?)
    }
}
