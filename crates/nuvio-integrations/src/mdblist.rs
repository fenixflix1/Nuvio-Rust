use crate::{IntegrationError, Result, default_http_client};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const MDBLIST_API_URL: &str = "https://api.mdblist.com";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdbListSummary {
    pub id: u64,
    pub name: String,
    pub items: Option<u64>,
    pub likes: Option<u64>,
}

#[derive(Clone)]
pub struct MdbListClient {
    http: Client,
    api_key: String,
}

impl MdbListClient {
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Ok(Self {
            http: default_http_client()?,
            api_key: api_key.into().trim().to_string(),
        })
    }

    pub async fn ratings(&self, provider: &str, kind: &str, id: &str) -> Result<Vec<Value>> {
        let kind = if kind == "movie" { "movie" } else { "show" };
        let data: Value = self
            .http
            .get(format!("{MDBLIST_API_URL}/{provider}/{kind}/{id}/"))
            .query(&[("apikey", &self.api_key)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(data["ratings"].as_array().cloned().unwrap_or_default())
    }

    pub async fn my_lists(&self) -> Result<Vec<MdbListSummary>> {
        let resp = self
            .http
            .get(format!("{MDBLIST_API_URL}/lists/user"))
            .query(&[("apikey", self.api_key.as_str())])
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

    pub async fn list_items(&self, list_id: u64) -> Result<Value> {
        let resp = self
            .http
            .get(format!("{MDBLIST_API_URL}/lists/{list_id}/items"))
            .query(&[("apikey", self.api_key.as_str())])
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
