
mod crash_log;
mod discord;
mod metrics;
mod party;
mod services;
mod connected;
mod player;
mod video_options;
mod player_assets;
#[cfg(target_os = "linux")]
mod player_overlay_linux;
mod plugins_repo;
mod skip;
mod simkl;
mod mdblist_lib;
mod updater;

use nuvio_core::integrations::addons::{
    AddonClient, AddonManifest, AddonStream, AddonSubtitle, MetaPreview,
};
use nuvio_core::p2p::ffi::NativeNuvioEngineLibrary;
use nuvio_core::p2p::{find_bundled_engine_library, platform_library_name};
use nuvio_core::plugins::{
    PluginCallArgs, PluginRuntime, PluginStreamResult, prefetch_tmdb_for_plugins,
    resolve_imdb_to_tmdb,
};
use nuvio_core::storage::{AppPaths, CacheStore, KvStore};
use nuvio_core::supabase::{
    DeviceLinkStart, NuvioProfile, OFFICIAL_NUVIO_SERVER_URL, ServerCapabilities, SupabaseClient,
    SupabaseConfig, SupabaseSession, SyncedAddonRow, SyncedWatchedItem, WatchProgressSyncEntry,
    discover_nuvio_server,
};
#[cfg(not(target_os = "android"))]
use player::ViewportRect;
use player::{EmbeddedPlayer, PlayerUiEventPayload, SubtitleStylePayload};
#[cfg(not(target_os = "android"))]
use player_assets::export_player_ui_assets;
use player_assets::{SubtitleCueItem, parse_subtitle_cues};
use plugins_repo::{PluginsState, load_plugins_state, save_plugins_state};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use skip::{SkipChapter, SkipInterval, resolve_skip_intervals, submit_introdb_segment};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

const HOME_CATALOG_FRESH_SECS: u64 = 900;
const HOME_CATALOG_CACHE_TTL_SECS: u64 = 24 * 3600;
const DISCOVER_PAGE_CACHE_TTL_SECS: u64 = 1800;

