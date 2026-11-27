use nuvio_core::plugins::{PluginCallArgs, PluginRuntime, PluginStreamResult};
use nuvio_core::storage::KvStore;
use nuvio_core::supabase::{SupabaseClient, SupabaseSession, SyncedPluginRow};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRepoItem {
    pub manifest_url: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub scraper_count: usize,
    pub last_updated: i64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginScraperItem {
    pub id: String,
    pub repository_url: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub filename: String,
    pub supported_types: Vec<String>,
    pub enabled: bool,
    #[serde(default)]
    pub has_settings: bool,
    pub code: String,
}

fn has_on_settings_hint(code: &str) -> bool {
    code.contains("function onSettings")
        || code.contains(".onSettings")
        || code.contains("onSettings =")
        || code.contains("onSettings=")
        || code.contains("onSettings:")
}

pub fn load_scraper_settings(kv: &KvStore, scraper_id: &str) -> String {
    kv.namespace("plugins")
        .get_string(&format!("settings:{scraper_id}"))
        .ok()
        .flatten()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "{}".to_string())
}

pub fn save_scraper_settings(
    kv: &KvStore,
    scraper_id: &str,
    settings_json: &str,
) -> Result<(), String> {
    let parsed: Value = serde_json::from_str(settings_json).map_err(|e| e.to_string())?;
    let normalized = serde_json::to_string(&parsed).map_err(|e| e.to_string())?;
    kv.namespace("plugins")
        .put_string(&format!("settings:{scraper_id}"), Some(&normalized))
        .map_err(|e| e.to_string())
}

impl PluginScraperItem {
    pub fn supports_type(&self, media_type: &str) -> bool {
        let norm = if media_type == "series" { "tv" } else { media_type };
        self.supported_types.iter().any(|t| {
            let tn = if t == "series" { "tv" } else { t.as_str() };
            tn.eq_ignore_ascii_case(norm)
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginsState {
    pub plugins_enabled: bool,
    pub group_by_repo: bool,
    pub repositories: Vec<PluginRepoItem>,
    pub scrapers: Vec<PluginScraperItem>,
}

impl Default for PluginsState {
    fn default() -> Self {
        Self {
            plugins_enabled: true,
            group_by_repo: false,
            repositories: Vec::new(),
            scrapers: Vec::new(),
        }
    }
}

pub fn load_plugins_state(kv: &KvStore) -> PluginsState {
    let mut state: PluginsState = kv
        .namespace("plugins")
        .get_string("state_json")
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    for s in &mut state.scrapers {
        if !s.has_settings && has_on_settings_hint(&s.code) {
            s.has_settings = true;
        }
    }
    state
}

pub fn save_plugins_state(kv: &KvStore, state: &PluginsState) -> Result<(), String> {
    let json = serde_json::to_string(state).map_err(|e| e.to_string())?;
    kv.namespace("plugins")
        .put_string("state_json", Some(&json))
        .map_err(|e| e.to_string())
}

pub fn normalize_manifest_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Informe a URL do repositório de plugins.".to_string());
    }
    let with_scheme = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    let mut url = Url::parse(&with_scheme).map_err(|e| e.to_string())?;
    if !url.path().ends_with("manifest.json") {
        let p = url.path().trim_end_matches('/');
        let np = if p.is_empty() {
            "/manifest.json".to_string()
        } else {
            format!("{p}/manifest.json")
        };
        url.set_path(&np);
    }
    Ok(url.to_string())
}

fn resolve_script_url(manifest_url: &str, filename: &str) -> Result<String, String> {
    if filename.starts_with("http://") || filename.starts_with("https://") {
        return Ok(filename.to_string());
    }
    let base = Url::parse(manifest_url).map_err(|e| e.to_string())?;
    base.join(filename)
        .map(|u| u.to_string())
        .map_err(|e| e.to_string())
}

pub async fn fetch_plugin_repository(
    manifest_url: &str,
    existing_scrapers: &[PluginScraperItem],
) -> Result<(PluginRepoItem, Vec<PluginScraperItem>), String> {
    let client = nuvio_core::storage::dns::client_builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let body = client
        .get(manifest_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let root: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    let name = root
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Plugin Repository")
        .to_string();
    let version = root
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("1.0.0")
        .to_string();
    let description = root
        .get("description")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let scrapers_arr = root
        .get("scrapers")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut parsed_entries = Vec::new();
    for entry in scrapers_arr {
        let Some(obj) = entry.as_object() else {
            continue;
        };
        let Some(id) = obj.get("id").and_then(|v| v.as_str()).map(str::to_string) else {
            continue;
        };
        let Some(sname) = obj.get("name").and_then(|v| v.as_str()).map(str::to_string) else {
            continue;
        };
        let Some(filename) = obj
            .get("filename")
            .and_then(|v| v.as_str())
            .map(str::to_string)
        else {
            continue;
        };
        let disabled_platforms: Vec<String> = obj
            .get("disabledPlatforms")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_ascii_lowercase()))
                    .collect()
            })
            .unwrap_or_default();
        if disabled_platforms.iter().any(|p| p == "desktop") {
            continue;
        }
        let supported_types: Vec<String> = obj
            .get("supportedTypes")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .filter(|v: &Vec<String>| !v.is_empty())
            .unwrap_or_else(|| vec!["movie".to_string(), "tv".to_string()]);
        let default_enabled = obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
        let manifest_has_settings = obj.get("hasSettings").and_then(|v| v.as_bool()).unwrap_or(false);
        let prev_enabled = existing_scrapers
            .iter()
            .find(|s| s.id == id)
            .map(|s| s.enabled);

