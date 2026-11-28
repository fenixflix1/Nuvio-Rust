use crate::{IntegrationError, Result, default_http_client};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddonManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub logo_url: Option<String>,
    pub resources: Vec<AddonResource>,
    pub types: Vec<String>,
    pub id_prefixes: Vec<String>,
    pub catalogs: Vec<AddonCatalog>,
    pub behavior_hints: AddonBehaviorHints,
    pub transport_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddonResource {
    pub name: String,
    pub types: Vec<String>,
    pub id_prefixes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddonCatalog {
    pub r#type: String,
    pub id: String,
    pub name: String,
    pub extra: Vec<AddonExtraProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddonExtraProperty {
    pub name: String,
    pub is_required: bool,
    pub options: Vec<String>,
    pub options_limit: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddonBehaviorHints {
    pub configurable: bool,
    pub configuration_required: bool,
    pub adult: bool,
    pub p2p: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetaPreview {
    pub id: String,
    pub r#type: String,
    pub name: String,
    pub poster: Option<String>,
    pub banner: Option<String>,
    pub logo: Option<String>,
    pub poster_shape: String,
    pub description: Option<String>,
    pub release_info: Option<String>,
    pub imdb_rating: Option<String>,
    pub genres: Vec<String>,
    pub runtime: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonStreamResponse {
    #[serde(default)]
    pub streams: Vec<AddonStream>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonStream {
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "infoHash")]
    pub info_hash: Option<String>,
    #[serde(rename = "fileIdx")]
    pub file_idx: Option<i32>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(rename = "behaviorHints")]
    pub behavior_hints: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonSubtitle {
    pub id: String,
    pub url: String,
    pub lang: String,
    pub addon_name: Option<String>,
}

impl AddonCatalog {
    pub fn supports_search(&self) -> bool {
        self.extra.iter().any(|p| p.name == "search")
            && self.extra.iter().all(|p| !p.is_required || p.name == "search")
    }
}

impl AddonManifest {
    pub fn parse(manifest_url: &str, payload: &str) -> Result<Self> {
        let root: Value = serde_json::from_str(payload)?;
        let obj = root
            .as_object()
            .ok_or_else(|| IntegrationError::Manifest("Manifest root must be an object".into()))?;

        let req_str = |key: &str| -> Result<String> {
            obj.get(key)
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .ok_or_else(|| IntegrationError::Manifest(format!("Missing required field: {key}")))
        };

        let opt_str = |key: &str| -> Option<String> {
            obj.get(key)
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        };

        let default_types = str_list(obj.get("types"));
        let default_prefixes = str_list(obj.get("idPrefixes"));

        let resources = obj
            .get("resources")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| match item {
                        Value::String(name) => {
                            let n = name.trim();
                            if n.is_empty() {
                                None
                            } else {
                                Some(AddonResource {
                                    name: n.to_string(),
                                    types: default_types.clone(),
                                    id_prefixes: default_prefixes.clone(),
                                })
                            }
                        }
                        Value::Object(map) => {
                            let name = map.get("name")?.as_str()?.trim().to_string();
                            if name.is_empty() {
                                return None;
                            }
                            let types = {
                                let t = str_list(map.get("types"));
                                if t.is_empty() {
                                    default_types.clone()
                                } else {
                                    t
                                }
                            };
                            let id_prefixes = {
                                let p = str_list(map.get("idPrefixes"));
                                if p.is_empty() {
                                    default_prefixes.clone()
                                } else {
                                    p
                                }
                            };
                            Some(AddonResource {
                                name,
                                types,
                                id_prefixes,
                            })
                        }
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let catalogs = obj
            .get("catalogs")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let cat = item.as_object()?;
                        let c_type = cat.get("type")?.as_str()?.to_string();
                        let id = cat.get("id")?.as_str()?.to_string();
                        let name = cat
                            .get("name")
                            .and_then(|v| v.as_str())
                            .filter(|s| !s.trim().is_empty())
                            .unwrap_or(&id)
                            .to_string();
                        let mut extra: Vec<AddonExtraProperty> = cat
                            .get("extra")
                            .and_then(|v| v.as_array())
                            .map(|extras| {
                                extras
                                    .iter()
                                    .filter_map(|e| {
                                        let eo = e.as_object()?;
                                        let ename = eo.get("name")?.as_str()?.trim().to_string();
                                        if ename.is_empty() {
                                            return None;
                                        }
                                        Some(AddonExtraProperty {
                                            name: ename,
                                            is_required: eo
                                                .get("isRequired")
                                                .and_then(|v| v.as_bool())
                                                .unwrap_or(false),
                                            options: str_list(eo.get("options")),
                                            options_limit: eo
                                                .get("optionsLimit")
                                                .and_then(|v| v.as_i64())
                                                .map(|v| v as i32),
                                        })
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        // Fallback for Stremio v3 legacy manifest fields (`extraSupported`, `extraRequired`, `genres`).
                        let extra_required = str_list(cat.get("extraRequired"));
                        for supported in str_list(cat.get("extraSupported")) {
                            if !extra.iter().any(|p| p.name == supported) {
                                let options = if supported == "genre" {
                                    str_list(cat.get("genres"))
                                } else {
                                    Vec::new()
                                };
                                extra.push(AddonExtraProperty {
                                    is_required: extra_required.iter().any(|r| r == &supported),
                                    name: supported,
                                    options,
                                    options_limit: None,
                                });
                            }
                        }
                        for req in extra_required {
                            if !extra.iter().any(|p| p.name == req) {
                                extra.push(AddonExtraProperty {
                                    name: req,
                                    is_required: true,
                                    options: Vec::new(),
                                    options_limit: None,
                                });
                            }
                        }
                        if extra.iter().all(|p| p.name != "genre") {
                            let genres = str_list(cat.get("genres"));
                            if !genres.is_empty() {
                                extra.push(AddonExtraProperty {
                                    name: "genre".to_string(),
                                    is_required: false,
                                    options: genres,
                                    options_limit: None,
                                });
                            }
                        }

                        Some(AddonCatalog {
                            r#type: c_type,
                            id,
                            name,
                            extra,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let hints_obj = obj.get("behaviorHints").and_then(|v| v.as_object());
        let behavior_hints = AddonBehaviorHints {
            configurable: hints_obj
                .and_then(|h| h.get("configurable"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            configuration_required: hints_obj
                .and_then(|h| h.get("configurationRequired"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            adult: hints_obj
                .and_then(|h| h.get("adult"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            p2p: hints_obj
                .and_then(|h| h.get("p2p"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        };

        let logo_url = opt_str("logo")
            .or_else(|| opt_str("icon"))
            .map(|raw| resolve_against_manifest(manifest_url, &raw));

        Ok(Self {
            id: req_str("id")?,
            name: req_str("name")?,
            description: opt_str("description").unwrap_or_default(),
            version: req_str("version")?,
            logo_url,
            resources,
            types: default_types,
            id_prefixes: default_prefixes,
            catalogs,
            behavior_hints,
            transport_url: manifest_url.to_string(),
        })
    }

    pub fn supports_resource(&self, name: &str, media_type: &str, id: &str) -> bool {
        self.resources.iter().any(|r| {
            r.name == name
                && r.types.iter().any(|t| t == media_type)
                && (r.id_prefixes.is_empty() || r.id_prefixes.iter().any(|p| id.starts_with(p)))
        })
    }

    pub fn subtitles_resource_name(&self, media_type: &str, id: &str) -> Option<&str> {
        let canonical_type = if media_type == "tv" { "series" } else { media_type };
        self.resources
            .iter()
            .find(|r| {
                (r.name == "subtitles" || r.name == "subtitle")
                    && (r.types.is_empty()
                        || r.types
                            .iter()
                            .any(|t| t == media_type || t == canonical_type))
                    && (r.id_prefixes.is_empty() || r.id_prefixes.iter().any(|p| id.starts_with(p)))
            })
            .map(|r| r.name.as_str())
    }

    pub fn is_tracking_addon(&self) -> bool {
        self.resources
            .iter()
            .any(|r| r.name == "player" || r.name == "library")
    }

    pub fn base_transport_url(&self) -> String {
        self.transport_url
            .split('?')
            .next()
            .unwrap_or(&self.transport_url)
            .trim_end_matches("/manifest.json")
            .trim_end_matches('/')
            .to_string()
    }

    pub fn resource_url(
        &self,
        resource: &str,
        media_type: &str,
        id: &str,
        extra: Option<&str>,
    ) -> String {
        let base = self.base_transport_url();
        let query = self
            .transport_url
            .split_once('?')
            .map(|(_, q)| format!("?{q}"))
            .unwrap_or_default();
        let encoded_id = url::form_urlencoded::byte_serialize(id.as_bytes())
            .collect::<String>()
            .replace('+', "%20");
        match extra.filter(|s| !s.trim().is_empty()) {
            Some(ex) => format!("{base}/{resource}/{media_type}/{encoded_id}/{ex}.json{query}"),
            None => format!("{base}/{resource}/{media_type}/{encoded_id}.json{query}"),
        }
    }
}

#[derive(Clone)]
pub struct AddonClient {
    http: Client,
}

impl AddonClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            http: default_http_client()?,
        })
    }

    pub async fn fetch_manifest(&self, manifest_url: &str) -> Result<AddonManifest> {
        let resp = self.http.get(manifest_url).send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(IntegrationError::Api {
                status: status.as_u16(),
                body,
            });
        }
        AddonManifest::parse(manifest_url, &body)
    }

    pub async fn fetch_streams(
        &self,
        manifest: &AddonManifest,
        media_type: &str,
        id: &str,
    ) -> Result<Vec<AddonStream>> {
        let url = manifest.resource_url("stream", media_type, id, None);
        let resp = self.http.get(url).send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(IntegrationError::Api {
                status: status.as_u16(),
                body,
            });
        }
        let parsed: AddonStreamResponse = serde_json::from_str(&body)?;
        Ok(parsed.streams)
    }

    pub async fn fetch_catalog(
        &self,
        manifest: &AddonManifest,
        media_type: &str,
        catalog_id: &str,
        extra: Option<&str>,
        max_items: Option<usize>,
    ) -> Result<Vec<MetaPreview>> {
        let url = manifest.resource_url("catalog", media_type, catalog_id, extra);
        let resp = self.http.get(url).send().await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            return Err(IntegrationError::Api {
                status: status.as_u16(),
                body: String::from_utf8_lossy(&bytes).into_owned(),
            });
        }
        Ok(parse_catalog_metas(&bytes, media_type, max_items)?)
    }

    pub async fn fetch_meta(
        &self,
        manifest: &AddonManifest,
        media_type: &str,
        id: &str,
    ) -> Result<Value> {
        let url = manifest.resource_url("meta", media_type, id, None);
        let resp = self.http.get(url).send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(IntegrationError::Api {
                status: status.as_u16(),
                body,
            });
        }
        let root: Value = serde_json::from_str(&body)?;
        Ok(root.get("meta").cloned().unwrap_or(root))
    }

    pub async fn fetch_subtitles(
        &self,
        manifest: &AddonManifest,
        media_type: &str,
        id: &str,
        extra: Option<&str>,
    ) -> Result<Vec<AddonSubtitle>> {
        let Some(resource) = manifest.subtitles_resource_name(media_type, id) else {
            return Ok(Vec::new());
        };
        let canonical_type = if media_type == "tv" { "series" } else { media_type };
        let url = manifest.resource_url(resource, canonical_type, id, extra);
        let resp = self.http.get(url).send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(IntegrationError::Api {
                status: status.as_u16(),
                body,
            });
        }
        let root: Value = serde_json::from_str(&body)?;
        let subs = root
            .get("subtitles")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut out = Vec::new();
        for item in subs {
            let Some(obj) = item.as_object() else {
                continue;
            };
            let Some(sub_url) = obj
                .get("url")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
            else {
                continue;
            };
            let id = obj
                .get("id")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or(sub_url)
                .to_string();
            let lang = obj
                .get("lang")
                .or_else(|| obj.get("language"))
                .or_else(|| obj.get("languageCode"))
                .or_else(|| obj.get("locale"))
                .or_else(|| obj.get("label"))
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("und")
                .to_string();
            out.push(AddonSubtitle {
                id,
                url: sub_url.to_string(),
                lang,
                addon_name: Some(manifest.name.clone()),
            });
        }
        Ok(out)
    }

    pub async fn fetch_text(&self, url: &str) -> Result<String> {
        let resp = self.http.get(url).send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(IntegrationError::Api {
                status: status.as_u16(),
                body,
            });
        }
        Ok(body)
    }

    /// Computes OpenSubtitles 64-bit hash via HTTP Range headers (head 64 KiB + tail 64 KiB + file size).
    pub async fn compute_opensubtitles_hash(&self, stream_url: &str) -> Option<(String, u64)> {
        if !(stream_url.starts_with("http://") || stream_url.starts_with("https://")) {
            return None;
        }
        let head_resp = self.http.head(stream_url).send().await.ok()?;
        let len_str = head_resp
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)?
            .to_str()
            .ok()?;
        let file_size: u64 = len_str.parse().ok()?;
        if file_size < 131_072 {
            return None;
        }

        let head_range = "bytes=0-65535";
        let tail_start = file_size - 65_536;
        let tail_range = format!("bytes={tail_start}-{}", file_size - 1);

        let (head_res, tail_res) = tokio::join!(
            self.http.get(stream_url).header("Range", head_range).send(),
            self.http.get(stream_url).header("Range", &tail_range).send(),
        );
        let head_bytes = head_res.ok()?.bytes().await.ok()?;
        let tail_bytes = tail_res.ok()?.bytes().await.ok()?;
        if head_bytes.len() < 65_536 || tail_bytes.len() < 65_536 {
            return None;
        }

        let mut hash: u64 = file_size;
        for &chunk in head_bytes[..65_536].as_chunks::<8>().0 {
            hash = hash.wrapping_add(u64::from_le_bytes(chunk));
        }
        for &chunk in tail_bytes[..65_536].as_chunks::<8>().0 {
            hash = hash.wrapping_add(u64::from_le_bytes(chunk));
        }
        Some((format!("{hash:016x}"), file_size))
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct RawCatalogResponse {
    metas: Vec<RawCatalogMeta>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct RawCatalogMeta {
    id: Option<String>,
    #[serde(rename = "type")]
    r#type: Option<String>,
    name: Option<String>,
    poster: Option<String>,
    banner: Option<String>,
    background: Option<String>,
    #[serde(rename = "landscapePoster")]
    landscape_poster: Option<String>,
    logo: Option<String>,
    #[serde(rename = "posterShape")]
    poster_shape: Option<String>,
    description: Option<String>,
    #[serde(rename = "releaseInfo")]
    release_info: Option<String>,
    released: Option<String>,
    #[serde(rename = "imdbRating", deserialize_with = "deserialize_rating")]
    imdb_rating: Option<String>,
    #[serde(deserialize_with = "deserialize_opt_str_list")]
    genres: Vec<String>,
    runtime: Option<String>,
}

fn deserialize_rating<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val: Option<Value> = Option::deserialize(deserializer)?;
    Ok(match val {
        Some(Value::String(s)) => {
            let t = s.trim();
            (!t.is_empty()).then(|| t.to_string())
        }
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => None,
    })
}

fn deserialize_opt_str_list<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val: Option<Vec<Value>> = Option::deserialize(deserializer)?;
    Ok(val
        .into_iter()
        .flatten()
        .filter_map(|v| match v {
            Value::String(s) => {
                let t = s.trim();
                (!t.is_empty()).then(|| t.to_string())
            }
            _ => None,
        })
        .collect())
}

fn clean_opt(v: Option<String>) -> Option<String> {
    v.and_then(|s| {
        let t = s.trim();
        if t.is_empty() {
            None
        } else if t.len() == s.len() {
            Some(s)
        } else {
            Some(t.to_string())
        }
    })
}

pub fn parse_catalog_metas(
    bytes: &[u8],
    default_media_type: &str,
    max_items: Option<usize>,
) -> Result<Vec<MetaPreview>> {
    let root: RawCatalogResponse = serde_json::from_slice(bytes)?;
    let cap = max_items
        .map(|m| m.min(root.metas.len()))
        .unwrap_or(root.metas.len());
    let mut items = Vec::with_capacity(cap);
    for meta in root.metas {
        if max_items.is_some_and(|m| items.len() >= m) {
            break;
        }
        let Some(id) = meta.id.filter(|s| !s.is_empty()) else {
            continue;
        };
        let Some(name) = meta.name.filter(|s| !s.is_empty()) else {
            continue;
        };
        let item_type = meta
            .r#type
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| default_media_type.to_string());
        let banner = clean_opt(meta.banner)
            .or_else(|| clean_opt(meta.background))
            .or_else(|| clean_opt(meta.landscape_poster));
        items.push(MetaPreview {
            id,
            r#type: item_type,
            name,
            poster: clean_opt(meta.poster),
            banner,
            logo: clean_opt(meta.logo),
            poster_shape: clean_opt(meta.poster_shape).unwrap_or_else(|| "poster".to_string()),
            description: clean_opt(meta.description),
            release_info: clean_opt(meta.release_info).or_else(|| clean_opt(meta.released)),
            imdb_rating: meta.imdb_rating,
            genres: meta.genres,
            runtime: clean_opt(meta.runtime),
        });
    }
    Ok(items)
}

fn str_list(val: Option<&Value>) -> Vec<String> {
    val.and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(str::trim).filter(|s| !s.is_empty()))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn resolve_against_manifest(manifest_url: &str, candidate: &str) -> String {
    if candidate.starts_with("http://") || candidate.starts_with("https://") {
        return candidate.to_string();
    }
    Url::parse(manifest_url)
        .and_then(|base| base.join(candidate))
        .map(|u| u.to_string())
        .unwrap_or_else(|_| candidate.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stremio_manifest_strings_and_objects() {
        let payload = r#"{
            "id": "org.nuvio.test",
            "name": "Test Addon",
            "version": "1.2.3",
            "types": ["movie", "series"],
            "idPrefixes": ["tt"],
            "resources": [
                "catalog",
                {"name": "stream", "types": ["movie"], "idPrefixes": ["tt"]}
            ],
            "catalogs": [{"type": "movie", "id": "top", "name": "Top Movies"}],
            "behaviorHints": {"p2p": true}
        }"#;
        let m = AddonManifest::parse("https://addon.example/manifest.json", payload).unwrap();
        assert!(m.supports_resource("stream", "movie", "tt123456"));
        assert!(!m.supports_resource("stream", "series", "tt123456"));
        assert!(m.behavior_hints.p2p);
        assert_eq!(m.catalogs.len(), 1);
    }

    #[test]
    fn parses_catalog_metas_fast_and_handles_types() {
        let payload = br#"{
            "metas": [
                {"id": "tt1", "name": "One", "imdbRating": 8.7, "genres": ["Drama", null, "Crime"], "background": "https://img/bg.jpg", "videos": [{"id": "skip"}]},
                {"id": "", "name": "Missing Id"},
                {"id": "tt2", "type": "series", "name": "Two", "imdbRating": "7.5", "released": "2026-11-27"}
            ]
        }"#;
        let all = parse_catalog_metas(payload, "movie", None).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].imdb_rating.as_deref(), Some("8.7"));
        assert_eq!(all[0].genres, vec!["Drama", "Crime"]);
        assert_eq!(all[0].banner.as_deref(), Some("https://img/bg.jpg"));
        assert_eq!(all[1].r#type, "series");
        assert_eq!(all[1].release_info.as_deref(), Some("2026-11-27"));

        let capped = parse_catalog_metas(payload, "movie", Some(1)).unwrap();
        assert_eq!(capped.len(), 1);
    }
}
