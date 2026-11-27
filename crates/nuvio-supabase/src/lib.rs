pub mod realtime;

use reqwest::{Client, Method, StatusCode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

pub const OFFICIAL_NUVIO_SERVER_URL: &str = "https://api.nuvio.tv";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(default)]
    pub email_password_auth: bool,
    #[serde(default)]
    pub tv_login: bool,
    #[serde(default)]
    pub watch_party: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryDocument {
    pub version: u32,
    pub service: String,
    #[serde(default)]
    pub self_hosted: bool,
    pub backend_url: String,
    pub publishable_key: String,
    pub capabilities: ServerCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuvioProfile {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub user_id: String,
    #[serde(default = "default_profile_index")]
    pub profile_index: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_avatar_color")]
    pub avatar_color_hex: String,
    pub avatar_id: Option<String>,
    pub avatar_url: Option<String>,
    pub profile_background_id: Option<String>,
    pub profile_background_url: Option<String>,
    #[serde(default)]
    pub uses_primary_addons: bool,
    #[serde(default)]
    pub uses_primary_plugins: bool,
    #[serde(default)]
    pub pin_enabled: bool,
}

fn default_profile_index() -> i32 {
    1
}

fn default_avatar_color() -> String {
    "#1E88E5".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedAddonRow {
    pub url: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub sort_order: i32,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLinkStart {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri_complete: String,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_seconds: u64,
}

fn default_poll_interval() -> u64 {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLinkPoll {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLinkExchange {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: Option<String>,
    pub user: Option<SupabaseUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchProgressSyncEntry {
    pub content_id: String,
    pub content_type: String,
    pub video_id: String,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    #[serde(default)]
    pub position: i64,
    #[serde(default)]
    pub duration: i64,
    #[serde(default)]
    pub last_watched: i64,
    #[serde(default)]
    pub progress_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedWatchedItem {
    pub content_id: String,
    pub content_type: String,
    #[serde(default)]
    pub title: String,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    #[serde(default)]
    pub watched_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedPluginRow {
    pub url: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub sort_order: i32,
}

/// Discovers a Nuvio backend configuration via `/.well-known/nuvio` (matching `ServerDiscovery.kt`).
pub async fn discover_nuvio_server(server_input: &str) -> Result<DiscoveryDocument> {
    let trimmed = server_input.trim();
    let with_scheme = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    let mut url = Url::parse(&with_scheme)?;
    let clean_path = url
        .path()
        .trim_end_matches("/.well-known/nuvio")
        .trim_end_matches('/');
    let discovery_path = if clean_path.is_empty() {
        "/.well-known/nuvio".to_string()
    } else {
        format!("{clean_path}/.well-known/nuvio")
    };
    url.set_path(&discovery_path);
    url.set_query(None);

    let client = nuvio_storage::dns::client_builder().build()?;
    let resp = client.get(url).send().await?;
    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        return Err(SupabaseError::Api {
            status: status.as_u16(),
            body,
        });
    }
    let doc: DiscoveryDocument = serde_json::from_str(&body)?;
    Ok(doc)
}

#[derive(Debug, thiserror::Error)]
pub enum SupabaseError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("API error ({status}): {body}")]
    Api { status: u16, body: String },
    #[error("No active Supabase session")]
    NotAuthenticated,
}

pub type Result<T> = std::result::Result<T, SupabaseError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseConfig {
    pub url: String,
    pub anon_key: String,
    pub fallback_url: Option<String>,
}

impl SupabaseConfig {
    pub fn new(url: impl Into<String>, anon_key: impl Into<String>) -> Self {
        Self {
            url: url.into().trim().trim_end_matches('/').to_string(),
            anon_key: anon_key.into().trim().to_string(),
            fallback_url: None,
        }
    }

    pub fn with_fallback(mut self, fallback: impl Into<String>) -> Self {
        let cleaned = fallback.into().trim().trim_end_matches('/').to_string();
        if !cleaned.is_empty() {
            self.fallback_url = Some(cleaned);
        }
        self
    }

    pub fn rest_url(&self) -> String {
        format!("{}/rest/v1", self.url)
    }

    pub fn auth_url(&self) -> String {
        format!("{}/auth/v1", self.url)
    }

    pub fn functions_url(&self) -> String {
        format!("{}/functions/v1", self.url)
    }

    pub fn storage_url(&self) -> String {
        format!("{}/storage/v1", self.url)
    }

    pub fn realtime_ws_url(&self) -> Result<String> {
        let mut parsed = Url::parse(&self.url)?;
        let ws_scheme = if parsed.scheme() == "https" {
            "wss"
        } else {
            "ws"
        };
        parsed
            .set_scheme(ws_scheme)
            .map_err(|_| url::ParseError::InvalidDomainCharacter)?;
        Ok(format!(
            "{}/realtime/v1/websocket?apikey={}&vsn=1.0.0&nuvio_realtime=1",
            parsed.as_str().trim_end_matches('/'),
            self.anon_key
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseSession {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub expires_in: Option<u64>,
    pub user: Option<SupabaseUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseUser {
    pub id: String,
    pub email: Option<String>,
    pub role: Option<String>,
    pub app_metadata: Option<Value>,
    pub user_metadata: Option<Value>,
}

type SessionListener = Arc<dyn Fn(&SupabaseSession) + Send + Sync>;

fn decode_base64url(input: &str) -> Option<Vec<u8>> {
    let mut buf: u32 = 0;
    let mut bits: u8 = 0;
    let mut out = Vec::with_capacity((input.len() * 3) / 4);
    for b in input.bytes() {
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            b'=' => break,
            _ => return None,
        };
        buf = (buf << 6) | u32::from(val);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Some(out)
}

fn jwt_exp_unix(token: &str) -> Option<u64> {
    let payload_b64 = token.split('.').nth(1)?;
    let bytes = decode_base64url(payload_b64)?;
    let payload: Value = serde_json::from_slice(&bytes).ok()?;
    payload.get("exp").and_then(Value::as_u64)
}

fn is_jwt_expiring_soon_at(token: &str, now_secs: u64, leeway_secs: u64) -> bool {
    let Some(exp) = jwt_exp_unix(token) else {
        return false;
    };
    now_secs.saturating_add(leeway_secs) >= exp
}

fn is_jwt_expired_error(err: &SupabaseError) -> bool {
    match err {
        SupabaseError::Api { status: 401, body } => {
            let lower = body.to_ascii_lowercase();
            body.contains("PGRST303")
                || lower.contains("jwt expired")
                || lower.contains("invalid jwt")
        }
        _ => false,
    }
}

/// Pure-Rust Supabase client covering Auth, PostgREST, Functions, Storage, and Realtime.
#[derive(Clone)]
pub struct SupabaseClient {
    config: SupabaseConfig,
    http: Client,
    session: Arc<RwLock<Option<SupabaseSession>>>,
    refresh_lock: Arc<tokio::sync::Mutex<()>>,
    session_listener: Arc<RwLock<Option<SessionListener>>>,
}

impl SupabaseClient {
    pub fn new(config: SupabaseConfig) -> Result<Self> {
        let http = nuvio_storage::dns::client_builder().build()?;
        Ok(Self {
            config,
            http,
            session: Arc::new(RwLock::new(None)),
            refresh_lock: Arc::new(tokio::sync::Mutex::new(())),
            session_listener: Arc::new(RwLock::new(None)),
        })
    }

    pub fn config(&self) -> &SupabaseConfig {
        &self.config
    }

    pub fn current_session(&self) -> Option<SupabaseSession> {
        self.session.read().expect("session lock poisoned").clone()
    }

    pub fn import_session(&self, session: SupabaseSession) {
        *self.session.write().expect("session lock poisoned") = Some(session);
    }

    pub fn set_session_listener<F>(&self, listener: F)
    where
        F: Fn(&SupabaseSession) + Send + Sync + 'static,
    {
        *self
            .session_listener
            .write()
            .expect("session listener lock poisoned") = Some(Arc::new(listener));
    }

    fn notify_session(&self, session: &SupabaseSession) {
        let cb = self
            .session_listener
            .read()
            .expect("session listener lock poisoned")
            .clone();
        if let Some(cb) = cb {
            cb(session);
        }
    }

    pub fn clear_session(&self) {
        *self.session.write().expect("session lock poisoned") = None;
    }

    fn bearer_token(&self) -> String {
        self.session
            .read()
            .expect("session lock poisoned")
            .as_ref()
            .map(|s| s.access_token.clone())
            .unwrap_or_else(|| self.config.anon_key.clone())
    }

    fn has_refresh_token(&self) -> bool {
        self.session
            .read()
            .expect("session lock poisoned")
            .as_ref()
            .and_then(|s| s.refresh_token.as_deref())
            .is_some_and(|t| !t.trim().is_empty())
    }

    async fn send_json_once<T: DeserializeOwned>(
        &self,
        method: Method,
        url: &str,
        body: Option<&Value>,
        extra_headers: &[(&str, &str)],
        bearer_override: Option<&str>,
    ) -> Result<T> {
        let token = bearer_override
            .map(str::to_string)
            .unwrap_or_else(|| self.bearer_token());
        let mut req = self
            .http
            .request(method, url)
            .header("apikey", &self.config.anon_key)
            .header("Authorization", format!("Bearer {token}"));

        for (k, v) in extra_headers {
            req = req.header(*k, *v);
        }
        if let Some(payload) = body {
            req = req.json(payload);
        }

        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(SupabaseError::Api {
                status: status.as_u16(),
                body: text,
            });
        }
        if text.trim().is_empty() {
            return Ok(serde_json::from_str("null")?);
        }
        Ok(serde_json::from_str(&text)?)
    }

    async fn send_json<T: DeserializeOwned>(
        &self,
        method: Method,
        url: &str,
        body: Option<&Value>,
        extra_headers: &[(&str, &str)],
    ) -> Result<T> {
        let used_token = self.bearer_token();
        match self
            .send_json_once(method.clone(), url, body, extra_headers, Some(&used_token))
            .await
        {
            Ok(val) => Ok(val),
            Err(err) if is_jwt_expired_error(&err) && self.has_refresh_token() => {
                self.refresh_session_if_needed(&used_token).await?;
                self.send_json_once(method, url, body, extra_headers, None)
                    .await
            }
            Err(err) => Err(err),
        }
    }

    // --- Auth ---

    pub async fn sign_up_email(&self, email: &str, password: &str) -> Result<SupabaseSession> {
        let url = format!("{}/signup", self.config.auth_url());
        let body = serde_json::json!({ "email": email, "password": password });
        let session: SupabaseSession = self
            .send_json_once(Method::POST, &url, Some(&body), &[], Some(&self.config.anon_key))
            .await?;
        self.import_session(session.clone());
        self.notify_session(&session);
        Ok(session)
    }

    pub async fn sign_in_email(&self, email: &str, password: &str) -> Result<SupabaseSession> {
        let url = format!("{}/token?grant_type=password", self.config.auth_url());
        let body = serde_json::json!({ "email": email, "password": password });
        let session: SupabaseSession = self
            .send_json_once(Method::POST, &url, Some(&body), &[], Some(&self.config.anon_key))
            .await?;
        self.import_session(session.clone());
        self.notify_session(&session);
        Ok(session)
    }

    async fn refresh_session_locked(&self) -> Result<SupabaseSession> {
        let refresh_token = self
            .current_session()
            .and_then(|s| s.refresh_token)
            .filter(|t| !t.trim().is_empty())
            .ok_or(SupabaseError::NotAuthenticated)?;
        let url = format!("{}/token?grant_type=refresh_token", self.config.auth_url());
        let body = serde_json::json!({ "refresh_token": refresh_token });
        let session: SupabaseSession = self
            .send_json_once(
                Method::POST,
                &url,
                Some(&body),
                &[],
                Some(&self.config.anon_key),
            )
            .await?;
        self.import_session(session.clone());
        self.notify_session(&session);
        Ok(session)
    }

    pub async fn refresh_session_if_needed(&self, stale_access_token: &str) -> Result<SupabaseSession> {
        let _guard = self.refresh_lock.lock().await;
        if let Some(current) = self.current_session()
            && current.access_token != stale_access_token
        {
            return Ok(current);
        }
        self.refresh_session_locked().await
    }

    pub async fn refresh_session(&self) -> Result<SupabaseSession> {
        let _guard = self.refresh_lock.lock().await;
        self.refresh_session_locked().await
    }

    pub async fn ensure_fresh_session(&self) -> Result<()> {
        let Some(sess) = self.current_session() else {
            return Ok(());
        };
        let has_refresh = sess
            .refresh_token
            .as_deref()
            .is_some_and(|t| !t.trim().is_empty());
        if !has_refresh {
            return Ok(());
        }
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if is_jwt_expiring_soon_at(&sess.access_token, now_secs, 60) {
            self.refresh_session_if_needed(&sess.access_token).await?;
        }
        Ok(())
    }

    pub async fn get_user(&self) -> Result<SupabaseUser> {
        let url = format!("{}/user", self.config.auth_url());
        self.send_json(Method::GET, &url, None, &[]).await
    }

    pub async fn sign_out(&self) -> Result<()> {
        let url = format!("{}/logout", self.config.auth_url());
        let resp = self
            .http
            .post(url)
            .header("apikey", &self.config.anon_key)
            .header("Authorization", format!("Bearer {}", self.bearer_token()))
            .send()
            .await?;
        self.clear_session();
        if resp.status() == StatusCode::NO_CONTENT || resp.status().is_success() {
            Ok(())
        } else {
            Err(SupabaseError::Api {
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            })
        }
    }

    // --- PostgREST ---

    pub async fn select<T: DeserializeOwned>(
        &self,
        table: &str,
        query: &[(&str, &str)],
    ) -> Result<Vec<T>> {
        let mut url = Url::parse(&format!("{}/{table}", self.config.rest_url()))?;
        {
            let mut pairs = url.query_pairs_mut();
            for (k, v) in query {
                pairs.append_pair(k, v);
            }
        }
        self.send_json(Method::GET, url.as_str(), None, &[]).await
    }

    pub async fn upsert<T: DeserializeOwned>(
        &self,
        table: &str,
        rows: &Value,
        on_conflict: Option<&str>,
    ) -> Result<T> {
        let mut url = Url::parse(&format!("{}/{table}", self.config.rest_url()))?;
        if let Some(conflict) = on_conflict {
            url.query_pairs_mut().append_pair("on_conflict", conflict);
        }
        self.send_json(
            Method::POST,
            url.as_str(),
            Some(rows),
            &[("Prefer", "resolution=merge-duplicates,return=representation")],
        )
        .await
    }

    pub async fn rpc<T: DeserializeOwned>(&self, function_name: &str, params: &Value) -> Result<T> {
        let url = format!("{}/rpc/{function_name}", self.config.rest_url());
        self.send_json(Method::POST, &url, Some(params), &[]).await
    }

    pub async fn pull_profiles(&self) -> Result<Vec<NuvioProfile>> {
        let url = format!("{}/rpc/sync_pull_profiles", self.config.rest_url());
        let mut profiles: Vec<NuvioProfile> =
            self.send_json(Method::POST, &url, Some(&serde_json::json!({})), &[])
                .await?;
        profiles.sort_by_key(|p| p.profile_index);
        Ok(profiles)
    }

    pub async fn pull_addons(&self, profile_id: i32) -> Result<Vec<SyncedAddonRow>> {
        let filter_val = format!("eq.{profile_id}");
        self.select(
            "addons",
            &[
                ("select", "url,name,enabled,sort_order"),
                ("profile_id", &filter_val),
                ("order", "sort_order.asc"),
            ],
        )
        .await
    }

    pub async fn push_addons(&self, profile_id: i32, addons: &[SyncedAddonRow]) -> Result<()> {
        let url = format!("{}/rpc/sync_push_addons", self.config.rest_url());
        let _: Value = self
            .send_json(
                Method::POST,
                &url,
                Some(&serde_json::json!({
                    "p_profile_id": profile_id,
                    "p_addons": addons,
                })),
                &[],
            )
            .await?;
        Ok(())
    }

    pub async fn pull_library(&self, profile_id: i32, limit: i32, offset: i32) -> Result<Vec<Value>> {
        self.rpc(
            "sync_pull_library",
            &serde_json::json!({
                "p_profile_id": profile_id,
                "p_limit": limit,
                "p_offset": offset,
            }),
        )
        .await
    }

    pub async fn pull_watch_progress(
        &self,
        profile_id: i32,
        limit: i32,
    ) -> Result<Vec<WatchProgressSyncEntry>> {
        self.rpc(
            "sync_pull_watch_progress",
            &serde_json::json!({
                "p_profile_id": profile_id,
                "p_limit": limit,
            }),
        )
        .await
    }

    pub async fn push_watch_progress(
        &self,
        profile_id: i32,
        entries: &[WatchProgressSyncEntry],
    ) -> Result<()> {
        let _: Value = self
            .rpc(
                "sync_push_watch_progress",
                &serde_json::json!({
                    "p_profile_id": profile_id,
                    "p_entries": entries,
                }),
            )
            .await?;
        Ok(())
    }

    pub async fn delete_watch_progress(
        &self,
        profile_id: i32,
        progress_keys: &[String],
    ) -> Result<()> {
        let _: Value = self
            .rpc(
                "sync_delete_watch_progress",
                &serde_json::json!({
                    "p_profile_id": profile_id,
                    "p_keys": progress_keys,
                }),
            )
            .await?;
        Ok(())
    }

    pub async fn push_library_items(&self, profile_id: i32, items: &[Value]) -> Result<()> {
        let _: Value = self
            .rpc(
                "sync_push_library_items",
                &serde_json::json!({
                    "p_profile_id": profile_id,
                    "p_items": items,
                }),
            )
            .await?;
        Ok(())
    }

    pub async fn delete_library_items(
        &self,
        profile_id: i32,
        keys: &[Value],
    ) -> Result<()> {
        let _: Value = self
            .rpc(
                "sync_delete_library_items",
                &serde_json::json!({
                    "p_profile_id": profile_id,
                    "p_keys": keys,
                }),
            )
            .await?;
        Ok(())
    }

    pub async fn pull_watched_items(
        &self,
        profile_id: i32,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<SyncedWatchedItem>> {
        self.rpc(
            "sync_pull_watched_items",
            &serde_json::json!({
                "p_profile_id": profile_id,
                "p_page": page,
                "p_page_size": page_size,
            }),
        )
        .await
    }

    pub async fn push_watched_items(
        &self,
        profile_id: i32,
        items: &[SyncedWatchedItem],
    ) -> Result<()> {
        let _: Value = self
            .rpc(
                "sync_push_watched_items",
                &serde_json::json!({
                    "p_profile_id": profile_id,
                    "p_items": items,
                }),
            )
            .await?;
        Ok(())
    }

    pub async fn delete_watched_items(&self, profile_id: i32, keys: &[Value]) -> Result<()> {
        let _: Value = self
            .rpc(
                "sync_delete_watched_items",
                &serde_json::json!({
                    "p_profile_id": profile_id,
                    "p_keys": keys,
                }),
            )
            .await?;
        Ok(())
    }

    pub async fn pull_plugins(&self, profile_id: i32) -> Result<Vec<SyncedPluginRow>> {
        let filter_val = format!("eq.{profile_id}");
        self.select(
            "plugins",
            &[
                ("select", "url,name,enabled,sort_order"),
                ("profile_id", &filter_val),
                ("order", "sort_order.asc"),
            ],
        )
        .await
    }

    pub async fn push_plugins(&self, profile_id: i32, plugins: &[SyncedPluginRow]) -> Result<()> {
        let _: Value = self
            .rpc(
                "sync_push_plugins",
                &serde_json::json!({
                    "p_profile_id": profile_id,
                    "p_plugins": plugins,
                }),
            )
            .await?;
        Ok(())
    }

    pub async fn start_device_login_session(
        &self,
        nonce: &str,
        redirect_base_url: &str,
        device_name: &str,
    ) -> Result<Option<DeviceLinkStart>> {
        let rows: Vec<DeviceLinkStart> = self
            .rpc(
                "start_device_login_session",
                &serde_json::json!({
                    "p_device_nonce": nonce,
                    "p_redirect_base_url": redirect_base_url,
                    "p_device_name": device_name,
                    "p_device_type": "mobile"
                }),
            )
            .await?;
        Ok(rows.into_iter().next())
    }

    pub async fn poll_tv_login_session(
        &self,
        device_code: &str,
        nonce: &str,
    ) -> Result<Option<DeviceLinkPoll>> {
        let rows: Vec<DeviceLinkPoll> = self
            .rpc(
                "poll_tv_login_session",
                &serde_json::json!({
                    "p_device_code": device_code,
                    "p_device_nonce": nonce
                }),
            )
            .await?;
        Ok(rows.into_iter().next())
    }

    pub async fn exchange_tv_login(
        &self,
        device_code: &str,
        nonce: &str,
    ) -> Result<SupabaseSession> {
        let exchanged: DeviceLinkExchange = self
            .invoke_function(
                "tv-logins-exchange",
                Some(&serde_json::json!({
                    "code": device_code,
                    "device_nonce": nonce
                })),
            )
            .await?;
        let mut session = SupabaseSession {
            access_token: exchanged.access_token,
            refresh_token: exchanged.refresh_token,
            token_type: exchanged.token_type.or_else(|| Some("bearer".to_string())),
            expires_in: exchanged.expires_in,
            user: exchanged.user,
        };
        self.import_session(session.clone());
        if session.user.is_none()
            && let Ok(user) = self.get_user().await
        {
            session.user = Some(user);
            self.import_session(session.clone());
        }
        self.notify_session(&session);
        Ok(session)
    }

    // --- Edge Functions ---

    pub async fn invoke_function<T: DeserializeOwned>(
        &self,
        name: &str,
        payload: Option<&Value>,
    ) -> Result<T> {
        let url = format!("{}/{name}", self.config.functions_url());
        self.send_json(Method::POST, &url, payload, &[]).await
    }

    // --- Storage ---

    async fn upload_object_once(
        &self,
        url: &str,
        bytes: Vec<u8>,
        content_type: &str,
        token: &str,
    ) -> Result<Value> {
        let resp = self
            .http
            .post(url)
            .header("apikey", &self.config.anon_key)
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", content_type)
            .header("x-upsert", "true")
            .body(bytes)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(SupabaseError::Api {
                status: status.as_u16(),
                body: text,
            });
        }
        Ok(serde_json::from_str(&text)?)
    }

    pub async fn upload_object(
        &self,
        bucket: &str,
        object_path: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<Value> {
        let url = format!(
            "{}/object/{}/{}",
            self.config.storage_url(),
            bucket.trim_matches('/'),
            object_path.trim_start_matches('/')
        );
        let used_token = self.bearer_token();
        match self
            .upload_object_once(&url, bytes.clone(), content_type, &used_token)
            .await
        {
            Ok(v) => Ok(v),
            Err(err) if is_jwt_expired_error(&err) && self.has_refresh_token() => {
                self.refresh_session_if_needed(&used_token).await?;
                let fresh = self.bearer_token();
                self.upload_object_once(&url, bytes, content_type, &fresh)
                    .await
            }
            Err(err) => Err(err),
        }
    }

    async fn download_object_once(&self, url: &str, token: &str) -> Result<Vec<u8>> {
        let resp = self
            .http
            .get(url)
            .header("apikey", &self.config.anon_key)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(SupabaseError::Api {
                status: status.as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }
        Ok(resp.bytes().await?.to_vec())
    }

    pub async fn download_object(&self, bucket: &str, object_path: &str) -> Result<Vec<u8>> {
        let url = format!(
            "{}/object/{}/{}",
            self.config.storage_url(),
            bucket.trim_matches('/'),
            object_path.trim_start_matches('/')
        );
        let used_token = self.bearer_token();
        match self.download_object_once(&url, &used_token).await {
            Ok(bytes) => Ok(bytes),
            Err(err) if is_jwt_expired_error(&err) && self.has_refresh_token() => {
                self.refresh_session_if_needed(&used_token).await?;
                let fresh = self.bearer_token();
                self.download_object_once(&url, &fresh).await
            }
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_supabase_endpoints_and_ws_url() {
        let cfg = SupabaseConfig::new("https://example.supabase.co/", "anon123");
        assert_eq!(cfg.rest_url(), "https://example.supabase.co/rest/v1");
        assert_eq!(
            cfg.realtime_ws_url().unwrap(),
            "wss://example.supabase.co/realtime/v1/websocket?apikey=anon123&vsn=1.0.0&nuvio_realtime=1"
        );
    }

    #[test]
    fn detects_jwt_expiration_and_pgrst303() {
        // {"alg":"HS256","typ":"JWT"}.{"sub":"u1","exp":2000000000}.sig
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1MSIsImV4cCI6MjAwMDAwMDAwMH0.sig";
        assert_eq!(jwt_exp_unix(token), Some(2_000_000_000));
        assert!(!is_jwt_expiring_soon_at(token, 1_999_999_900, 60));
        assert!(is_jwt_expiring_soon_at(token, 1_999_999_950, 60));
        assert!(is_jwt_expiring_soon_at(token, 2_000_000_010, 60));

        let err = SupabaseError::Api {
            status: 401,
            body: r#"{"code":"PGRST303","details":null,"hint":null,"message":"JWT expired"}"#
                .to_string(),
        };
        assert!(is_jwt_expired_error(&err));
        let other = SupabaseError::Api {
            status: 403,
            body: "forbidden".to_string(),
        };
        assert!(!is_jwt_expired_error(&other));
    }
}
