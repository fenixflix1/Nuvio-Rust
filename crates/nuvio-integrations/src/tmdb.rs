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
        self.get_resource(
            &format!("{kind}/{tmdb_id}"),
            "external_ids,images,release_dates,content_ratings,credits,videos,recommendations",
        )
        .await
    }

    /// Plain TV details without appended resources: the fast path for schedules.
    pub async fn tv(&self, tmdb_id: u64) -> Result<Value> {
        self.get_resource(&format!("tv/{tmdb_id}"), "").await
    }

    /// TV details with `external_ids` so calendar reconciliation can look up IMDb/TVDB on TVMaze.
    pub async fn tv_with_externals(&self, tmdb_id: u64) -> Result<Value> {
        self.get_resource(&format!("tv/{tmdb_id}"), "external_ids")
            .await
    }

    pub async fn season(&self, series: u64, season: u32) -> Result<Value> {
        self.get_resource(&format!("tv/{series}/season/{season}"), "")
            .await
    }

    pub async fn collection(&self, id: u64) -> Result<Value> {
        self.get_resource(&format!("collection/{id}"), "").await
    }

    pub async fn person(&self, person_id: u64) -> Result<Value> {
        self.get_resource(
            &format!("person/{person_id}"),
            "combined_credits,external_ids",
        )
        .await
    }

    /// Search people by name (used when a cast entry carries no TMDB id).
    pub async fn search_person(&self, query: &str) -> Result<Value> {
        let resp = self
            .http
            .get(format!("{TMDB_BASE_URL}/search/person"))
            .query(&[
                ("api_key", self.api_key.as_str()),
                ("language", self.language.as_str()),
                ("query", query),
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

    async fn get_resource(&self, path: &str, append: &str) -> Result<Value> {
        let resp = self
            .http
            .get(format!("{TMDB_BASE_URL}/{path}"))
            .query(&[
                ("api_key", self.api_key.as_str()),
                ("language", self.language.as_str()),
                ("append_to_response", append),
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
