use crate::{IntegrationError, Result, default_http_client};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const TMDB_BASE_URL: &str = "https://api.themoviedb.org/3";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbSearchResponse {
    pub page: u32,
    #[serde(default)]
    pub results: Vec<TmdbMediaItem>,
    pub total_results: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbMediaItem {
    pub id: u64,
    pub media_type: Option<String>,
    pub title: Option<String>,
    pub name: Option<String>,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub release_date: Option<String>,
    pub first_air_date: Option<String>,
    pub vote_average: Option<f64>,
}

#[derive(Clone)]
pub struct TmdbClient {
    http: Client,
    api_key: String,
    language: String,
}

impl TmdbClient {
    pub fn new(api_key: impl Into<String>, language: impl Into<String>) -> Result<Self> {
        Ok(Self {
            http: default_http_client()?,
            api_key: api_key.into().trim().to_string(),
            language: language.into().trim().to_string(),
        })
    }

    pub async fn search_multi(&self, query: &str, page: u32) -> Result<TmdbSearchResponse> {
        let resp = self
            .http
            .get(format!("{TMDB_BASE_URL}/search/multi"))
            .query(&[
                ("api_key", self.api_key.as_str()),
                ("language", self.language.as_str()),
                ("query", query),
                ("page", &page.to_string()),
            ])
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

    pub async fn details(&self, media_type: &str, tmdb_id: u64) -> Result<Value> {
        let kind = match media_type {
            "series" | "tv" => "tv",
            _ => "movie",
        };
        let resp = self
            .http
            .get(format!("{TMDB_BASE_URL}/{kind}/{tmdb_id}"))
            .query(&[
                ("api_key", self.api_key.as_str()),
                ("language", self.language.as_str()),
                ("append_to_response", "external_ids,images,release_dates,content_ratings"),
            ])
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
