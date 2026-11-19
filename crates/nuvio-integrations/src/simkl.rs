use crate::{IntegrationError, Result, default_http_client};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const SIMKL_API_URL: &str = "https://api.simkl.com";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimklPinResponse {
    pub result: String,
    pub device_code: Option<String>,
    pub user_code: Option<String>,
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

    pub async fn request_pin(&self) -> Result<SimklPinResponse> {
        let resp = self
            .http
            .get(format!("{SIMKL_API_URL}/oauth/pin"))
            .query(&[("client_id", self.client_id.as_str())])
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

    pub async fn get_all_items(&self, access_token: &str) -> Result<Value> {
        let resp = self
            .http
            .get(format!("{SIMKL_API_URL}/sync/all-items/"))
            .header("simkl-api-key", &self.client_id)
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
}
