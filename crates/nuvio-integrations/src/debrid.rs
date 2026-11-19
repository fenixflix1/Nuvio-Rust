use crate::{IntegrationError, Result, default_http_client};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebridProviderId {
    Torbox,
    Premiumize,
    RealDebrid,
}

impl DebridProviderId {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "torbox" | "tb" => Some(Self::Torbox),
            "premiumize" | "pm" => Some(Self::Premiumize),
            "realdebrid" | "real_debrid" | "real-debrid" | "rd" => Some(Self::RealDebrid),
            _ => None,
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Self::Torbox => "TB",
            Self::Premiumize => "PM",
            Self::RealDebrid => "RD",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedDebridStream {
    pub provider: DebridProviderId,
    pub download_url: String,
    pub filename: Option<String>,
}

#[async_trait]
pub trait DebridProviderClient: Send + Sync {
    fn id(&self) -> DebridProviderId;
    async fn validate_api_key(&self, api_key: &str) -> Result<bool>;
    async fn check_instant_availability(&self, api_key: &str, hash: &str) -> Result<bool>;
}

pub struct RealDebridClient {
    http: Client,
}

impl RealDebridClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            http: default_http_client()?,
        })
    }

    pub async fn unrestrict_link(&self, api_key: &str, link: &str) -> Result<ResolvedDebridStream> {
        let resp = self
            .http
            .post("https://api.real-debrid.com/rest/1.0/unrestrict/link")
            .bearer_auth(api_key)
            .form(&[("link", link)])
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
        let val: Value = serde_json::from_str(&body)?;
        let download_url = val
            .get("download")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IntegrationError::Manifest("Real-Debrid missing download URL".into()))?
            .to_string();
        let filename = val
            .get("filename")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        Ok(ResolvedDebridStream {
            provider: DebridProviderId::RealDebrid,
            download_url,
            filename,
        })
    }
}

#[async_trait]
impl DebridProviderClient for RealDebridClient {
    fn id(&self) -> DebridProviderId {
        DebridProviderId::RealDebrid
    }

    async fn validate_api_key(&self, api_key: &str) -> Result<bool> {
        let resp = self
            .http
            .get("https://api.real-debrid.com/rest/1.0/user")
            .bearer_auth(api_key)
            .send()
            .await?;
        Ok(resp.status().is_success())
    }

    async fn check_instant_availability(&self, api_key: &str, hash: &str) -> Result<bool> {
        let clean = hash.trim().to_ascii_lowercase();
        let resp = self
            .http
            .get(format!(
                "https://api.real-debrid.com/rest/1.0/torrents/instantAvailability/{clean}"
            ))
            .bearer_auth(api_key)
            .send()
            .await?;
        Ok(resp.status().is_success())
    }
}

pub struct TorboxClient {
    http: Client,
}

impl TorboxClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            http: default_http_client()?,
        })
    }
}

#[async_trait]
impl DebridProviderClient for TorboxClient {
    fn id(&self) -> DebridProviderId {
        DebridProviderId::Torbox
    }

    async fn validate_api_key(&self, api_key: &str) -> Result<bool> {
        let resp = self
            .http
            .get("https://api.torbox.app/v1/api/user/me")
            .bearer_auth(api_key)
            .send()
            .await?;
        Ok(resp.status().is_success())
    }

    async fn check_instant_availability(&self, api_key: &str, hash: &str) -> Result<bool> {
        let clean = hash.trim().to_ascii_lowercase();
        let resp = self
            .http
            .get("https://api.torbox.app/v1/api/torrents/checkcached")
            .bearer_auth(api_key)
            .query(&[("hash", clean.as_str()), ("format", "list")])
            .send()
            .await?;
        if !resp.status().is_success() {
            return Ok(false);
        }
        let val: Value = resp.json().await?;
        Ok(val
            .get("data")
            .and_then(|d| d.as_array())
            .is_some_and(|arr| !arr.is_empty()))
    }
}

pub struct PremiumizeClient {
    http: Client,
}

impl PremiumizeClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            http: default_http_client()?,
        })
    }
}

#[async_trait]
impl DebridProviderClient for PremiumizeClient {
    fn id(&self) -> DebridProviderId {
        DebridProviderId::Premiumize
    }

    async fn validate_api_key(&self, api_key: &str) -> Result<bool> {
        let resp = self
            .http
            .get("https://www.premiumize.me/api/account/info")
            .query(&[("apikey", api_key)])
            .send()
            .await?;
        if !resp.status().is_success() {
            return Ok(false);
        }
        let val: Value = resp.json().await?;
        Ok(val.get("status").and_then(|s| s.as_str()) == Some("success"))
    }

    async fn check_instant_availability(&self, api_key: &str, hash: &str) -> Result<bool> {
        let resp = self
            .http
            .get("https://www.premiumize.me/api/cache/check")
            .query(&[("apikey", api_key), ("items[]", hash)])
            .send()
            .await?;
        if !resp.status().is_success() {
            return Ok(false);
        }
        let val: Value = resp.json().await?;
        Ok(val
            .get("response")
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
            .and_then(|b| b.as_bool())
            .unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_debrid_aliases() {
        assert_eq!(DebridProviderId::parse("RD"), Some(DebridProviderId::RealDebrid));
        assert_eq!(DebridProviderId::parse("torbox"), Some(DebridProviderId::Torbox));
        assert_eq!(
            DebridProviderId::parse("premiumize"),
            Some(DebridProviderId::Premiumize)
        );
    }
}
