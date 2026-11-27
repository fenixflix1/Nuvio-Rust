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

    pub async fn user_lists_raw(&self) -> Result<Value> {
        self.get_json(
            "/lists/user",
            &[("unified", "false"), ("sort", "ranked")],
        )
        .await
    }

    pub async fn watchlist_items(&self) -> Result<Value> {
        self.get_json(
            "/watchlist/items",
            &[
                ("limit", "1000"),
                ("unified", "true"),
                ("append_to_response", "poster,description,genres"),
            ],
        )
        .await
    }

    pub async fn list_items_rich(&self, list_id: u64) -> Result<Value> {
        self.get_json(
            &format!("/lists/{list_id}/items"),
            &[
                ("limit", "1000"),
                ("unified", "true"),
                ("append_to_response", "poster,description,genres"),
            ],
        )
        .await
    }

    pub async fn mutate_watchlist(&self, action: &str, body: &Value) -> Result<Value> {
        let act = if action == "remove" { "remove" } else { "add" };
        self.post_json(&format!("/watchlist/items/{act}"), body).await
    }

    pub async fn mutate_list(&self, list_id: u64, action: &str, body: &Value) -> Result<Value> {
        let act = if action == "remove" { "remove" } else { "add" };
        self.post_json(&format!("/lists/{list_id}/items/{act}"), body).await
    }

    pub async fn user_profile(&self) -> Result<Value> {
        self.get_json("/user", &[]).await
    }

    pub async fn sync_watched(&self) -> Result<Value> {
        self.get_json(
            "/sync/watched",
            &[("limit", "1000"), ("append_to_response", "poster")],
        )
        .await
    }

    pub async fn sync_playback(&self) -> Result<Value> {
        self.get_json("/sync/playback", &[]).await
    }

    pub async fn mutate_watched(&self, remove: bool, body: &Value) -> Result<Value> {
        if remove {
            self.post_json("/sync/watched/remove", body).await
        } else {
            self.post_json("/sync/watched", body).await
        }
    }

    pub async fn scrobble(&self, action: &str, body: &Value) -> Result<Value> {
        let path = match action {
            "start" => "/scrobble/start",
            "pause" => "/scrobble/pause",
            "stop" => "/scrobble/stop",
            "clear" => "/scrobble/clear",
            _ => "/scrobble/pause",
        };
        self.post_json(path, body).await
    }

    async fn get_json(&self, path: &str, extra_query: &[(&str, &str)]) -> Result<Value> {
        let mut query = Vec::with_capacity(extra_query.len() + 1);
        query.push(("apikey", self.api_key.as_str()));
        query.extend_from_slice(extra_query);
        let resp = self
            .http
            .get(format!("{MDBLIST_API_URL}{path}"))
            .query(&query)
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

    async fn post_json(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self
            .http
            .post(format!("{MDBLIST_API_URL}{path}"))
            .query(&[("apikey", self.api_key.as_str())])
            .json(body)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(IntegrationError::Api {
                status: status.as_u16(),
                body: text,
            });
        }
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }
        Ok(serde_json::from_str(&text)?)
    }
}