struct AppState {
    paths: AppPaths,
    kv: KvStore,
    cache: CacheStore,
    addons_http: AddonClient,
    home_catalog_gen: Arc<AtomicU64>,
    addon_search: Mutex<(u64, Option<tokio::task::JoinHandle<()>>)>,
    supabase: Arc<RwLock<Option<SupabaseClient>>>,
    player: EmbeddedPlayer,
    party: Arc<party::PartyRuntime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerConfigState {
    backend_url: String,
    publishable_key: String,
    capabilities: ServerCapabilities,
    is_custom: bool,
    discovery_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuthStatusResponse {
    signed_in: bool,
    is_guest: bool,
    email: Option<String>,
    user_id: Option<String>,
    server: Option<ServerConfigState>,
    profiles: Vec<NuvioProfile>,
    active_profile_index: i32,
}

#[derive(Debug, Serialize)]
struct DiagnosticsResponse {
    os: &'static str,
    arch: &'static str,
    config_dir: String,
    cache_dir: String,
    sqlite_ok: bool,
    redb_ok: bool,
    engine_library_name: &'static str,
    engine_found: bool,
    engine_api_version: Option<u32>,
    engine_backend_version: Option<String>,
    mpv_available: bool,
}

fn default_true_val() -> bool {
    true
}
fn default_nav_style() -> String {
    "sidebar".to_string()
}
fn default_nav_mode() -> String {
    "adaptive".to_string()
}
fn default_cw_style() -> String {
    "card".to_string()
}
fn default_detail_bg_mode() -> String {
    "cinematic".to_string()
}
fn default_streams_bg_mode() -> String {
    "cover".to_string()
}
fn default_dns_provider() -> String {
    "cloudflare".to_string()
}
fn default_pref_audio() -> String {
    "por".to_string()
}
fn default_pref_sub() -> String {
    "pt-br".to_string()
}
fn default_hold_speed() -> f64 {
    2.0
}
fn default_introdb_url() -> String {
    skip::DEFAULT_INTRODB_URL.to_string()
}
fn default_library_source() -> String {
    "default".to_string()
}
fn default_custom_colors() -> Vec<String> {
    vec![
        "#FFFFFF".to_string(),
        "#B8BEC5".to_string(),
        "#6E7178".to_string(),
    ]
}
fn default_subtitle_style() -> SubtitleStylePayload {
    SubtitleStylePayload {
        font_size_sp: 20.0,
        outline_enabled: true,
        bold: false,
        bottom_offset: 24,
        text_color: "#FFFFFFFF".to_string(),
        outline_color: "#FF000000".to_string(),
        background_color: "#00000000".to_string(),
        shadow_enabled: true,
        shadow_opacity: 75.0,
        shadow_size: 2.0,
        font_name: None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DesktopSettings {
    #[serde(default)]
    playback: video_options::PlaybackOptions,
    #[serde(default)]
    services: services::Services,
    #[serde(default)]
    connected: connected::Preferences,
    #[serde(default)]
    seekr: connected::Preview,
    #[serde(default)]
    tmdb_api_key: String,
    #[serde(default)]
    theme: String,
    #[serde(default)]
    amoled_black: bool,
    #[serde(default)]
    mpv_path: String,
    #[serde(default = "default_custom_colors")]
    custom_theme_colors: Vec<String>,
    #[serde(default = "default_nav_style")]
    nav_style: String,
    #[serde(default = "default_nav_mode")]
    nav_mode: String,
    #[serde(default = "default_true_val")]
    show_hero: bool,
    #[serde(default = "default_cw_style")]
    cw_style: String,
    #[serde(default)]
    blur_unwatched_episodes: bool,
    #[serde(default = "default_true_val")]
    hover_preview_enabled: bool,
    #[serde(default = "default_detail_bg_mode")]
    detail_bg_mode: String,
    #[serde(default = "default_streams_bg_mode")]
    streams_bg_mode: String,
    #[serde(default = "default_dns_provider")]
    dns_provider: String,
    #[serde(default)]
    dns_custom: String,
    #[serde(default = "default_true_val")]
    stream_badges_top: bool,
    #[serde(default = "default_true_val")]
    show_addon_logo: bool,
    #[serde(default = "default_true_val")]
    show_file_size_badge: bool,
    #[serde(default = "default_pref_audio")]
    preferred_audio_lang: String,
    #[serde(default = "default_pref_sub")]
    preferred_sub_lang: String,
    #[serde(default = "default_true_val")]
    skip_intro_enabled: bool,
    #[serde(default)]
    auto_skip_intro: bool,
    #[serde(default)]
    auto_skip_outro: bool,
    #[serde(default)]
    auto_skip_recap: bool,
    #[serde(default = "default_true_val")]
    aniskip_enabled: bool,
    #[serde(default = "default_introdb_url")]
    introdb_url: String,
    #[serde(default)]
    introdb_api_key: String,
    #[serde(default = "default_true_val")]
    hold_to_speed_enabled: bool,
    #[serde(default = "default_hold_speed")]
    hold_to_speed_value: f64,
    #[serde(default = "default_true_val")]
    pause_overlay_enabled: bool,
    #[serde(default = "default_true_val")]
    next_episode_enabled: bool,
    #[serde(default)]
    external_player_enabled: bool,
    #[serde(default = "default_library_source")]
    library_source: String,
    #[serde(default = "default_library_source")]
    progress_source: String,
    #[serde(default = "default_subtitle_style")]
    subtitle_style: SubtitleStylePayload,
}

impl Default for DesktopSettings {
    fn default() -> Self {
        Self {
            services: services::Services::default(),
            connected: Default::default(),
            seekr: Default::default(),
            playback: video_options::PlaybackOptions::default(),
            tmdb_api_key: String::new(),
            theme: "white".to_string(),
            amoled_black: false,
            mpv_path: String::new(),
            custom_theme_colors: default_custom_colors(),
            nav_style: default_nav_style(),
            nav_mode: default_nav_mode(),
            show_hero: true,
            cw_style: default_cw_style(),
            blur_unwatched_episodes: false,
            hover_preview_enabled: true,
            detail_bg_mode: default_detail_bg_mode(),
            streams_bg_mode: default_streams_bg_mode(),
            dns_provider: default_dns_provider(),
            dns_custom: String::new(),
            stream_badges_top: true,
            show_addon_logo: true,
            show_file_size_badge: true,
            preferred_audio_lang: default_pref_audio(),
            preferred_sub_lang: default_pref_sub(),
            skip_intro_enabled: true,
            auto_skip_intro: false,
            auto_skip_outro: false,
            auto_skip_recap: false,
            aniskip_enabled: true,
            introdb_url: default_introdb_url(),
            introdb_api_key: String::new(),
            hold_to_speed_enabled: true,
            hold_to_speed_value: 2.0,
            pause_overlay_enabled: true,
            next_episode_enabled: true,
            external_player_enabled: false,
            library_source: default_library_source(),
            progress_source: default_library_source(),
            subtitle_style: default_subtitle_style(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredAddon {
    manifest_url: String,
    manifest: AddonManifest,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CatalogSection {
    order: usize,
    key: String,
    title: String,
    addon_name: String,
    manifest_url: String,
    media_type: String,
    catalog_id: String,
    items: Vec<MetaPreview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedHomeCatalogs {
    signature: String,
    updated_at_secs: u64,
    sections: Vec<CatalogSection>,
}

#[derive(Debug, Clone, Serialize)]
struct AddonStreamGroup {
    order: usize,
    addon_id: String,
    addon_name: String,
    addon_logo: Option<String>,
    streams: Vec<AddonStream>,
}

#[derive(Debug, Serialize)]
struct ProgressiveStartAck {
    total: usize,
}

#[derive(Debug, Serialize)]
struct HomeCatalogStartAck {
    total: usize,
    cached_sections: Vec<CatalogSection>,
    cache_fresh: bool,
}

#[derive(Debug, Clone, Serialize)]
struct SearchSectionEvent {
    seq: u64,
    section: CatalogSection,
}

#[derive(Debug, Clone, Serialize)]
struct SearchDoneEvent {
    seq: u64,
    total: usize,
}

#[derive(Debug, Clone, Serialize)]
struct ContinueWatchingCard {
    content_id: String,
    content_type: String,
    video_id: String,
    season: Option<i32>,
    episode: Option<i32>,
    title: String,
    episode_title: Option<String>,
    artwork: Option<String>,
    poster: Option<String>,
    background: Option<String>,
    logo: Option<String>,
    progress_percent: u8,
    remaining_minutes: Option<i64>,
    position_ms: i64,
    duration_ms: i64,
    last_watched: i64,
}

fn current_supabase(state: &AppState) -> Option<SupabaseClient> {
    state.supabase.read().expect("sb lock").clone()
}

async fn ensure_supabase_client(state: &AppState) -> Result<(SupabaseClient, ServerConfigState), String> {
    let saved_server = load_server_config_inner(&state.kv);
    let server = match saved_server {
        Some(cfg) => cfg,
        None => {
            let doc = discover_nuvio_server(OFFICIAL_NUVIO_SERVER_URL)
                .await
                .map_err(|e| format!("Falha ao conectar em {OFFICIAL_NUVIO_SERVER_URL}: {e}"))?;
            let cfg = ServerConfigState {
                backend_url: doc.backend_url,
                publishable_key: doc.publishable_key,
                capabilities: doc.capabilities,
                is_custom: false,
                discovery_url: Some(format!("{OFFICIAL_NUVIO_SERVER_URL}/.well-known/nuvio")),
            };
            save_server_config_inner(&state.kv, &cfg)?;
            cfg
        }
    };

    if let Some(existing) = current_supabase(state)
        && existing.config().url == server.backend_url.trim_end_matches('/')
        && existing.config().anon_key == server.publishable_key
    {
        if existing.current_session().is_none()
            && let Some(session) = load_session_inner(&state.kv)
        {
            existing.import_session(session);
        }
        let _ = existing.ensure_fresh_session().await;
        return Ok((existing, server));
    }

    let client = SupabaseClient::new(SupabaseConfig::new(
        &server.backend_url,
        &server.publishable_key,
    ))
    .map_err(|e| e.to_string())?;

    let kv_for_session = state.kv.clone();
    client.set_session_listener(move |sess| {
        if let Ok(json) = serde_json::to_string(sess) {
            let _ = kv_for_session
                .namespace("auth")
                .put_string("session_json", Some(&json));
        }
    });

    if let Some(session) = load_session_inner(&state.kv) {
        client.import_session(session);
        let _ = client.ensure_fresh_session().await;
    }

    *state.supabase.write().expect("sb lock") = Some(client.clone());
    Ok((client, server))
}

#[tauri::command]
async fn get_auth_status(state: State<'_, AppState>) -> Result<AuthStatusResponse, String> {
    let is_guest = state
        .kv
        .namespace("auth")
        .get_bool("is_guest")
        .ok()
        .flatten()
        .unwrap_or(false);

    let active_profile_index = state
        .kv
        .namespace("profiles")
        .get_int("active_index")
        .ok()
        .flatten()
        .unwrap_or(1) as i32;

    let cached_profiles = load_profiles_inner(&state.kv);
    let saved_session = load_session_inner(&state.kv);

    if saved_session.is_none() {
        return Ok(AuthStatusResponse {
            signed_in: false,
            is_guest,
            email: None,
            user_id: None,
            server: load_server_config_inner(&state.kv),
            profiles: if is_guest && cached_profiles.is_empty() {
                vec![default_guest_profile()]
            } else {
                cached_profiles
            },
            active_profile_index,
        });
    }

    let (client, server) = match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ensure_supabase_client(&state),
    )
    .await
    {
        Ok(Ok((c, s))) => (Some(c), Some(s)),
        _ => (None, load_server_config_inner(&state.kv)),
    };

    if let Some(sess) = saved_session {
        let active_sess = if let Some(c) = client {
            if let Some(cur) = c.current_session() {
                cur
            } else {
                c.import_session(sess.clone());
                sess
            }
        } else {
            sess
        };
        let user = active_sess.user.clone();
        return Ok(AuthStatusResponse {
            signed_in: true,
            is_guest: false,
            email: user.as_ref().and_then(|u| u.email.clone()),
            user_id: user.as_ref().map(|u| u.id.clone()),
            server,
            profiles: cached_profiles,
            active_profile_index,
        });
    }

    Ok(AuthStatusResponse {
        signed_in: false,
        is_guest,
        email: None,
        user_id: None,
        server,
        profiles: if is_guest && cached_profiles.is_empty() {
            vec![default_guest_profile()]
        } else {
            cached_profiles
        },
        active_profile_index,
    })
}

#[tauri::command]
async fn sign_in_email(
    state: State<'_, AppState>,
    email: String,
    password: String,
) -> Result<AuthStatusResponse, String> {
    let (client, _) = ensure_supabase_client(&state).await?;
    let session = client
        .sign_in_email(email.trim(), &password)
        .await
        .map_err(format_supabase_err)?;
    complete_auth_session(&state, &client, session).await
}

#[tauri::command]
async fn sign_up_email(
    state: State<'_, AppState>,
    email: String,
    password: String,
) -> Result<AuthStatusResponse, String> {
    let (client, _) = ensure_supabase_client(&state).await?;
    let session = client
        .sign_up_email(email.trim(), &password)
        .await
        .map_err(format_supabase_err)?;
    complete_auth_session(&state, &client, session).await
}

#[tauri::command]
async fn continue_as_guest(state: State<'_, AppState>) -> Result<AuthStatusResponse, String> {
    let auth_ns = state.kv.namespace("auth");
    auth_ns
        .put_bool("is_guest", true)
        .map_err(|e| e.to_string())?;
    auth_ns.remove("session_json").map_err(|e| e.to_string())?;

    let mut profiles = load_profiles_inner(&state.kv);
    if profiles.is_empty() {
        profiles.push(default_guest_profile());
        save_profiles_inner(&state.kv, &profiles)?;
    }
    state
        .kv
        .namespace("profiles")
        .put_int("active_index", 1)
        .map_err(|e| e.to_string())?;

    get_auth_status(state).await
}

#[tauri::command]
async fn sign_out(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(client) = current_supabase(&state) {
        let _ = client.sign_out().await;
    }
    let auth_ns = state.kv.namespace("auth");
    auth_ns.remove("session_json").map_err(|e| e.to_string())?;
    auth_ns.put_bool("is_guest", false).map_err(|e| e.to_string())?;
    state
        .kv
        .namespace("profiles")
        .remove("list_json")
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Serialize)]
struct DeviceLinkSessionInfo {
    device_code: String,
    user_code: String,
    verification_uri_complete: String,
    poll_interval_seconds: u64,
    nonce: String,
}

#[tauri::command]
async fn start_device_link(state: State<'_, AppState>) -> Result<DeviceLinkSessionInfo, String> {
    let (client, server) = ensure_supabase_client(&state).await?;
    let mut nonce_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = nonce_bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();

    let redirect_base = if server.is_custom {
        server.backend_url.clone()
    } else {
        "https://nuvio.tv/link".to_string()
    };

    let device_name = format!("Nuvio Desktop ({})", std::env::consts::OS);
    let started: DeviceLinkStart = client
        .start_device_login_session(&nonce, &redirect_base, &device_name)
        .await
        .map_err(format_supabase_err)?
        .ok_or_else(|| "Falha ao criar código de login.".to_string())?;

    Ok(DeviceLinkSessionInfo {
        device_code: started.device_code,
        user_code: started.user_code,
        verification_uri_complete: started.verification_uri_complete,
        poll_interval_seconds: started.poll_interval_seconds,
        nonce,
    })
}

#[derive(Debug, Serialize)]
struct DeviceLinkPollResult {
    status: String,
    auth: Option<AuthStatusResponse>,
}

#[tauri::command]
async fn poll_device_link(
    state: State<'_, AppState>,
    device_code: String,
    nonce: String,
) -> Result<DeviceLinkPollResult, String> {
    let (client, _) = ensure_supabase_client(&state).await?;
    let poll = client
        .poll_tv_login_session(&device_code, &nonce)
        .await
        .map_err(format_supabase_err)?;

    let status_str = poll
        .map(|p| p.status.trim().to_ascii_lowercase())
        .unwrap_or_else(|| "pending".to_string());

    if status_str == "approved" {
        let session = client
            .exchange_tv_login(&device_code, &nonce)
            .await
            .map_err(format_supabase_err)?;
        let auth = complete_auth_session(&state, &client, session).await?;
        return Ok(DeviceLinkPollResult {
            status: "approved".to_string(),
            auth: Some(auth),
        });
    }

    Ok(DeviceLinkPollResult {
        status: status_str,
        auth: None,
    })
}

#[tauri::command]
async fn configure_server(
    state: State<'_, AppState>,
    server_url: Option<String>,
) -> Result<ServerConfigState, String> {
    let target = server_url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(OFFICIAL_NUVIO_SERVER_URL);

    let is_custom = !target.eq_ignore_ascii_case(OFFICIAL_NUVIO_SERVER_URL);
    let doc = discover_nuvio_server(target)
        .await
        .map_err(|e| format!("Falha na descoberta do servidor ({target}): {e}"))?;

    let cfg = ServerConfigState {
        backend_url: doc.backend_url,
        publishable_key: doc.publishable_key,
        capabilities: doc.capabilities,
        is_custom,
        discovery_url: Some(target.to_string()),
    };
    save_server_config_inner(&state.kv, &cfg)?;
    *state.supabase.write().expect("sb lock") = None;
    Ok(cfg)
}

#[tauri::command]
async fn select_profile(
    state: State<'_, AppState>,
    profile_index: i32,
) -> Result<Vec<StoredAddon>, String> {
    state
        .kv
        .namespace("profiles")
        .put_int("active_index", i64::from(profile_index))
        .map_err(|e| e.to_string())?;

    let effective_profile_id = resolve_effective_profile_id(&state.kv, profile_index);

    if let (Some(client), Some(session)) = (
        current_supabase(&state),
        load_session_inner(&state.kv),
    ) {
        client.import_session(session.clone());
        let plugin_pid = resolve_effective_plugins_profile_id(&state.kv, profile_index);
        let plugins_res = plugins_repo::sync_pull_plugins_into_kv(
            &state.kv,
            &client,
            session.clone(),
            plugin_pid,
        );
        let addons_res = client.pull_addons(effective_profile_id);
        let (rows_res, _) = tokio::join!(addons_res, plugins_res);

        if let Ok(mut rows) = rows_res {
            rows.sort_by_key(|r| r.sort_order);
            let mut results: Vec<Option<StoredAddon>> = vec![None; rows.len()];
            let mut missing: Vec<(usize, String, bool)> = Vec::new();

            for (idx, row) in rows.iter().enumerate() {
                let normalized_url = ensure_manifest_suffix(&row.url);
                if let Some(manifest) = load_cached_manifest(&state.kv, &normalized_url) {
                    results[idx] = Some(StoredAddon {
                        manifest_url: normalized_url,
                        manifest,
                        enabled: row.enabled,
                    });
                } else {
                    missing.push((idx, normalized_url, row.enabled));
                }
            }

            if !missing.is_empty() {
                let shared_http = state.addons_http.clone();
                let mut handles = Vec::with_capacity(missing.len());
                for (idx, url, enabled) in missing {
                    let http = shared_http.clone();
                    handles.push(tokio::spawn(async move {
                        let fetched = http.fetch_manifest(&url).await.ok();
                        (idx, url, enabled, fetched)
                    }));
                }
                for h in handles {
                    if let Ok((idx, url, enabled, Some(manifest))) = h.await {
                        save_cached_manifest(&state.kv, &url, &manifest);
                        results[idx] = Some(StoredAddon {
                            manifest_url: url,
                            manifest,
                            enabled,
                        });
                    }
                }
            }

            let synced: Vec<StoredAddon> = results.into_iter().flatten().collect();
            if !synced.is_empty() {
                save_addons_inner(&state.kv, &synced)?;
                return Ok(synced);
            }
        }
    }

    load_addons_inner(&state.kv)
}

#[tauri::command]
async fn pull_account_library(state: State<'_, AppState>) -> Result<Vec<Value>, String> {
    let active_profile_index = state
        .kv
        .namespace("profiles")
        .get_int("active_index")
        .ok()
        .flatten()
        .unwrap_or(1) as i32;

    if let (Some(client), Some(session)) = (
        current_supabase(&state),
        load_session_inner(&state.kv),
    ) {
        client.import_session(session);
        if let Ok(items) = client.pull_library(active_profile_index, 150, 0).await {
            let _ = save_local_library_inner(&state.kv, &items);
            return Ok(items);
        }
    }
    Ok(load_local_library_inner(&state.kv))
}

#[tauri::command]
async fn pull_account_collections(state: State<'_, AppState>) -> Result<Vec<Value>, String> {
    let Some(session) = load_session_inner(&state.kv) else {
        return Ok(Vec::new());
    };
    let client = current_supabase(&state).ok_or("Servidor da conta indisponível.")?;
    client.import_session(session);
    let rows: Vec<Value> = client
        .rpc(
            "sync_pull_collections",
            &serde_json::json!({ "p_profile_id": active_profile_idx(&state.kv) }),
        )
        .await
        .map_err(|e| e.to_string())?;

    match rows.first().and_then(|row| row.get("collections_json")) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(collections)) => Ok(collections.clone()),
        _ => Err("O servidor retornou coleções em um formato inválido.".into()),
    }
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn home_catalog_cache_key(profile_idx: i32) -> String {
    format!("home_catalogs:p{profile_idx}")
}

fn load_cached_home_catalogs(cache: &CacheStore, key: &str) -> Option<CachedHomeCatalogs> {
    let raw = cache.get(key).ok().flatten()?;
    serde_json::from_slice(&raw).ok()
}

fn save_cached_home_catalogs(cache: &CacheStore, key: &str, entry: &CachedHomeCatalogs) {
    if let Ok(bytes) = serde_json::to_vec(entry) {
        let _ = cache.set(key, &bytes, Some(HOME_CATALOG_CACHE_TTL_SECS));
    }
}

fn catalog_items_changed(prev: Option<&CatalogSection>, next_items: &[MetaPreview]) -> bool {
    let Some(prev) = prev else {
        return true;
    };
    prev.items != next_items
}

/// Progressively fetches catalogs across enabled addons and emits `home:section`
/// as soon as each catalog responds (matching `HomeRepository.kt` incremental publish).
/// Uses stale-while-revalidate disk cache so startup renders instantly.
#[tauri::command]
async fn start_home_catalogs(
    app: AppHandle,
    state: State<'_, AppState>,
    force: Option<bool>,
) -> Result<HomeCatalogStartAck, String> {
    let setup_started = std::time::Instant::now();
    let force = force.unwrap_or(false);
    let profile_idx = active_profile_idx(&state.kv);
    let addons = load_addons_inner(&state.kv)?;
    let enabled_addons = addons.iter().filter(|a| a.enabled).count();
    let mut requests = Vec::new();
    for addon in addons.into_iter().filter(|a| a.enabled) {
        for cat in &addon.manifest.catalogs {
            let search_only = cat.extra.iter().any(|e| e.name == "search" && e.is_required);
            if search_only {
                continue;
            }
            requests.push((
                requests.len(),
                addon.manifest.clone(),
                cat.r#type.clone(),
                cat.id.clone(),
                cat.name.clone(),
            ));
            if requests.len() >= 18 {
                break;
            }
        }
        if requests.len() >= 18 {
            break;
        }
    }

    let total = requests.len();
    let batch_gen = state.home_catalog_gen.fetch_add(1, Ordering::SeqCst) + 1;
    let signature = requests
        .iter()
        .map(|(_, m, t, id, _)| format!("{}|{t}|{id}", m.transport_url))
        .collect::<Vec<_>>()
        .join(";");
    let cache_key = home_catalog_cache_key(profile_idx);
    let cached_entry = if total > 0 {
        load_cached_home_catalogs(&state.cache, &cache_key).filter(|c| c.signature == signature && !c.sections.is_empty())
    } else {
        None
    };

    let mut cached_by_order: HashMap<usize, CatalogSection> = HashMap::new();
    let mut initial_cached_sections: Vec<CatalogSection> = Vec::new();
    if let Some(ref cached) = cached_entry {
        for sec in &cached.sections {
            cached_by_order.insert(sec.order, sec.clone());
        }
        initial_cached_sections = cached.sections.clone();
        let age_secs = now_unix_secs().saturating_sub(cached.updated_at_secs);
        crash_log::note(&format!(
            "home: cache hit ({} secoes, idade {}s) em {} ms",
            cached.sections.len(),
            age_secs,
            setup_started.elapsed().as_millis()
        ));
        if !force && age_secs < HOME_CATALOG_FRESH_SECS {
            let _ = app.emit("home:done", total);
            return Ok(HomeCatalogStartAck {
                total,
                cached_sections: initial_cached_sections,
                cache_fresh: true,
            });
        }
    } else {
        crash_log::note(&format!(
            "home: {total} catalogos de {enabled_addons} addons preparados em {} ms",
            setup_started.elapsed().as_millis()
        ));
    }

    if total == 0 {
        let _ = app.emit("home:done", 0usize);
        return Ok(HomeCatalogStartAck {
            total: 0,
            cached_sections: Vec::new(),
            cache_fresh: true,
        });
    }

    let shared_http = state.addons_http.clone();
    let cache_store = state.cache.clone();
    let gen_counter = Arc::clone(&state.home_catalog_gen);
    let cached_by_order = Arc::new(cached_by_order);
    tokio::spawn(async move {
        let batch_started = std::time::Instant::now();
        let mut handles = Vec::with_capacity(total);
        for (order, manifest, media_type, catalog_id, title) in requests {
            let app_clone = app.clone();
            let http = shared_http.clone();
            let cache_for_task = cache_store.clone();
            let gen_ref = Arc::clone(&gen_counter);
            let cached_map = Arc::clone(&cached_by_order);
            handles.push(tokio::spawn(async move {
                let started = std::time::Instant::now();
                let fetched = http
                    .fetch_catalog(&manifest, &media_type, &catalog_id, None, None)
                    .await;
                let elapsed = started.elapsed().as_millis();
                if gen_ref.load(Ordering::SeqCst) != batch_gen {
                    return None;
                }
                match fetched {
                    Ok(mut full_items) if !full_items.is_empty() => {
                        // Pre-warm Discover / Full Catalog page 0 cache using the full response
                        // we already downloaded instead of discarding items beyond 20.
                        let page_cache_key = format!(
                            "catalog_page:{}:{media_type}:{catalog_id}:",
                            manifest.transport_url
                        );
                        if let Ok(bytes) = serde_json::to_vec(&full_items) {
                            let _ = cache_for_task.set(
                                &page_cache_key,
                                &bytes,
                                Some(DISCOVER_PAGE_CACHE_TTL_SECS),
                            );
                        }
                        if full_items.len() > 20 {
                            full_items.truncate(20);
                        }
                        let items = full_items;
                        let prev = cached_map.get(&order);
                        let changed = catalog_items_changed(prev, &items);
                        let sec = CatalogSection {
                            order,
                            key: format!("{}:{media_type}:{catalog_id}", manifest.id),
                            title: title.clone(),
                            addon_name: manifest.name.clone(),
                            manifest_url: manifest.transport_url.clone(),
                            media_type,
                            catalog_id,
                            items,
                        };
                        crash_log::note(&format!(
                            "home: \"{}\" [{}] {} itens em {elapsed} ms{}",
                            title,
                            manifest.name,
                            sec.items.len(),
                            if changed { "" } else { " (sem alteracoes)" }
                        ));
                        if changed {
                            let _ = app_clone.emit("home:section", &sec);
                        }
                        Some(sec)
                    }
                    Ok(_) => {
                        crash_log::note(&format!(
                            "home: \"{}\" [{}] vazio em {elapsed} ms",
                            title, manifest.name
                        ));
                        None
                    }
                    Err(err) => {
                        crash_log::note(&format!(
                            "home: \"{}\" [{}] falhou em {elapsed} ms: {err}",
                            title, manifest.name
                        ));
                        cached_map.get(&order).cloned()
                    }
                }
            }));
        }
        let mut collected = Vec::new();
        for h in handles {
            if let Ok(Some(sec)) = h.await {
                collected.push(sec);
            }
        }
        if gen_counter.load(Ordering::SeqCst) != batch_gen {
            return;
        }
        collected.sort_by_key(|s| s.order);
        if !collected.is_empty() {
            save_cached_home_catalogs(
                &cache_store,
                &cache_key,
                &CachedHomeCatalogs {
                    signature,
                    updated_at_secs: now_unix_secs(),
                    sections: collected,
                },
            );
        }
        crash_log::note(&format!(
            "home: lote concluido em {} ms ({total} catalogos)",
            batch_started.elapsed().as_millis()
        ));
        let _ = app.emit("home:done", total);
    });

    Ok(HomeCatalogStartAck {
        total,
        cached_sections: initial_cached_sections,
        cache_fresh: false,
    })
}

/// Pulls `sync_pull_watch_progress` from Supabase and emits Continue Watching items
/// progressively as addon `/meta` resolves titles, artwork, and episode thumbnails.
#[tauri::command]
async fn start_continue_watching(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ProgressiveStartAck, String> {
    let active_profile_index = state
        .kv
        .namespace("profiles")
        .get_int("active_index")
        .ok()
        .flatten()
        .unwrap_or(1) as i32;

    let local_entries: Vec<WatchProgressSyncEntry> = load_local_progress_inner(&state.kv)
        .into_values()
        .collect();

    let mut entries: Vec<WatchProgressSyncEntry> = Vec::new();
    let mut fetched_cloud = false;
    if let Some(session) = load_session_inner(&state.kv)
        && let Ok((client, _)) = ensure_supabase_client(&state).await
    {
        client.import_session(session);
        match client.pull_watch_progress(active_profile_index, 60).await {
            Ok(cloud) => {
                entries = cloud;
                fetched_cloud = true;
            }
            Err(err) => tracing::warn!(
                "Continue watching: falha ao buscar progresso na nuvem ({err}); usando o progresso local"
            ),
        }
    }

    // Sem sessão, offline ou com a nuvem indisponível, o progresso local mantém
    // "Continuar Assistindo" funcionando em vez de esconder a fileira.
    if !fetched_cloud {
        entries = local_entries;
    } else if load_profiles_inner(&state.kv).len() <= 1 {
        // Conta com um único perfil: preserva o que foi assistido sem rede e
        // ainda não subiu para a nuvem.
        let mut seen: HashSet<String> = entries
            .iter()
            .map(|e| e.progress_key.clone())
            .collect();
        for entry in local_entries {
            if seen.insert(entry.progress_key.clone()) {
                entries.push(entry);
            }
        }
    }

    entries.sort_by_key(|a| std::cmp::Reverse(a.last_watched));

    // Keep most recent incomplete entry per content_id (matching HomeContinueWatchingSection)
    let mut deduped = Vec::new();
    let mut seen_content = std::collections::HashSet::new();
    for e in entries {
        if e.position <= 0 {
            continue;
        }
        if e.duration > 0 && (e.position as f64 / e.duration as f64) >= 0.95 {
            continue;
        }
        if seen_content.insert(e.content_id.clone()) {
            deduped.push(e);
            if deduped.len() >= 20 {
                break;
            }
        }
    }

    let total = deduped.len();
    if total == 0 {
        return Ok(ProgressiveStartAck { total: 0 });
    }

    let meta_addons: Vec<AddonManifest> = load_addons_inner(&state.kv)?
        .into_iter()
        .filter(|a| a.enabled)
        .map(|a| a.manifest)
        .collect();

    let shared_http = state.addons_http.clone();
    tokio::spawn(async move {
        let mut handles = Vec::with_capacity(total);
        for (order, e) in deduped.into_iter().enumerate() {
            let app_clone = app.clone();
            let addons_clone = meta_addons.clone();
            let http = shared_http.clone();
            handles.push(tokio::spawn(async move {
                let pct = if e.duration > 0 {
                    ((e.position as f64 / e.duration as f64) * 100.0)
                        .round()
                        .clamp(1.0, 99.0) as u8
                } else {
                    5
                };
                let remaining_minutes = if e.duration > e.position {
                    Some(((e.duration - e.position) / 60_000).max(1))
                } else {
                    None
                };

                let mut card = ContinueWatchingCard {
                    content_id: e.content_id.clone(),
                    content_type: e.content_type.clone(),
                    video_id: e.video_id.clone(),
                    season: e.season,
                    episode: e.episode,
                    title: e.content_id.clone(),
                    episode_title: None,
                    artwork: None,
                    poster: None,
                    background: None,
                    logo: None,
                    progress_percent: pct,
                    remaining_minutes,
                    position_ms: e.position,
                    duration_ms: e.duration,
                    last_watched: e.last_watched,
                };

                for manifest in addons_clone
                    .iter()
                    .filter(|m| m.supports_resource("meta", &e.content_type, &e.content_id))
                {
                    if let Ok(meta) = http
                        .fetch_meta(manifest, &e.content_type, &e.content_id)
                        .await
                    {
                            if let Some(name) = meta.get("name").and_then(|v| v.as_str()) {
                                card.title = name.to_string();
                            }
                            card.poster = meta
                                .get("poster")
                                .and_then(|v| v.as_str())
                                .map(str::to_string);
                            card.background = meta
                                .get("background")
                                .and_then(|v| v.as_str())
                                .map(str::to_string);
                            card.logo = meta
                                .get("logo")
                                .and_then(|v| v.as_str())
                                .map(str::to_string);

                            let mut ep_thumb = None;
                            if let Some(videos) = meta.get("videos").and_then(|v| v.as_array()) {
                                let matched = videos.iter().find(|v| {
                                    let vid_match = v.get("id").and_then(|x| x.as_str())
                                        == Some(e.video_id.as_str());
                                    let se_match = e.season.is_some()
                                        && e.episode.is_some()
                                        && v.get("season").and_then(|x| x.as_i64())
                                            == e.season.map(i64::from)
                                        && (v.get("episode").and_then(|x| x.as_i64())
                                            == e.episode.map(i64::from)
                                            || v.get("number").and_then(|x| x.as_i64())
                                                == e.episode.map(i64::from));
                                    vid_match || se_match
                                });
                                if let Some(ep) = matched {
                                    card.episode_title = ep
                                        .get("title")
                                        .or_else(|| ep.get("name"))
                                        .and_then(|x| x.as_str())
                                        .map(str::to_string);
                                    ep_thumb = ep
                                        .get("thumbnail")
                                        .and_then(|x| x.as_str())
                                        .map(str::to_string);
                                }
                            }
                        card.artwork = ep_thumb
                            .or_else(|| card.background.clone())
                            .or_else(|| card.poster.clone());
                        break;
                    }
                }

                let _ = app_clone.emit(
                    "home:continue-watching-item",
                    serde_json::json!({ "order": order, "card": card }),
                );
            }));
        }
        for h in handles {
            let _ = h.await;
        }
    });

    Ok(ProgressiveStartAck { total })
}

#[tauri::command]
async fn load_catalog_page(
    state: State<'_, AppState>,
    manifest_url: String,
    media_type: String,
    catalog_id: String,
    skip: Option<u32>,
    genre: Option<String>,
    search: Option<String>,
) -> Result<Vec<MetaPreview>, String> {
    let addons = load_addons_inner(&state.kv)?;
    let manifest = addons
        .into_iter()
        .find(|a| a.manifest_url == manifest_url)
        .map(|a| a.manifest);
    let manifest = match manifest {
        Some(m) => m,
        None => state
            .addons_http
            .fetch_manifest(&manifest_url)
            .await
            .map_err(|e| e.to_string())?,
    };
    let mut parts = Vec::new();
    if let Some(q) = search.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        parts.push(format!("search={}", url_encode_segment(q)));
    }
    if let Some(g) = genre.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        parts.push(format!("genre={}", url_encode_segment(g)));
    }
    if let Some(s) = skip.filter(|&s| s > 0) {
        parts.push(format!("skip={s}"));
    }
    let extra = (!parts.is_empty()).then(|| parts.join("&"));
    let can_cache = search.as_deref().map(str::trim).is_none_or(str::is_empty);
    let page_cache_key = format!(
        "catalog_page:{}:{media_type}:{catalog_id}:{}",
        manifest.transport_url,
        extra.as_deref().unwrap_or("")
    );
    if can_cache
        && let Some(raw) = state.cache.get(&page_cache_key).ok().flatten()
        && let Ok(cached_items) = serde_json::from_slice::<Vec<MetaPreview>>(&raw)
    {
        return Ok(cached_items);
    }

    let items = state
        .addons_http
        .fetch_catalog(&manifest, &media_type, &catalog_id, extra.as_deref(), None)
        .await
        .map_err(|e| e.to_string())?;
    if can_cache
        && !items.is_empty()
        && let Ok(bytes) = serde_json::to_vec(&items)
    {
        let _ = state
            .cache
            .set(&page_cache_key, &bytes, Some(DISCOVER_PAGE_CACHE_TTL_SECS));
    }
    Ok(items)
}

#[tauri::command]
async fn start_addon_search(
    app: AppHandle,
    state: State<'_, AppState>,
    query: String,
    seq: u64,
) -> Result<ProgressiveStartAck, String> {
    const MAX_CONCURRENT_CATALOGS: usize = 4;
    let mut active = state.addon_search.lock().map_err(|e| e.to_string())?;
    if seq < active.0 {
        return Ok(ProgressiveStartAck { total: 0 });
    }
    active.0 = seq;
    if let Some(previous) = active.1.take() {
        previous.abort();
    }
    let query = query.trim();
    if query.is_empty() {
        return Ok(ProgressiveStartAck { total: 0 });
    }

    let extra = Arc::new(format!("search={}", url_encode_segment(query)));
    let addons = load_addons_inner(&state.kv)?;
    let mut targets = Vec::new();
    for addon in addons.into_iter().filter(|addon| addon.enabled) {
        let addon = Arc::new(addon);
        for catalog in addon.manifest.catalogs.iter().filter(|cat| cat.supports_search()) {
            targets.push((addon.clone(), catalog.clone()));
        }
    }

    let total = targets.len();
    let http = state.addons_http.clone();
    active.1 = Some(tokio::spawn(async move {
        let mut pending = tokio::task::JoinSet::new();
        for (order, (addon, cat)) in targets.into_iter().enumerate() {
            if pending.len() >= MAX_CONCURRENT_CATALOGS {
                pending.join_next().await;
            }
            let app = app.clone();
            let http = http.clone();
            let extra = extra.clone();
            pending.spawn(async move {
                let Ok(items) = http
                    .fetch_catalog(&addon.manifest, &cat.r#type, &cat.id, Some(&extra), None)
                    .await
                else {
                    return;
                };
                if items.is_empty() {
                    return;
                }
                let type_label = match cat.r#type.as_str() {
                    "movie" => "Filmes",
                    "series" => "Séries",
                    other => other,
                };
                let section = CatalogSection {
                    order,
                    key: format!("{}:search:{}:{}", addon.manifest.id, cat.r#type, cat.id),
                    title: format!("{} - {}", cat.name, type_label),
                    addon_name: addon.manifest.name.clone(),
                    manifest_url: addon.manifest_url.clone(),
                    media_type: cat.r#type,
                    catalog_id: cat.id,
                    items,
                };
                let _ = app.emit("search:section", SearchSectionEvent { seq, section });
            });
        }
        while pending.join_next().await.is_some() {}
        let _ = app.emit("search:done", SearchDoneEvent { seq, total });
    }));

    Ok(ProgressiveStartAck { total })
}

#[tauri::command]
async fn fetch_meta_details(
    state: State<'_, AppState>,
    media_type: String,
    id: String,
) -> Result<Value, String> {
    let addons = load_addons_inner(&state.kv)?;
    let settings = load_settings_inner(&state.kv);
    let meta_addons_sig: Vec<&str> = addons
        .iter()
        .filter(|a| a.enabled && a.manifest.supports_resource("meta", &media_type, &id))
        .map(|a| a.manifest.id.as_str())
        .collect();
    let cache_key = format!(
        "meta_details_v1:{media_type}:{id}:{}:{}:{}:{}:{}",
        meta_addons_sig.join(","),
        settings.services.tmdb_enabled,
        settings.services.tmdb_collections,
        settings.services.mdblist_enabled,
        !settings.tmdb_api_key.trim().is_empty(),
    );
    if let Ok(Some(bytes)) = state.cache.get(&cache_key)
        && let Ok(cached) = serde_json::from_slice::<Value>(&bytes)
        && cached.as_object().is_some_and(|m| !m.is_empty())
    {
        return Ok(cached);
    }

    let mut details = serde_json::json!({});
    for addon in addons.into_iter().filter(|a| a.enabled) {
        if !addon.manifest.supports_resource("meta", &media_type, &id) {
            continue;
        }
        if let Ok(meta) = state.addons_http.fetch_meta(&addon.manifest, &media_type, &id).await
            && meta.is_object()
        {
            details = meta;
            break;
        }
    }
    settings.services.enrich(&mut details, &media_type, &id, &settings.tmdb_api_key).await;
    if details.as_object().is_none_or(|m| m.is_empty()) {
        return Err("Nenhum serviço retornou metadados para este título.".into());
    }
    if let Ok(bytes) = serde_json::to_vec(&details) {
        let _ = state.cache.set(&cache_key, &bytes, Some(300));
    }
    Ok(details)
}

#[tauri::command]
async fn runtime_metrics(app: AppHandle) -> metrics::RuntimeMetrics {
    let player = {
        let state = app.state::<AppState>();
        state.player.metrics()
    };
    let mut result = tokio::task::spawn_blocking(|| metrics::collect(500))
        .await
        .unwrap_or_else(|_| metrics::collect(0));
    result.player = player;
    result
}

#[tauri::command]
fn get_diagnostics(state: State<'_, AppState>) -> DiagnosticsResponse {
    let sqlite_ok = state
        .kv
        .namespace("diagnostics")
        .put_string("last_ui", Some("tauri-v2"))
        .is_ok();
    let redb_ok = state.cache.set("ui_probe", b"ok", Some(60)).is_ok();

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let lib_path = find_bundled_engine_library(&cwd);
    let mut engine_api_version = None;
    let mut engine_backend_version = None;
    if let Some(ref p) = lib_path
        && let Ok(lib) = unsafe { NativeNuvioEngineLibrary::load(p) }
    {
        engine_api_version = lib.api_version().ok();
        engine_backend_version = lib.protocol_backend_version().ok();
    }

    #[cfg(target_os = "android")]
    let mpv_available = false;
    #[cfg(not(target_os = "android"))]
    let mpv_available = {
        let mpv_bin = resolve_mpv_binary(&load_settings_inner(&state.kv).mpv_path);
        Command::new(&mpv_bin)
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success())
    };

    DiagnosticsResponse {
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        config_dir: state.paths.config_dir.display().to_string(),
        cache_dir: state.paths.cache_dir.display().to_string(),
        sqlite_ok,
        redb_ok,
        engine_library_name: platform_library_name(),
        engine_found: lib_path.is_some(),
        engine_api_version,
        engine_backend_version,
        mpv_available,
    }
}

/// Tempos de arranque reportados pela UI, gravados no crash.log.
#[tauri::command]
fn frontend_log(message: String) {
    crash_log::note(&format!("ui: {message}"));
}

#[tauri::command]
fn load_settings(state: State<'_, AppState>) -> DesktopSettings {
    load_settings_inner(&state.kv)
}

#[tauri::command]
fn save_settings(state: State<'_, AppState>, settings: DesktopSettings) -> Result<(), String> {
    nuvio_core::storage::dns::configure(&settings.dns_provider, &settings.dns_custom);
    let ns = state.kv.namespace("desktop_settings");
    let json = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    ns.put_string("config_json", Some(&json))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_addons(state: State<'_, AppState>) -> Result<Vec<StoredAddon>, String> {
    load_addons_inner(&state.kv)
}

#[tauri::command]
async fn add_addon(state: State<'_, AppState>, manifest_url: String) -> Result<StoredAddon, String> {
    let clean_url = ensure_manifest_suffix(manifest_url.trim());
    if clean_url.is_empty() {
        return Err("URL do manifest vazia.".to_string());
    }
    let manifest = state
        .addons_http
        .fetch_manifest(&clean_url)
        .await
        .map_err(|e| e.to_string())?;
    save_cached_manifest(&state.kv, &clean_url, &manifest);

    let mut list = load_addons_inner(&state.kv)?;
    list.retain(|item| item.manifest_url != clean_url);
    let entry = StoredAddon {
        manifest_url: clean_url,
        manifest,
        enabled: true,
    };
    list.push(entry.clone());
    save_addons_inner(&state.kv, &list)?;
    spawn_cloud_addons_sync(&state, &list);
    Ok(entry)
}

#[tauri::command]
fn remove_addon(state: State<'_, AppState>, manifest_url: String) -> Result<Vec<StoredAddon>, String> {
    let mut list = load_addons_inner(&state.kv)?;
    list.retain(|item| item.manifest_url != manifest_url);
    save_addons_inner(&state.kv, &list)?;
    spawn_cloud_addons_sync(&state, &list);
    Ok(list)
}

#[tauri::command]
fn move_addon(
    state: State<'_, AppState>,
    manifest_url: String,
    delta: i32,
) -> Result<Vec<StoredAddon>, String> {
    let mut list = load_addons_inner(&state.kv)?;
    if let Some(pos) = list.iter().position(|item| item.manifest_url == manifest_url) {
        let target = (pos as isize + delta as isize).clamp(0, list.len().saturating_sub(1) as isize)
            as usize;
        if target != pos {
            list.swap(pos, target);
            save_addons_inner(&state.kv, &list)?;
            spawn_cloud_addons_sync(&state, &list);
        }
    }
    Ok(list)
}

#[tauri::command]
fn toggle_addon(
    state: State<'_, AppState>,
    manifest_url: String,
    enabled: bool,
) -> Result<Vec<StoredAddon>, String> {
    let mut list = load_addons_inner(&state.kv)?;
    if let Some(item) = list.iter_mut().find(|item| item.manifest_url == manifest_url) {
        item.enabled = enabled;
        save_addons_inner(&state.kv, &list)?;
        spawn_cloud_addons_sync(&state, &list);
    }
    Ok(list)
}

#[tauri::command]
async fn refresh_addon(
    state: State<'_, AppState>,
    manifest_url: String,
) -> Result<Vec<StoredAddon>, String> {
    let clean_url = ensure_manifest_suffix(manifest_url.trim());
    let manifest = state
        .addons_http
        .fetch_manifest(&clean_url)
        .await
        .map_err(|e| e.to_string())?;
    save_cached_manifest(&state.kv, &clean_url, &manifest);

    let mut list = load_addons_inner(&state.kv)?;
    if let Some(item) = list
        .iter_mut()
        .find(|item| item.manifest_url == manifest_url || item.manifest_url == clean_url)
    {
        item.manifest_url = clean_url;
        item.manifest = manifest;
        save_addons_inner(&state.kv, &list)?;
        spawn_cloud_addons_sync(&state, &list);
    }
    Ok(list)
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
        return Err("URL inválida.".to_string());
    }
    #[cfg(target_os = "linux")]
    let spawn_res = Command::new("xdg-open").arg(trimmed).spawn();
    #[cfg(target_os = "windows")]
    let spawn_res = Command::new("rundll32.exe").args(["url.dll,FileProtocolHandler", trimmed]).spawn();
    #[cfg(target_os = "macos")]
    let spawn_res = Command::new("open").arg(trimmed).spawn();
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    let spawn_res: std::io::Result<std::process::Child> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Abertura externa tratada pelo navegador nesta plataforma.",
    ));
    spawn_res.map(|_| ()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_app_update() -> Result<updater::AppUpdateInfo, String> {
    updater::check_for_updates().await
}

#[tauri::command]
async fn install_app_update(
    app: AppHandle,
    state: State<'_, AppState>,
    asset_url: String,
    asset_size: Option<u64>,
) -> Result<(), String> {
    updater::download_and_install_update(&app, &state.paths.cache_dir, &asset_url, asset_size)
        .await?;
    state.player.stop();
    app.exit(0);
    Ok(())
}

/// Progressively fetches streams from every supporting addon and active JS scraper in parallel,
/// emitting `streams:addon` as each provider responds and `streams:active_scrapers` while scrapers run.
#[tauri::command]
async fn start_stream_search(
    app: AppHandle,
    state: State<'_, AppState>,
    media_type: String,
    id: String,
    tmdb_id: Option<String>,
    season: Option<i32>,
    episode: Option<i32>,
) -> Result<ProgressiveStartAck, String> {
    let addons = load_addons_inner(&state.kv)?;
    let matching: Vec<AddonManifest> = addons
        .into_iter()
        .filter(|a| a.enabled && a.manifest.supports_resource("stream", &media_type, &id))
        .map(|a| a.manifest)
        .collect();

    let settings = load_settings_inner(&state.kv);
    let plugins_state = load_plugins_state(&state.kv);
    let active_scrapers: Vec<(plugins_repo::PluginScraperItem, String)> = if plugins_state.plugins_enabled {
        plugins_state
            .scrapers
            .into_iter()
            .filter(|s| s.enabled && s.supports_type(&media_type))
            .map(|s| {
                let cfg = plugins_repo::load_scraper_settings(&state.kv, &s.id);
                (s, cfg)
            })
            .collect()
    } else {
        Vec::new()
    };

    let resolved_tmdb = tmdb_id.or_else(|| {
        id.strip_prefix("tmdb:")
            .map(|rest| rest.split(':').next().unwrap_or(rest).to_string())
    });
    let can_resolve_imdb = resolved_tmdb.is_none()
        && id.starts_with("tt")
        && !settings.tmdb_api_key.trim().is_empty();
    let will_run_scrapers = !active_scrapers.is_empty() && (resolved_tmdb.is_some() || can_resolve_imdb);

    let scraper_names: Vec<String> = if will_run_scrapers {
        active_scrapers.iter().map(|(s, _)| s.name.clone()).collect()
    } else {
        Vec::new()
    };
    let _ = app.emit("streams:active_scrapers", scraper_names);

    let total = matching.len() + if will_run_scrapers { active_scrapers.len() } else { 0 };
    let addon_offset = matching.len();

    let shared_http = state.addons_http.clone();
    tokio::spawn(async move {
        let mut handles = Vec::with_capacity(total);
        for (order, manifest) in matching.into_iter().enumerate() {
            let app_clone = app.clone();
            let mt = media_type.clone();
            let vid = id.clone();
            let http = shared_http.clone();
            handles.push(tokio::spawn(async move {
                if let Ok(streams) = http.fetch_streams(&manifest, &mt, &vid).await
                    && !streams.is_empty()
                {
                    let _ = app_clone.emit(
                        "streams:addon",
                        AddonStreamGroup {
                            order,
                            addon_id: manifest.id,
                            addon_name: manifest.name,
                            addon_logo: manifest.logo_url,
                            streams,
                        },
                    );
                }
            }));
        }

        if will_run_scrapers {
            let app_sc = app.clone();
            let mt_sc = media_type.clone();
            let id_sc = id.clone();
            let tmdb_key = settings.tmdb_api_key;
            handles.push(tokio::spawn(async move {
                let tid = match resolved_tmdb {
                    Some(t) => Some(t),
                    None if id_sc.starts_with("tt") => {
                        resolve_imdb_to_tmdb(&id_sc, &mt_sc, &tmdb_key).await
                    }
                    None => None,
                };
                let Some(tid) = tid else {
                    let _ = app_sc.emit("streams:active_scrapers", Vec::<String>::new());
                    return;
                };

                // Warm TMDB metadata once in the shared single-flight cache so scrapers share 1 lookup.
                prefetch_tmdb_for_plugins(&tid, &mt_sc, season, &tmdb_key).await;

                let remaining = Arc::new(std::sync::Mutex::new(
                    active_scrapers
                        .iter()
                        .map(|(s, _)| s.name.clone())
                        .collect::<Vec<_>>(),
                ));
                let mut scraper_handles = Vec::with_capacity(active_scrapers.len());
                for (s_idx, (scraper, settings_c)) in active_scrapers.into_iter().enumerate() {
                    let app_clone = app_sc.clone();
                    let mt = mt_sc.clone();
                    let tid_c = tid.clone();
                    let key_c = tmdb_key.clone();
                    let rem_c = Arc::clone(&remaining);
                    scraper_handles.push(tokio::spawn(async move {
                        let res = plugins_repo::run_scraper_for_stream(
                            &scraper,
                            &tid_c,
                            &mt,
                            season,
                            episode,
                            &key_c,
                            &settings_c,
                        )
                        .await;
                        if let Ok(list) = res
                            && !list.is_empty()
                        {
                            let mapped: Vec<AddonStream> = list
                                .into_iter()
                                .map(|ps| AddonStream {
                                    name: Some(
                                        ps.name
                                            .filter(|s| !s.trim().is_empty())
                                            .unwrap_or_else(|| scraper.name.clone()),
                                    ),
                                    title: ps.title.filter(|s| !s.trim().is_empty()),
                                    description: None,
                                    url: Some(ps.url),
                                    info_hash: None,
                                    file_idx: None,
                                    sources: Vec::new(),
                                    behavior_hints: (!ps.headers.is_empty()).then(|| {
                                        serde_json::json!({ "proxyHeaders": { "request": ps.headers } })
                                    }),
                                })
                                .collect();
                            let _ = app_clone.emit(
                                "streams:addon",
                                AddonStreamGroup {
                                    order: addon_offset + s_idx,
                                    addon_id: format!("plugin:{}", scraper.id),
                                    addon_name: scraper.name.clone(),
                                    addon_logo: None,
                                    streams: mapped,
                                },
                            );
                        }
                        let next_active = {
                            let mut g = rem_c.lock().expect("rem lock");
                            g.retain(|n| n != &scraper.name);
                            g.clone()
                        };
                        let _ = app_clone.emit("streams:active_scrapers", next_active);
                    }));
                }
                for sh in scraper_handles {
                    let _ = sh.await;
                }
            }));
        }

        for h in handles {
            let _ = h.await;
        }
        let _ = app.emit("streams:active_scrapers", Vec::<String>::new());
        let _ = app.emit("streams:done", total);
        player::trim_process_heap();
    });

    Ok(ProgressiveStartAck { total })
}

// --- PLUGINS REPOSITORY MANAGER ---
#[tauri::command]
fn list_plugins_state(state: State<'_, AppState>) -> PluginsState {
    load_plugins_state(&state.kv)
}

#[tauri::command]
async fn add_plugin_repo(state: State<'_, AppState>, manifest_url: String) -> Result<PluginsState, String> {
    let url = plugins_repo::normalize_manifest_url(&manifest_url)?;
    let mut ps = load_plugins_state(&state.kv);
    if ps.repositories.iter().any(|r| r.manifest_url == url) {
        return Err("Repositório já instalado.".to_string());
    }
    let (repo, scrapers) = plugins_repo::fetch_plugin_repository(&url, &ps.scrapers).await?;
    ps.repositories.push(repo);
    ps.scrapers.retain(|s| s.repository_url != url);
    ps.scrapers.extend(scrapers);
    save_plugins_state(&state.kv, &ps)?;
    let eff = resolve_effective_plugins_profile_id_active(&state.kv);
    plugins_repo::spawn_push_plugins(current_supabase(&state), load_session_inner(&state.kv), eff, &ps);
    Ok(ps)
}

#[tauri::command]
async fn refresh_plugin_repo(state: State<'_, AppState>, manifest_url: String) -> Result<PluginsState, String> {
    let url = plugins_repo::normalize_manifest_url(&manifest_url)?;
    let mut ps = load_plugins_state(&state.kv);
    let (repo, scrapers) = plugins_repo::fetch_plugin_repository(&url, &ps.scrapers).await?;
    ps.repositories.retain(|r| r.manifest_url != url);
    ps.repositories.push(repo);
    ps.scrapers.retain(|s| s.repository_url != url);
    ps.scrapers.extend(scrapers);
    save_plugins_state(&state.kv, &ps)?;
    Ok(ps)
}

#[tauri::command]
fn remove_plugin_repo(state: State<'_, AppState>, manifest_url: String) -> Result<PluginsState, String> {
    let mut ps = load_plugins_state(&state.kv);
    ps.repositories.retain(|r| r.manifest_url != manifest_url);
    ps.scrapers.retain(|s| s.repository_url != manifest_url);
    save_plugins_state(&state.kv, &ps)?;
    let eff = resolve_effective_plugins_profile_id_active(&state.kv);
    plugins_repo::spawn_push_plugins(current_supabase(&state), load_session_inner(&state.kv), eff, &ps);
    Ok(ps)
}

#[tauri::command]
fn toggle_plugin_scraper(
    state: State<'_, AppState>,
    scraper_id: String,
    enabled: bool,
) -> Result<PluginsState, String> {
    let mut ps = load_plugins_state(&state.kv);
    if let Some(s) = ps.scrapers.iter_mut().find(|s| s.id == scraper_id) {
        s.enabled = enabled;
        save_plugins_state(&state.kv, &ps)?;
    }
    Ok(ps)
}

#[tauri::command]
fn set_plugins_enabled(
    state: State<'_, AppState>,
    enabled: Option<bool>,
    group_by_repo: Option<bool>,
) -> Result<PluginsState, String> {
    let mut ps = load_plugins_state(&state.kv);
    if let Some(e) = enabled {
        ps.plugins_enabled = e;
    }
    if let Some(g) = group_by_repo {
        ps.group_by_repo = g;
    }
    save_plugins_state(&state.kv, &ps)?;
    Ok(ps)
}

#[tauri::command]
async fn test_plugin_scraper(
    state: State<'_, AppState>,
    scraper_id: String,
) -> Result<Vec<PluginStreamResult>, String> {
    let ps = load_plugins_state(&state.kv);
    let scraper = ps
        .scrapers
        .into_iter()
        .find(|s| s.id == scraper_id)
        .ok_or_else(|| "Scraper não encontrado.".to_string())?;
    let settings = load_settings_inner(&state.kv);
    let scraper_settings = plugins_repo::load_scraper_settings(&state.kv, &scraper.id);
    let media_type = if scraper.supports_type("movie") { "movie" } else { "tv" };
    let (s, e) = if media_type == "tv" { (Some(1), Some(1)) } else { (None, None) };
    plugins_repo::run_scraper_for_stream(
        &scraper,
        "603",
        media_type,
        s,
        e,
        &settings.tmdb_api_key,
        &scraper_settings,
    )
    .await
}

#[tauri::command]
async fn get_plugin_settings_layout(
    state: State<'_, AppState>,
    scraper_id: String,
) -> Result<serde_json::Value, String> {
    let ps = load_plugins_state(&state.kv);
    let scraper = ps
        .scrapers
        .into_iter()
        .find(|s| s.id == scraper_id)
        .ok_or_else(|| "Scraper não encontrado.".to_string())?;
    let settings = load_settings_inner(&state.kv);
    let saved_raw = plugins_repo::load_scraper_settings(&state.kv, &scraper.id);
    let layout_raw = PluginRuntime::get_settings_layout(
        scraper.code,
        scraper.id.clone(),
        saved_raw.clone(),
        settings.tmdb_api_key,
    )
    .await
    .map_err(|e| e.to_string())?;
    let layout: serde_json::Value =
        serde_json::from_str(&layout_raw).unwrap_or_else(|_| serde_json::json!([]));
    let values: serde_json::Value =
        serde_json::from_str(&saved_raw).unwrap_or_else(|_| serde_json::json!({}));
    Ok(serde_json::json!({
        "layout": layout,
        "values": values,
    }))
}

#[tauri::command]
fn save_plugin_scraper_settings(
    state: State<'_, AppState>,
    scraper_id: String,
    settings: serde_json::Value,
) -> Result<(), String> {
    plugins_repo::save_scraper_settings(&state.kv, &scraper_id, &settings.to_string())
}

// --- FOLLOWED PEOPLE (LOCAL PER-PROFILE SQLITE) ---
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FollowedPerson {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    photo: String,
    #[serde(default)]
    role: String,
}

#[derive(Debug, Serialize)]
struct ProfileInsightsResponse {
    profile_index: i32,
    library: Vec<Value>,
    watched_items: Vec<SyncedWatchedItem>,
    watch_progress: Vec<WatchProgressSyncEntry>,
    followed_people: Vec<FollowedPerson>,
}

fn followed_people_key(kv: &KvStore) -> String {
    format!("profile_{}", active_profile_idx(kv))
}

fn load_followed_people_inner(kv: &KvStore) -> Vec<FollowedPerson> {
    let key = followed_people_key(kv);
    kv.namespace("followed_people")
        .get_string(&key)
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn parse_watched_key_item(raw: &str) -> Option<SyncedWatchedItem> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parts: Vec<&str> = trimmed.rsplitn(3, ':').collect();
    if parts.len() == 3
        && let (Ok(ep), Ok(sea)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>())
    {
        return Some(SyncedWatchedItem {
            content_id: parts[2].to_string(),
            content_type: "series".to_string(),
            title: String::new(),
            season: Some(sea),
            episode: Some(ep),
            watched_at: 0,
        });
    }
    Some(SyncedWatchedItem {
        content_id: trimmed.to_string(),
        content_type: "movie".to_string(),
        title: String::new(),
        season: None,
        episode: None,
        watched_at: 0,
    })
}

#[tauri::command]
async fn profile_insights(state: State<'_, AppState>) -> Result<ProfileInsightsResponse, String> {
    let pid = active_profile_idx(&state.kv);
    let mut library = load_local_library_inner(&state.kv);
    let mut watch_progress: Vec<WatchProgressSyncEntry> =
        load_local_progress_inner(&state.kv).into_values().collect();
    let local_watched = load_watched_keys_inner(&state.kv);
    let mut watched_items: Vec<SyncedWatchedItem> = local_watched
        .iter()
        .filter_map(|k| parse_watched_key_item(k))
        .collect();

    if let (Some(client), Some(session)) =
        (current_supabase(&state), load_session_inner(&state.kv))
    {
        client.import_session(session);
        let (lib_res, wp_res, wk_p1) = tokio::join!(
            client.pull_library(pid, 500, 0),
            client.pull_watch_progress(pid, 500),
            client.pull_watched_items(pid, 1, 500),
        );
        if let Ok(items) = lib_res {
            let _ = save_local_library_inner(&state.kv, &items);
            library = items;
        }
        if let Ok(entries) = wp_res {
            let map: HashMap<String, WatchProgressSyncEntry> = entries
                .iter()
                .map(|e| (e.progress_key.clone(), e.clone()))
                .collect();
            let _ = save_local_progress_inner(&state.kv, &map);
            watch_progress = entries;
        }
        if let Ok(mut items) = wk_p1 {
            let mut page = 2;
            let mut last_len = items.len();
            while last_len == 500 && page <= 10 {
                let Ok(next_page) = client.pull_watched_items(pid, page, 500).await else {
                    break;
                };
                last_len = next_page.len();
                items.extend(next_page);
                page += 1;
            }
            let mut key_set: HashSet<String> = local_watched.into_iter().collect();
            for it in &items {
                key_set.insert(watched_key(&it.content_id, it.season, it.episode));
            }
            let merged_keys: Vec<String> = key_set.into_iter().collect();
            let _ = save_watched_keys_inner(&state.kv, &merged_keys);
            watched_items = items;
        }
    }

    Ok(ProfileInsightsResponse {
        profile_index: pid,
        library,
        watched_items,
        watch_progress,
        followed_people: load_followed_people_inner(&state.kv),
    })
}

#[tauri::command]
fn list_followed_people(state: State<'_, AppState>) -> Vec<FollowedPerson> {
    load_followed_people_inner(&state.kv)
}

#[tauri::command]
fn save_followed_people(
    state: State<'_, AppState>,
    people: Vec<FollowedPerson>,
) -> Result<(), String> {
    let key = followed_people_key(&state.kv);
    let json = serde_json::to_string(&people).map_err(|e| e.to_string())?;
    state
        .kv
        .namespace("followed_people")
        .put_string(&key, Some(&json))
        .map_err(|e| e.to_string())
}

// --- LIBRARY, WATCHED & WATCH PROGRESS SYNC ---
#[tauri::command]
fn toggle_library_item(state: State<'_, AppState>, item: Value) -> Result<bool, String> {
    let id = item
        .get("content_id")
        .or_else(|| item.get("id"))
        .and_then(|v| v.as_str())
        .ok_or("Missing content_id")?
        .to_string();
    let ctype = item
        .get("content_type")
        .or_else(|| item.get("type"))
        .and_then(|v| v.as_str())
        .unwrap_or("movie")
        .to_string();

    let mut list = load_local_library_inner(&state.kv);
    let exists = list.iter().any(|x| {
        x.get("content_id").and_then(|v| v.as_str()) == Some(id.as_str())
            && x.get("content_type").and_then(|v| v.as_str()) == Some(ctype.as_str())
    });

    if exists {
        list.retain(|x| {
            !(x.get("content_id").and_then(|v| v.as_str()) == Some(id.as_str())
                && x.get("content_type").and_then(|v| v.as_str()) == Some(ctype.as_str()))
        });
    } else {
        list.insert(0, item.clone());
    }
    save_local_library_inner(&state.kv, &list)?;

    if let (Some(client), Some(session)) = (current_supabase(&state), load_session_inner(&state.kv))
    {
        let pid = active_profile_idx(&state.kv);
        tauri::async_runtime::spawn(async move {
            client.import_session(session);
            if exists {
                let key = serde_json::json!({ "content_id": id, "content_type": ctype });
                let _ = client.delete_library_items(pid, &[key]).await;
            } else {
                let _ = client.push_library_items(pid, &[item]).await;
            }
        });
    }
    Ok(!exists)
}

#[tauri::command]
async fn list_watched_keys(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut keys = load_watched_keys_inner(&state.kv);
    if let (Some(client), Some(session)) = (current_supabase(&state), load_session_inner(&state.kv))
    {
        client.import_session(session);
        let pid = active_profile_idx(&state.kv);
        if let Ok(items) = client.pull_watched_items(pid, 1, 250).await {
            let mut set: HashSet<String> = keys.into_iter().collect();
            for it in items {
                set.insert(watched_key(&it.content_id, it.season, it.episode));
            }
            keys = set.into_iter().collect();
            let _ = save_watched_keys_inner(&state.kv, &keys);
        }
    }
    Ok(keys)
}

#[tauri::command]
fn toggle_watched_item(
    state: State<'_, AppState>,
    content_id: String,
    content_type: String,
    title: String,
    season: Option<i32>,
    episode: Option<i32>,
) -> Result<bool, String> {
    let key = watched_key(&content_id, season, episode);
    let mut keys = load_watched_keys_inner(&state.kv);
    let was_watched = keys.iter().any(|k| k == &key);
    if was_watched {
        keys.retain(|k| k != &key);
    } else {
        keys.push(key);
    }
    save_watched_keys_inner(&state.kv, &keys)?;

    if let (Some(client), Some(session)) = (current_supabase(&state), load_session_inner(&state.kv))
    {
        let pid = active_profile_idx(&state.kv);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        tauri::async_runtime::spawn(async move {
            client.import_session(session);
            if was_watched {
                let del = serde_json::json!({
                    "content_id": content_id,
                    "season": season,
                    "episode": episode,
                });
                let _ = client.delete_watched_items(pid, &[del]).await;
            } else {
                let row = SyncedWatchedItem {
                    content_id,
                    content_type,
                    title,
                    season,
                    episode,
                    watched_at: now_ms,
                };
                let _ = client.push_watched_items(pid, &[row]).await;
            }
        });
    }
    Ok(!was_watched)
}

#[tauri::command]
fn upsert_watch_progress(
    state: State<'_, AppState>,
    entry: WatchProgressSyncEntry,
    sync_remote: bool,
) -> Result<(), String> {
    let mut map = load_local_progress_inner(&state.kv);
    let pkey = if entry.progress_key.is_empty() {
        entry.video_id.clone()
    } else {
        entry.progress_key.clone()
    };
    let mut stored = entry.clone();
    stored.progress_key = pkey.clone();
    map.insert(pkey, stored.clone());
    save_local_progress_inner(&state.kv, &map)?;

    if sync_remote
        && let (Some(client), Some(session)) =
            (current_supabase(&state), load_session_inner(&state.kv))
    {
        let pid = active_profile_idx(&state.kv);
        tauri::async_runtime::spawn(async move {
            client.import_session(session);
            let _ = client.push_watch_progress(pid, &[stored]).await;
        });
    }
    Ok(())
}

#[tauri::command]
fn remove_watch_progress(
    state: State<'_, AppState>,
    progress_key: String,
) -> Result<(), String> {
    let mut map = load_local_progress_inner(&state.kv);
    map.remove(&progress_key);
    save_local_progress_inner(&state.kv, &map)?;
    if let (Some(client), Some(session)) = (current_supabase(&state), load_session_inner(&state.kv))
    {
        let pid = active_profile_idx(&state.kv);
        tauri::async_runtime::spawn(async move {
            client.import_session(session);
            let _ = client.delete_watch_progress(pid, &[progress_key]).await;
        });
    }
    Ok(())
}

#[tauri::command]
fn clear_continue_watching_cache(state: State<'_, AppState>) -> Result<(), String> {
    state
        .kv
        .namespace("watch_progress")
        .remove("entries_json")
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn run_js_plugin(
    state: State<'_, AppState>,
    code: String,
    tmdb_id: String,
    media_type: String,
    season: Option<i32>,
    episode: Option<i32>,
) -> Result<Vec<PluginStreamResult>, String> {
    let settings = load_settings_inner(&state.kv);
    PluginRuntime::execute_plugin(
        code,
        "desktop_sandbox".to_string(),
        "{}".to_string(),
        settings.tmdb_api_key,
        PluginCallArgs {
            tmdb_id,
            media_type,
            season,
            episode,
        },
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn player_play(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    url: String,
    headers: Option<HashMap<String, String>>,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    start_secs: Option<f64>,
    initial_controls_json: Option<String>,
    anime: Option<bool>,
) -> Result<String, String> {
    let clean_url = url.trim().to_string();
    if clean_url.is_empty() {
        return Err("Empty stream URL.".to_string());
    }
    #[cfg(target_os = "android")]
    {
        let _ = (app, window, state, headers, x, y, w, h, start_secs, initial_controls_json, anime);
        return Ok("webview:html5".to_string());
    }
    #[cfg(not(target_os = "android"))]
    {
    let settings = load_settings_inner(&state.kv);
    let options = settings
        .playback
        .prepare(anime.unwrap_or(false), &state.paths.cache_dir)?
        .with_languages(&settings.preferred_audio_lang, &settings.preferred_sub_lang);
    if settings.external_player_enabled {
        return launch_mpv_external(&state, &clean_url, headers, &options);
    }

    let scale = window.scale_factor().unwrap_or(1.0);
    let rect = ViewportRect {
        x: (f64::from(x) * scale).round() as i32,
        y: (f64::from(y) * scale).round() as i32,
        w: (f64::from(w) * scale).round().max(16.0) as u32,
        h: (f64::from(h) * scale).round().max(16.0) as u32,
    };

    let controls_url = export_player_ui_assets(&state.paths.cache_dir)?;

    let res = state.player.play(
        app,
        &window,
        &clean_url,
        headers.clone(),
        rect,
        start_secs,
        controls_url,
        initial_controls_json,
        &options,
    );
    match res {
        Ok(has_overlay) => {
            let _ = state.player.apply_subtitle_style(&settings.subtitle_style);
            Ok(if has_overlay {
                "embedded:overlay".to_string()
            } else {
                "embedded".to_string()
            })
        }
        Err(embed_err) => {
            tracing::error!("Embedded libmpv unavailable ({embed_err}); falling back to external mpv");
            eprintln!("Embedded libmpv unavailable ({embed_err}); falling back to external mpv");
            launch_mpv_external(&state, &clean_url, headers, &options)
        }
    }
    }
}

#[tauri::command]
fn player_set_bounds(
    window: WebviewWindow,
    state: State<'_, AppState>,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
) {
    let scale = window.scale_factor().unwrap_or(1.0);
    state.player.set_bounds(
        &window,
        (f64::from(x) * scale).round() as i32,
        (f64::from(y) * scale).round() as i32,
        (f64::from(w) * scale).round().max(16.0) as u32,
        (f64::from(h) * scale).round().max(16.0) as u32,
    );
}

#[tauri::command]
fn player_push_controls(state: State<'_, AppState>, controls_json: String) {
    state.player.push_controls_json(controls_json);
}

#[tauri::command]
fn player_focus_overlay(state: State<'_, AppState>) {
    state.player.focus_overlay();
}

#[tauri::command]
fn player_ui_message(
    app: AppHandle,
    state: State<'_, AppState>,
    event_type: String,
    value: f64,
) {
    state.player.handle_ui_action(&event_type, value);
    let _ = app.emit(
        "player:ui_event",
        PlayerUiEventPayload { event_type, value },
    );
}

#[tauri::command]
fn player_set_paused(state: State<'_, AppState>, paused: bool) -> Result<(), String> {
    state.player.set_paused(paused)
}

#[tauri::command]
fn player_seek(state: State<'_, AppState>, secs: f64) -> Result<(), String> {
    state.player.seek_absolute(secs)
}

#[tauri::command]
fn player_set_volume(state: State<'_, AppState>, volume: f64) -> Result<(), String> {
    state.player.set_volume(volume)
}

#[tauri::command]
fn player_set_speed(state: State<'_, AppState>, speed: f64) -> Result<(), String> {
    state.player.set_speed(speed)
}

#[tauri::command]
fn player_set_resize_mode(state: State<'_, AppState>, mode: usize) -> Result<(), String> {
    state.player.set_resize_mode(mode)
}

#[tauri::command]
fn player_set_audio(state: State<'_, AppState>, aid: String) -> Result<(), String> {
    state.player.set_audio_track(&aid)
}

#[tauri::command]
fn player_set_subtitle(state: State<'_, AppState>, sid: String) -> Result<(), String> {
    state.player.set_subtitle_track(&sid)
}

#[tauri::command]
fn player_add_subtitle_url(state: State<'_, AppState>, url: String) -> Result<(), String> {
    state.player.add_subtitle_url(&url)
}

#[tauri::command]
fn player_clear_external_subtitles(state: State<'_, AppState>) -> Result<(), String> {
    state.player.clear_external_subtitles()
}

#[tauri::command]
fn player_set_subtitle_visible(state: State<'_, AppState>, visible: bool) -> Result<(), String> {
    state.player.set_subtitle_visible(visible)
}

#[tauri::command]
fn player_set_subtitle_delay(state: State<'_, AppState>, delay_ms: i64) -> Result<(), String> {
    state.player.set_subtitle_delay_ms(delay_ms)
}

#[tauri::command]
fn player_apply_subtitle_style(
    state: State<'_, AppState>,
    style: SubtitleStylePayload,
) -> Result<(), String> {
    state.player.apply_subtitle_style(&style)
}

#[tauri::command]
async fn player_fetch_addon_subtitles(
    state: State<'_, AppState>,
    media_type: String,
    id: String,
    stream_url: Option<String>,
) -> Result<Vec<AddonSubtitle>, String> {
    let addons = load_addons_inner(&state.kv)?;
    let http = state.addons_http.clone();
    let hash_info = match stream_url.as_deref() {
        Some(url) => http.compute_opensubtitles_hash(url).await,
        None => None,
    };
    let extra = hash_info.map(|(h, size)| format!("videoHash={h}&videoSize={size}"));

    let mut handles = Vec::new();
    for addon in addons.into_iter().filter(|a| a.enabled) {
        if addon
            .manifest
            .subtitles_resource_name(&media_type, &id)
            .is_none()
        {
            continue;
        }
        let hc = http.clone();
        let m = addon.manifest;
        let mt = media_type.clone();
        let vid = id.clone();
        let ex = extra.clone();
        handles.push(tokio::spawn(async move {
            hc.fetch_subtitles(&m, &mt, &vid, ex.as_deref())
                .await
                .unwrap_or_default()
        }));
    }

    let mut out = Vec::new();
    for h in handles {
        if let Ok(subs) = h.await {
            out.extend(subs);
        }
    }
    Ok(out)
}

#[tauri::command]
fn player_set_reference_track(state: State<'_, AppState>, sid: Option<i64>) -> Result<(), String> {
    state.player.set_reference_track(sid)
}

#[tauri::command]
async fn player_load_subtitle_cues(state: State<'_, AppState>, url: String) -> Result<Vec<SubtitleCueItem>, String> {
    let text = state.addons_http.fetch_text(&url).await.map_err(|e| e.to_string())?;
    Ok(parse_subtitle_cues(&text))
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn player_resolve_skip_intervals(
    state: State<'_, AppState>,
    video_id: String,
    tmdb_id: Option<String>,
    media_type: String,
    season: Option<i32>,
    episode: Option<i32>,
    chapters: Vec<SkipChapter>,
) -> Result<Vec<SkipInterval>, String> {
    let settings = load_settings_inner(&state.kv);
    if !settings.skip_intro_enabled {
        return Ok(Vec::new());
    }
    let chap_intervals = skip::chapters_to_skip_intervals(&chapters);
    Ok(resolve_skip_intervals(
        &video_id,
        tmdb_id.as_deref(),
        &media_type,
        season,
        episode,
        &settings.introdb_url,
        &settings.tmdb_api_key,
        settings.aniskip_enabled,
        chap_intervals,
    )
    .await)
}

#[tauri::command]
async fn player_submit_intro(
    state: State<'_, AppState>,
    video_id: String,
    season: Option<i32>,
    episode: Option<i32>,
    start_sec: f64,
    end_sec: f64,
    segment_type: String,
) -> Result<(), String> {
    let settings = load_settings_inner(&state.kv);
    submit_introdb_segment(
        &settings.introdb_url,
        &settings.introdb_api_key,
        &video_id,
        season,
        episode,
        start_sec,
        end_sec,
        &segment_type,
    )
    .await
}

#[tauri::command]
async fn player_stop(state: State<'_, AppState>) -> Result<(), String> {
    state.player.stop();
    Ok(())
}

#[tauri::command]
fn window_toggle_fullscreen(app: AppHandle, window: WebviewWindow) -> Result<bool, String> {
    #[cfg(target_os = "android")]
    {
        let _ = (app, window);
        Ok(false)
    }
    #[cfg(not(target_os = "android"))]
    {
        let target = app.get_webview_window("main").unwrap_or(window);
        let current = target.is_fullscreen().map_err(|e| e.to_string())?;
        target
            .set_fullscreen(!current)
            .map_err(|e| e.to_string())?;
        Ok(!current)
    }
}

fn launch_mpv_external(
    state: &AppState,
    clean_url: &str,
    headers: Option<HashMap<String, String>>,
    options: &video_options::VideoOptions,
) -> Result<String, String> {
    let settings = load_settings_inner(&state.kv);
    let mpv_bin = resolve_mpv_binary(&settings.mpv_path);
    let mut cmd = Command::new(&mpv_bin);
    cmd.arg("--force-window=yes");
    for (name, value) in &options.options {
        cmd.arg(format!("--{name}={value}"));
    }
    if let Some(ref alang) = options.alang {
        cmd.arg(format!("--alang={alang}"));
    }
    if let Some(ref slang) = options.slang {
        cmd.arg(format!("--slang={slang}"));
    }
    if let Some(path) = &options.config {
        cmd.arg(format!("--include={}", path.display()));
    }
    if let Some(hdrs) = headers
        && let Some(arg_val) = player::format_external_mpv_header_fields(&hdrs)
    {
        cmd.arg(format!("--http-header-fields={arg_val}"));
    }
    cmd.arg("--").arg(clean_url);
    cmd.spawn()
        .map(|child| format!("external:{mpv_bin}:{}", child.id()))
        .map_err(|e| format!("Falha ao abrir '{mpv_bin}': {e}"))
}

#[tauri::command]
fn launch_mpv(
    state: State<'_, AppState>,
    url: String,
    anime: Option<bool>,
    headers: Option<HashMap<String, String>>,
) -> Result<String, String> {
    let clean_url = url.trim();
    if clean_url.is_empty() {
        return Err("Empty stream URL.".to_string());
    }
    let settings = load_settings_inner(&state.kv);
    let options = settings
        .playback
        .prepare(anime.unwrap_or(false), &state.paths.cache_dir)?
        .with_languages(&settings.preferred_audio_lang, &settings.preferred_sub_lang);
    launch_mpv_external(&state, clean_url, headers, &options)
}

async fn complete_auth_session(
    state: &AppState,
    client: &SupabaseClient,
    session: SupabaseSession,
) -> Result<AuthStatusResponse, String> {
    let auth_ns = state.kv.namespace("auth");
    let json = serde_json::to_string(&session).map_err(|e| e.to_string())?;
    auth_ns
        .put_string("session_json", Some(&json))
        .map_err(|e| e.to_string())?;
    auth_ns.put_bool("is_guest", false).map_err(|e| e.to_string())?;

    let profiles = client.pull_profiles().await.unwrap_or_default();
    save_profiles_inner(&state.kv, &profiles)?;
    let first_idx = profiles.first().map(|p| p.profile_index).unwrap_or(1);
    state
        .kv
        .namespace("profiles")
        .put_int("active_index", i64::from(first_idx))
        .map_err(|e| e.to_string())?;

    let server = load_server_config_inner(&state.kv);
    let user = session.user.clone();
    Ok(AuthStatusResponse {
        signed_in: true,
        is_guest: false,
        email: user.as_ref().and_then(|u| u.email.clone()),
        user_id: user.as_ref().map(|u| u.id.clone()),
        server,
        profiles,
        active_profile_index: first_idx,
    })
}

fn default_guest_profile() -> NuvioProfile {
    NuvioProfile {
        id: "guest-1".to_string(),
        user_id: "guest".to_string(),
        profile_index: 1,
        name: "Local".to_string(),
        avatar_color_hex: "#E8A91C".to_string(),
        avatar_id: None,
        avatar_url: None,
        profile_background_id: None,
        profile_background_url: None,
        uses_primary_addons: false,
        uses_primary_plugins: false,
        pin_enabled: false,
    }
}

fn ensure_manifest_suffix(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let (base, query) = match trimmed.split_once('?') {
        Some((b, q)) => (b, Some(q)),
        None => (trimmed, None),
    };
    let clean_base = base.trim_end_matches('/');
    let with_manifest = if clean_base.ends_with("/manifest.json") {
        clean_base.to_string()
    } else {
        format!("{clean_base}/manifest.json")
    };
    match query {
        Some(q) if !q.is_empty() => format!("{with_manifest}?{q}"),
        _ => with_manifest,
    }
}

fn url_encode_segment(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for b in input.as_bytes() {
        if b.is_ascii_alphanumeric() || matches!(*b, b'-' | b'_' | b'.' | b'~') {
            out.push(*b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn format_supabase_err(err: nuvio_core::supabase::SupabaseError) -> String {
    match err {
        nuvio_core::supabase::SupabaseError::Api { status, body } => {
            if let Ok(val) = serde_json::from_str::<Value>(&body)
                && let Some(msg) = val
                    .get("error_description")
                    .or_else(|| val.get("msg"))
                    .or_else(|| val.get("message"))
                    .and_then(|v| v.as_str())
            {
                return msg.to_string();
            }
            format!("HTTP {status}: {body}")
        }
        other => other.to_string(),
    }
}

fn load_session_inner(kv: &KvStore) -> Option<SupabaseSession> {
    kv.namespace("auth")
        .get_string("session_json")
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

fn load_server_config_inner(kv: &KvStore) -> Option<ServerConfigState> {
    kv.namespace("server")
        .get_string("config_json")
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

fn save_server_config_inner(kv: &KvStore, cfg: &ServerConfigState) -> Result<(), String> {
    let json = serde_json::to_string(cfg).map_err(|e| e.to_string())?;
    kv.namespace("server")
        .put_string("config_json", Some(&json))
        .map_err(|e| e.to_string())
}

fn load_profiles_inner(kv: &KvStore) -> Vec<NuvioProfile> {
    kv.namespace("profiles")
        .get_string("list_json")
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_profiles_inner(kv: &KvStore, profiles: &[NuvioProfile]) -> Result<(), String> {
    let json = serde_json::to_string(profiles).map_err(|e| e.to_string())?;
    kv.namespace("profiles")
        .put_string("list_json", Some(&json))
        .map_err(|e| e.to_string())
}

fn load_settings_inner(kv: &KvStore) -> DesktopSettings {
    kv.namespace("desktop_settings")
        .get_string("config_json")
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn load_addons_inner(kv: &KvStore) -> Result<Vec<StoredAddon>, String> {
    let raw = kv
        .namespace("addons")
        .get_string("installed_json")
        .map_err(|e| e.to_string())?;
    match raw {
        Some(json) => serde_json::from_str(&json).map_err(|e| e.to_string()),
        None => Ok(Vec::new()),
    }
}

fn resolve_effective_profile_id(kv: &KvStore, profile_index: i32) -> i32 {
    load_profiles_inner(kv)
        .iter()
        .find(|p| p.profile_index == profile_index)
        .map(|p| if p.uses_primary_addons { 1 } else { p.profile_index })
        .unwrap_or(profile_index)
}

fn load_cached_manifest(kv: &KvStore, manifest_url: &str) -> Option<AddonManifest> {
    kv.namespace("addon_manifests_cache")
        .get_string(manifest_url)
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

fn save_cached_manifest(kv: &KvStore, manifest_url: &str, manifest: &AddonManifest) {
    if let Ok(json) = serde_json::to_string(manifest) {
        let _ = kv
            .namespace("addon_manifests_cache")
            .put_string(manifest_url, Some(&json));
    }
}

fn resolve_effective_plugins_profile_id(kv: &KvStore, profile_index: i32) -> i32 {
    load_profiles_inner(kv)
        .iter()
        .find(|p| p.profile_index == profile_index)
        .map(|p| if p.uses_primary_plugins { 1 } else { p.profile_index })
        .unwrap_or(profile_index)
}

fn active_profile_idx(kv: &KvStore) -> i32 {
    kv.namespace("profiles")
        .get_int("active_index")
        .ok()
        .flatten()
        .unwrap_or(1) as i32
}

fn resolve_effective_plugins_profile_id_active(kv: &KvStore) -> i32 {
    resolve_effective_plugins_profile_id(kv, active_profile_idx(kv))
}

fn watched_key(content_id: &str, season: Option<i32>, episode: Option<i32>) -> String {
    match (season, episode) {
        (Some(s), Some(e)) => format!("{content_id}:{s}:{e}"),
        _ => content_id.to_string(),
    }
}

fn load_watched_keys_inner(kv: &KvStore) -> Vec<String> {
    kv.namespace("watched")
        .get_string("keys_json")
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_watched_keys_inner(kv: &KvStore, keys: &[String]) -> Result<(), String> {
    let json = serde_json::to_string(keys).map_err(|e| e.to_string())?;
    kv.namespace("watched")
        .put_string("keys_json", Some(&json))
        .map_err(|e| e.to_string())
}

fn load_local_library_inner(kv: &KvStore) -> Vec<Value> {
    kv.namespace("library")
        .get_string("items_json")
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_local_library_inner(kv: &KvStore, items: &[Value]) -> Result<(), String> {
    let json = serde_json::to_string(items).map_err(|e| e.to_string())?;
    kv.namespace("library")
        .put_string("items_json", Some(&json))
        .map_err(|e| e.to_string())
}

fn load_local_progress_inner(kv: &KvStore) -> HashMap<String, WatchProgressSyncEntry> {
    kv.namespace("watch_progress")
        .get_string("entries_json")
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_local_progress_inner(
    kv: &KvStore,
    map: &HashMap<String, WatchProgressSyncEntry>,
) -> Result<(), String> {
    let json = serde_json::to_string(map).map_err(|e| e.to_string())?;
    kv.namespace("watch_progress")
        .put_string("entries_json", Some(&json))
        .map_err(|e| e.to_string())
}

fn spawn_cloud_addons_sync(state: &AppState, list: &[StoredAddon]) {
    let Some(client) = current_supabase(state) else {
        return;
    };
    let Some(session) = load_session_inner(&state.kv) else {
        return;
    };
    let active_idx = active_profile_idx(&state.kv);
    let effective_profile_id = resolve_effective_profile_id(&state.kv, active_idx);
    let rows: Vec<SyncedAddonRow> = list
        .iter()
        .enumerate()
        .map(|(idx, item)| SyncedAddonRow {
            url: item.manifest_url.clone(),
            name: Some(item.manifest.name.clone()),
            enabled: item.enabled,
            sort_order: idx as i32,
        })
        .collect();

    // CRITICAL: tauri::async_runtime::spawn works safely from synchronous #[tauri::command] handlers.
    tauri::async_runtime::spawn(async move {
        client.import_session(session);
        if let Err(err) = client.push_addons(effective_profile_id, &rows).await {
            tracing::warn!("Failed to push addons to cloud: {err}");
        }
    });
}

fn save_addons_inner(kv: &KvStore, list: &[StoredAddon]) -> Result<(), String> {
    for item in list {
        save_cached_manifest(kv, &item.manifest_url, &item.manifest);
    }
    let json = serde_json::to_string(list).map_err(|e| e.to_string())?;
    kv.namespace("addons")
        .put_string("installed_json", Some(&json))
        .map_err(|e| e.to_string())
}

fn resolve_mpv_binary(configured: &str) -> String {
    let trimmed = configured.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }
    if cfg!(target_os = "windows") {
        "mpv.exe".to_string()
    } else {
        "mpv".to_string()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(not(target_os = "android"))]
    crash_log::install();

    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("DISPLAY").is_some() && std::env::var_os("GDK_BACKEND").is_none() {
            // Match Nuvio Linux player_bridge.cpp so X11 wid embedding works under Wayland/XWayland.
            unsafe { std::env::set_var("GDK_BACKEND", "x11") };
        }
    }

    let _ = tracing_subscriber::fmt::try_init();

    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "android")]
            let paths = {
                let cfg = app
                    .path()
                    .app_config_dir()
                    .map_err(|e| std::io::Error::other(e.to_string()))?;
                let cache = app
                    .path()
                    .app_cache_dir()
                    .map_err(|e| std::io::Error::other(e.to_string()))?;
                let resolved = AppPaths::from_dirs(cfg, cache)?;
                crash_log::set_log_dir(resolved.config_dir.clone());
                crash_log::install();
                resolved
            };
            #[cfg(not(target_os = "android"))]
            let paths = AppPaths::resolve()?;

            let kv = KvStore::open(paths.db_path()).expect("Failed to open SQLite KV store");
            let startup_settings = load_settings_inner(&kv);
            nuvio_core::storage::dns::configure(
                &startup_settings.dns_provider,
                &startup_settings.dns_custom,
            );
            let cache = CacheStore::open(paths.cache_db_path()).expect("Failed to open Redb cache");
            let addons_http = AddonClient::new().expect("Failed to initialize HTTP client");
            // Keeps an hour of memory samples so a slow climb can be told from steady state.
            metrics::spawn_monitor();

            let state = AppState {
                paths,
                kv,
                cache,
                addons_http,
                home_catalog_gen: Arc::new(AtomicU64::new(0)),
                addon_search: Mutex::default(),
                supabase: Arc::new(RwLock::new(None)),
                player: EmbeddedPlayer::new(),
                party: Arc::new(party::PartyRuntime::new()),
            };
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_auth_status,
            simkl::simkl_status,
            simkl::simkl_action,
            mdblist_lib::mdblist_library_sync,
            mdblist_lib::mdblist_library_mutate,
            mdblist_lib::mdblist_account_status,
            mdblist_lib::mdblist_progress_sync,
            mdblist_lib::mdblist_watched_mutate,
            mdblist_lib::mdblist_scrobble,
            sign_in_email,
            sign_up_email,
            continue_as_guest,
            sign_out,
            start_device_link,
            poll_device_link,
            configure_server,
            select_profile,
            pull_account_library,
            pull_account_collections,
            start_continue_watching,
            start_home_catalogs,
            load_catalog_page,
            start_addon_search,
            fetch_meta_details,
            discord::discord_presence,
            discord::discord_probe,
            runtime_metrics,
            party::party_create,
            party::party_join,
            party::party_preview,
            party::party_leave,
            party::party_end,
            party::party_status,
            party::party_set_host_only,
            party::party_report_playback,
            party::party_set_status,
            party::party_send_message,
            party::party_retry_message,
            party::party_set_chat_open,
            party::party_remove_member,
            services::tmdb_season_episodes,
            services::tmdb_next_episodes,
            services::calendar_episodes,
            services::person_details,
            services::resolve_people_photos,
            services::hero_ratings,
            get_diagnostics,
            frontend_log,
            load_settings,
            save_settings,
            list_addons,
            add_addon,
            remove_addon,
            move_addon,
            toggle_addon,
            refresh_addon,
            open_external_url,
            connected::connected_accounts,
            connected::connected_disconnect,
            connected::connected_use_key,
            connected::connected_start,
            connected::connected_poll,
            connected::connected_files,
            connected::connected_play,
            connected::connected_resolve,
            connected::seekr_track,
            start_stream_search,
            run_js_plugin,
            list_plugins_state,
            add_plugin_repo,
            refresh_plugin_repo,
            remove_plugin_repo,
            toggle_plugin_scraper,
            set_plugins_enabled,
            test_plugin_scraper,
            get_plugin_settings_layout,
            save_plugin_scraper_settings,
            list_followed_people,
            save_followed_people,
            profile_insights,
            toggle_library_item,
            list_watched_keys,
            toggle_watched_item,
            upsert_watch_progress,
            remove_watch_progress,
            clear_continue_watching_cache,
            player_play,
            player_set_bounds,
            player_push_controls,
            player_focus_overlay,
            player_ui_message,
            player_set_paused,
            player_seek,
            player_set_volume,
            player_set_speed,
            player_set_resize_mode,
            player_set_audio,
            player_set_subtitle,
            player_add_subtitle_url,
            player_clear_external_subtitles,
            player_set_subtitle_visible,
            player_set_subtitle_delay,
            player_apply_subtitle_style,
            player_fetch_addon_subtitles,
            player_load_subtitle_cues,
            player_set_reference_track,
            player_resolve_skip_intervals,
            player_submit_intro,
            player_stop,
            window_toggle_fullscreen,
            launch_mpv,
            check_app_update,
            install_app_update,
        ])
        .on_window_event(|window, event| {
            #[cfg(not(target_os = "android"))]
            {
                if window.label() != "main" {
                    return;
                }
                match event {
                    tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_) => {
                        if let Some(main_win) = window.app_handle().get_webview_window("main") {
                            window
                                .app_handle()
                                .state::<AppState>()
                                .player
                                .sync_windows_overlay_bounds(&main_win);
                        }
                    }
                    tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed => {
                        window.app_handle().state::<AppState>().player.stop();
                        if let Some(ov) = window.app_handle().get_webview_window("player-overlay") {
                            let _ = ov.close();
                        }
                    }
                    _ => {}
                }
            }
            #[cfg(target_os = "android")]
            let _ = (window, event);
        })
        .run(tauri::generate_context!())
        .expect("error while running Nuvio Desktop Tauri application");
}

#[cfg(test)]
mod insights_tests {
    use super::{
        CatalogSection, catalog_items_changed, home_catalog_cache_key, parse_watched_key_item,
    };
    use nuvio_core::integrations::addons::MetaPreview;

    fn sample_meta(id: &str, name: &str) -> MetaPreview {
        MetaPreview {
            id: id.to_string(),
            r#type: "movie".to_string(),
            name: name.to_string(),
            poster: None,
            banner: None,
            logo: None,
            poster_shape: "poster".to_string(),
            description: None,
            release_info: None,
            imdb_rating: None,
            genres: Vec::new(),
            runtime: None,
        }
    }

    #[test]
    fn parses_movie_and_series_watched_keys() {
        let ep = parse_watched_key_item("tt6741278:4:3").expect("episode parsed");
        assert_eq!(ep.content_id, "tt6741278");
        assert_eq!(ep.content_type, "series");
        assert_eq!(ep.season, Some(4));
        assert_eq!(ep.episode, Some(3));

        let mv = parse_watched_key_item("tt4154796").expect("movie parsed");
        assert_eq!(mv.content_id, "tt4154796");
        assert_eq!(mv.content_type, "movie");
        assert_eq!(mv.season, None);
        assert_eq!(mv.episode, None);
    }

    #[test]
    fn detects_catalog_item_changes_for_swr_cache() {
        assert_eq!(home_catalog_cache_key(2), "home_catalogs:p2");
        let item_a = sample_meta("tt01", "Alpha");
        let item_b = sample_meta("tt02", "Beta");
        let cached = CatalogSection {
            order: 0,
            key: "k".to_string(),
            title: "Popular".to_string(),
            addon_name: "Addon".to_string(),
            manifest_url: "https://example.com/manifest.json".to_string(),
            media_type: "movie".to_string(),
            catalog_id: "top".to_string(),
            items: vec![item_a.clone()],
        };
        assert!(!catalog_items_changed(Some(&cached), &[item_a.clone()]));
        assert!(catalog_items_changed(Some(&cached), &[item_b]));
        assert!(catalog_items_changed(None, &[item_a]));
    }
}