        parsed_entries.push((
            id,
            sname,
            obj.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            obj.get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("1.0.0")
                .to_string(),
            filename,
            supported_types,
            prev_enabled.unwrap_or(default_enabled),
            manifest_has_settings,
        ));
    }

    let mut handles = Vec::with_capacity(parsed_entries.len());
    for entry in parsed_entries {
        let client_c = client.clone();
        let man_url = manifest_url.to_string();
        handles.push(tokio::spawn(async move {
            let script_url = resolve_script_url(&man_url, &entry.4).ok()?;
            let code = client_c
                .get(&script_url)
                .send()
                .await
                .ok()?
                .error_for_status()
                .ok()?
                .text()
                .await
                .ok()?;
            let has_settings = entry.7 || has_on_settings_hint(&code);
            Some(PluginScraperItem {
                id: entry.0,
                repository_url: man_url,
                name: entry.1,
                description: entry.2,
                version: entry.3,
                filename: entry.4,
                supported_types: entry.5,
                enabled: entry.6,
                has_settings,
                code,
            })
        }));
    }

    let mut scrapers = Vec::new();
    for h in handles {
        if let Ok(Some(s)) = h.await {
            scrapers.push(s);
        }
    }

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let repo = PluginRepoItem {
        manifest_url: manifest_url.to_string(),
        name,
        version,
        description,
        scraper_count: scrapers.len(),
        last_updated: now_ms,
        error_message: None,
    };

    Ok((repo, scrapers))
}

pub async fn sync_pull_plugins_into_kv(
    kv: &KvStore,
    client: &SupabaseClient,
    session: SupabaseSession,
    effective_profile_id: i32,
) -> PluginsState {
    client.import_session(session);
    let mut state = load_plugins_state(kv);
    let Ok(rows) = client.pull_plugins(effective_profile_id).await else {
        return state;
    };

    let mut next_repos = Vec::new();
    let mut next_scrapers = Vec::new();
    let mut handles = Vec::new();

    for row in rows {
        let Ok(url) = normalize_manifest_url(&row.url) else {
            continue;
        };
        if let Some(existing_repo) = state
            .repositories
            .iter()
            .find(|r| r.manifest_url == url)
            .cloned()
        {
            next_repos.push(existing_repo);
            next_scrapers.extend(
                state
                    .scrapers
                    .iter()
                    .filter(|s| s.repository_url == url)
                    .cloned(),
            );
        } else {
            let prev = state.scrapers.clone();
            handles.push(tokio::spawn(async move {
                (url.clone(), fetch_plugin_repository(&url, &prev).await)
            }));
        }
    }

    for h in handles {
        if let Ok((url, res)) = h.await {
            match res {
                Ok((repo, scrapers)) => {
                    next_repos.push(repo);
                    next_scrapers.extend(scrapers);
                }
                Err(err) => {
                    next_repos.push(PluginRepoItem {
                        manifest_url: url,
                        name: "Repositório".to_string(),
                        version: "-".to_string(),
                        description: None,
                        scraper_count: 0,
                        last_updated: 0,
                        error_message: Some(err),
                    });
                }
            }
        }
    }

    state.repositories = next_repos;
    state.scrapers = next_scrapers;
    let _ = save_plugins_state(kv, &state);
    state
}

pub fn spawn_push_plugins(
    client: Option<SupabaseClient>,
    session: Option<SupabaseSession>,
    profile_id: i32,
    state: &PluginsState,
) {
    let (Some(client), Some(session)) = (client, session) else {
        return;
    };
    let rows: Vec<SyncedPluginRow> = state
        .repositories
        .iter()
        .enumerate()
        .map(|(i, r)| SyncedPluginRow {
            url: r.manifest_url.clone(),
            name: Some(r.name.clone()),
            enabled: true,
            sort_order: i as i32,
        })
        .collect();
    tauri::async_runtime::spawn(async move {
        client.import_session(session);
        let _ = client.push_plugins(profile_id, &rows).await;
    });
}

pub async fn run_scraper_for_stream(
    scraper: &PluginScraperItem,
    tmdb_id: &str,
    media_type: &str,
    season: Option<i32>,
    episode: Option<i32>,
    tmdb_api_key: &str,
    settings_json: &str,
) -> Result<Vec<PluginStreamResult>, String> {
    let norm_type = if media_type == "series" { "tv" } else { media_type };
    PluginRuntime::execute_plugin(
        scraper.code.clone(),
        scraper.id.clone(),
        settings_json.to_string(),
        tmdb_api_key.to_string(),
        PluginCallArgs {
            tmdb_id: tmdb_id.to_string(),
            media_type: norm_type.to_string(),
            season,
            episode,
        },
    )
    .await
    .map_err(|e| e.to_string())
}
