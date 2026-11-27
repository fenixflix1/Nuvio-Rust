//! Watch party over the Nuvio backend's `watch_party_*` RPCs.
//!
//! The backend keeps the party record (media, members, host, last playback state)
//! and every call answers a snapshot. Polling `watch_party_heartbeat` keeps the
//! membership alive and doubles as the resync source, so a dropped poll cannot
//! leave two devices drifting apart without either of them noticing.

use nuvio_core::supabase::SupabaseClient;
use nuvio_core::supabase::realtime::RealtimeConnection;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{AppState, ensure_supabase_client};

const POLL_MS: u64 = 2500;
const COORDINATOR_MS: u64 = 450;
const REALTIME_HEARTBEAT_SECS: u64 = 25;
const CHAT_LIMIT: usize = 200;

fn realtime_topic(party_id: &str) -> String {
    format!("realtime:watch-party:{party_id}")
}

fn is_waiting_status(status: &str) -> bool {
    matches!(status, "choosing" | "loading" | "buffering")
}

fn wire_status(status: &str) -> &str {
    match status {
        "choosing" => "browsing",
        "loading" | "buffering" | "ready" | "behind" | "browsing" => status,
        _ => "browsing",
    }
}

fn normalize_incoming_status(raw_status: &str, incoming_media: &str, party_media: &str) -> String {
    if raw_status == "browsing"
        && !incoming_media.is_empty()
        && media_keys_compatible(party_media, incoming_media)
    {
        "choosing".to_string()
    } else {
        raw_status.to_string()
    }
}

fn media_keys_compatible(expected: &str, candidate: &str) -> bool {
    if expected.is_empty() || candidate.is_empty() || expected == candidate {
        return true;
    }
    let e: Vec<&str> = expected.split('|').collect();
    let c: Vec<&str> = candidate.split('|').collect();
    if e.len() >= 4 && c.len() >= 4 {
        let same_type = e[0].eq_ignore_ascii_case(c[0])
            || (matches!(e[0], "series" | "tv") && matches!(c[0], "series" | "tv"));
        if !same_type || e[2] != c[2] || e[3] != c[3] {
            return false;
        }
        // If both IDs share the same scheme (e.g. both IMDb or both TMDB), require exact match;
        // if one is IMDb and the other is TMDB/addon for the active party room, allow syncing.
        let e_tmdb = e[1].starts_with("tmdb:");
        let c_tmdb = c[1].starts_with("tmdb:");
        let e_imdb = e[1].starts_with("tt");
        let c_imdb = c[1].starts_with("tt");
        if (e_tmdb && c_imdb) || (e_imdb && c_tmdb) {
            return true;
        }
    }
    false
}

fn party_media_key_from_value(media: &Value) -> String {
    let meta_type = media
        .get("meta_type")
        .or_else(|| media.get("type"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_lowercase();
    let meta_id = media
        .get("meta_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let season = media
        .get("season")
        .and_then(Value::as_i64)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let episode = media
        .get("episode")
        .and_then(Value::as_i64)
        .map(|v| v.to_string())
        .unwrap_or_default();
    format!("{meta_type}|{meta_id}|{season}|{episode}")
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PartyMemberView {
    pub id: String,
    pub name: String,
    pub color: String,
    pub avatar: String,
    pub is_host: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PartyPlaybackView {
    pub seq: i64,
    pub member_id: String,
    pub position_ms: i64,
    pub paused: bool,
    pub rate: f64,
    pub media_key: String,
    pub waiting: bool,
    /// Server clock, so a guest can work out how stale the position is.
    pub at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PartyChatMessageView {
    pub key: String,
    pub id: Option<i64>,
    pub member_id: String,
    pub name: String,
    pub avatar_url: String,
    pub avatar_color: String,
    pub body: String,
    pub sent_at: i64,
    pub mine: bool,
    pub sent: bool,
    pub failed: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PartyView {
    pub active: bool,
    pub connected: bool,
    pub code: String,
    pub member_id: String,
    pub host_member_id: String,
    pub is_host: bool,
    pub can_control: bool,
    pub host_only: bool,
    pub max_members: i64,
    pub members: Vec<PartyMemberView>,
    pub title: String,
    pub media: Option<Value>,
    pub waiting_for: Vec<String>,
    pub playback: Option<PartyPlaybackView>,
    pub chat_ready: bool,
    pub chat_unread: usize,
    pub chat_messages: Vec<PartyChatMessageView>,
    /// `server_clock - local_clock`; added to `Date.now()` to read server time.
    pub server_offset_ms: i64,
    pub error: String,
}

#[derive(Default)]
struct Inner {
    active: bool,
    connected: bool,
    party_id: String,
    device_id: String,
    code: String,
    member_id: String,
    host_member_id: String,
    host_only: bool,
    max_members: i64,
    title: String,
    media_key: String,
    media: Option<Value>,
    members: Vec<PartyMemberView>,
    member_statuses: HashMap<String, String>,
    self_status: String,
    self_media_key: String,
    was_playing_before_wait: bool,
    playback: Option<PartyPlaybackView>,
    playback_seq: i64,
    chat_ready: bool,
    chat_open: bool,
    chat_unread: usize,
    chat_messages: Vec<PartyChatMessageView>,
    server_offset_ms: i64,
    clock_calibrated: bool,
    error: String,
}

#[derive(Default)]
pub struct PartyRuntime {
    inner: Mutex<Inner>,
    generation: AtomicU64,
    realtime: Mutex<Option<Arc<RealtimeConnection>>>,
    persist_gate: Arc<tokio::sync::Mutex<()>>,
}

impl PartyRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn view(&self) -> PartyView {
        let inner = self.inner.lock().expect("party lock");
        let waiting_for: Vec<String> = inner
            .members
            .iter()
            .filter(|m| is_waiting_status(&m.status))
            .map(|m| m.name.clone())
            .collect();
        PartyView {
            active: inner.active,
            connected: inner.connected,
            code: inner.code.clone(),
            member_id: inner.member_id.clone(),
            host_member_id: inner.host_member_id.clone(),
            is_host: inner.active
                && !inner.member_id.is_empty()
                && inner.member_id == inner.host_member_id,
            can_control: inner.active
                && (!inner.host_only
                    || (!inner.member_id.is_empty() && inner.member_id == inner.host_member_id)),
            host_only: inner.host_only,
            max_members: inner.max_members,
            members: inner.members.clone(),
            title: inner.title.clone(),
            media: inner.media.clone(),
            waiting_for,
            playback: inner.playback.clone(),
            chat_ready: inner.chat_ready,
            chat_unread: inner.chat_unread,
            chat_messages: inner.chat_messages.clone(),
            server_offset_ms: inner.server_offset_ms,
            error: inner.error.clone(),
        }
    }

    fn stop(&self, error: &str) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        let prev_rt = self.realtime.lock().expect("rt lock").take();
        if let Some(rt) = prev_rt {
            let party_id = self.inner.lock().expect("party lock").party_id.clone();
            if !party_id.is_empty()
                && let Ok(handle) = tokio::runtime::Handle::try_current()
            {
                handle.spawn(async move {
                    let _ = rt.leave_channel(&realtime_topic(&party_id)).await;
                });
            }
        }
        let mut inner = self.inner.lock().expect("party lock");
        *inner = Inner {
            error: error.to_string(),
            ..Inner::default()
        };
    }

    fn can_control(&self) -> bool {
        let inner = self.inner.lock().expect("party lock");
        let is_host = !inner.member_id.is_empty() && inner.member_id == inner.host_member_id;
        inner.active && (!inner.host_only || is_host)
    }

    fn is_control_locked(&self) -> bool {
        let inner = self.inner.lock().expect("party lock");
        let is_host = !inner.member_id.is_empty() && inner.member_id == inner.host_member_id;
        let is_waiting = inner.members.iter().any(|m| is_waiting_status(&m.status))
            || inner.playback.as_ref().is_some_and(|p| p.waiting);
        inner.active && ((inner.host_only && !is_host) || is_waiting)
    }

    fn emit_view(&self, app: &AppHandle) -> PartyView {
        app.state::<AppState>()
            .player
            .set_party_locked(self.is_control_locked());
        let view = self.view();
        let _ = app.emit("party:state", view.clone());
        view
    }

    fn set_device_id(&self, device_id: &str) {
        self.inner.lock().expect("party lock").device_id = device_id.to_string();
    }

    fn apply(&self, snapshot: &Value) -> bool {
        let Some(party) = snapshot.get("party").filter(|v| v.is_object()) else {
            return false;
        };
        let mut inner = self.inner.lock().expect("party lock");
        inner.active = true;
        inner.connected = true;
        inner.error.clear();
        if let Some(code) = party.get("code").and_then(Value::as_str) {
            inner.code = code.to_string();
        }
        if let Some(id) = party.get("id").and_then(Value::as_str) {
            inner.party_id = id.to_string();
        }
        if let Some(member) = snapshot.get("member_id").and_then(Value::as_str) {
            inner.member_id = member.to_string();
        }
        if let Some(host) = party.get("host_member_id").and_then(Value::as_str) {
            inner.host_member_id = host.to_string();
        }
        inner.host_only = party
            .get("host_only")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        inner.max_members = party
            .get("max_members")
            .and_then(Value::as_i64)
            .unwrap_or(3);
        if let Some(title) = party
            .pointer("/media/title")
            .and_then(Value::as_str)
            .filter(|t| !t.trim().is_empty())
        {
            inner.title = title.to_string();
        }
        if let Some(media) = party.get("media").filter(|v| v.is_object()) {
            inner.media = Some(media.clone());
            let key = party_media_key_from_value(media);
            if !key.trim_matches('|').is_empty() {
                inner.media_key = key;
            }
        }
        if !inner.clock_calibrated
            && let Some(server) = snapshot.get("server_time_ms").and_then(Value::as_i64)
        {
            inner.server_offset_ms = server - now_ms();
        }
        let host_id = inner.host_member_id.clone();
        let self_id = inner.member_id.clone();
        let self_status = inner.self_status.clone();
        inner.members = snapshot
            .get("members")
            .or_else(|| party.get("members"))
            .and_then(Value::as_array)
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| {
                        let id = row.get("id").and_then(Value::as_str)?.to_string();
                        let status = if id == self_id && !self_status.is_empty() {
                            self_status.clone()
                        } else {
                            inner
                                .member_statuses
                                .get(&id)
                                .cloned()
                                .unwrap_or_else(|| "browsing".into())
                        };
                        Some(PartyMemberView {
                            is_host: false,
                            name: row
                                .get("display_name")
                                .and_then(Value::as_str)
                                .unwrap_or("Convidado")
                                .to_string(),
                            color: row
                                .get("avatar_color")
                                .and_then(Value::as_str)
                                .unwrap_or("#1E88E5")
                                .to_string(),
                            avatar: row
                                .get("avatar_url")
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_string(),
                            id,
                            status,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        for member in &mut inner.members {
            member.is_host = member.id == host_id;
        }
        let member_ids: std::collections::HashSet<String> =
            inner.members.iter().map(|m| m.id.clone()).collect();
        inner.member_statuses.retain(|id, _| member_ids.contains(id));
        if let Some(playback) = party.get("playback").filter(|v| v.is_object()) {
            let incoming_seq = playback.get("seq").and_then(Value::as_i64).unwrap_or(0);
            let incoming_member = playback
                .get("member_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let current_seq = inner.playback.as_ref().map(|p| p.seq).unwrap_or(-1);
            let same_seq_higher_member = incoming_seq == current_seq
                && incoming_member.as_str()
                    > inner
                        .playback
                        .as_ref()
                        .map(|p| p.member_id.as_str())
                        .unwrap_or("");
            if inner.playback.is_none() || incoming_seq > current_seq || same_seq_higher_member {
                if incoming_seq > inner.playback_seq {
                    inner.playback_seq = incoming_seq;
                }
                inner.playback = Some(PartyPlaybackView {
                    seq: incoming_seq,
                    member_id: incoming_member,
                    position_ms: playback
                        .get("position_ms")
                        .and_then(Value::as_i64)
                        .unwrap_or(0),
                    paused: playback
                        .get("paused")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    rate: playback.get("rate").and_then(Value::as_f64).unwrap_or(1.0),
                    media_key: playback
                        .get("media_key")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    waiting: playback
                        .get("waiting")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    at_ms: playback.get("at").and_then(Value::as_i64).unwrap_or(0),
                });
            }
        }
        true
    }

    fn set_local_playback(
        &self,
        position_ms: i64,
        paused: bool,
        rate: f64,
        media_key: &str,
        waiting: bool,
    ) -> Value {
        let mut inner = self.inner.lock().expect("party lock");
        let any_waiting = inner.members.iter().any(|m| is_waiting_status(&m.status));
        let final_paused = if any_waiting { true } else { paused };
        let final_waiting = if any_waiting { true } else { waiting };
        if !final_waiting {
            inner.was_playing_before_wait = !final_paused;
        }
        let offset = inner.server_offset_ms;
        let next_seq = inner.playback_seq + 1;
        inner.playback_seq = next_seq;
        let member_id = inner.member_id.clone();
        let clamped_pos = position_ms.max(0);
        let clamped_rate = rate.clamp(0.25, 4.0);
        let at_ms = now_ms() + offset;
        inner.playback = Some(PartyPlaybackView {
            seq: next_seq,
            member_id: member_id.clone(),
            position_ms: clamped_pos,
            paused: final_paused,
            rate: clamped_rate,
            media_key: media_key.to_string(),
            waiting: final_waiting,
            at_ms,
        });
        json!({
            "seq": next_seq,
            "member_id": member_id,
            "paused": final_paused,
            "position_ms": clamped_pos,
            "rate": clamped_rate,
            "at": at_ms,
            "media_key": media_key,
            "waiting": final_waiting,
        })
    }

    fn current_playback_json(&self) -> Option<Value> {
        let inner = self.inner.lock().expect("party lock");
        let p = inner.playback.as_ref()?;
        Some(json!({
            "seq": p.seq,
            "member_id": p.member_id,
            "paused": p.paused,
            "position_ms": p.position_ms,
            "rate": p.rate,
            "at": p.at_ms,
            "media_key": p.media_key,
            "waiting": p.waiting,
        }))
    }

    fn apply_realtime_playback(&self, playback: &Value) -> bool {
        let mut inner = self.inner.lock().expect("party lock");
        if !inner.active || !playback.is_object() {
            return false;
        }
        let incoming_media = playback
            .get("media_key")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !media_keys_compatible(&inner.media_key, incoming_media) {
            return false;
        }
        let incoming_member = playback
            .get("member_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if inner.host_only && !inner.host_member_id.is_empty() && incoming_member != inner.host_member_id {
            return false;
        }
        let Some(incoming_seq) = playback.get("seq").and_then(Value::as_i64) else {
            return false;
        };
        let current_seq = inner.playback.as_ref().map(|p| p.seq).unwrap_or(-1);
        let current_member = inner
            .playback
            .as_ref()
            .map(|p| p.member_id.as_str())
            .unwrap_or("");
        let newer = inner.playback.is_none()
            || incoming_seq > current_seq
            || (incoming_seq == current_seq && incoming_member.as_str() > current_member);
        if !newer {
            return false;
        }
        if incoming_seq > inner.playback_seq {
            inner.playback_seq = incoming_seq;
        }
        inner.playback = Some(PartyPlaybackView {
            seq: incoming_seq,
            member_id: incoming_member,
            position_ms: playback
                .get("position_ms")
                .and_then(Value::as_i64)
                .unwrap_or(0)
                .max(0),
            paused: playback
                .get("paused")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            rate: playback
                .get("rate")
                .and_then(Value::as_f64)
                .unwrap_or(1.0)
                .clamp(0.25, 4.0),
            media_key: incoming_media.to_string(),
            waiting: playback
                .get("waiting")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            at_ms: playback.get("at").and_then(Value::as_i64).unwrap_or(0),
        });
        true
    }

    fn apply_realtime_status(&self, payload: &Value) -> bool {
        let Some(member_id) = payload.get("member_id").and_then(Value::as_str) else {
            return false;
        };
        let Some(raw_status) = payload.get("status").and_then(Value::as_str) else {
            return false;
        };
        let incoming_media = payload
            .get("media_key")
            .and_then(Value::as_str)
            .unwrap_or("");
        let mut inner = self.inner.lock().expect("party lock");
        if !inner.active || member_id.is_empty() {
            return false;
        }
        let status = normalize_incoming_status(raw_status, incoming_media, &inner.media_key);
        let changed = inner.member_statuses.get(member_id) != Some(&status);
        inner.member_statuses.insert(member_id.to_string(), status.clone());
        for m in &mut inner.members {
            if m.id == member_id {
                m.status = status.clone();
            }
        }
        changed
    }

    fn apply_presence_joins(&self, map: &serde_json::Map<String, Value>) -> bool {
        let mut inner = self.inner.lock().expect("party lock");
        if !inner.active {
            return false;
        }
        let mut changed = false;
        let self_id = inner.member_id.clone();
        let party_media = inner.media_key.clone();
        for (key, entry) in map {
            let meta = entry
                .get("metas")
                .and_then(Value::as_array)
                .and_then(|metas| metas.last())
                .filter(|v| v.is_object());
            let member_id = meta
                .and_then(|m| m.get("member_id").and_then(Value::as_str))
                .filter(|s| !s.is_empty())
                .unwrap_or(key.as_str());
            if member_id.is_empty() || member_id == self_id {
                continue;
            }
            let raw_status = meta
                .and_then(|m| m.get("status").and_then(Value::as_str))
                .unwrap_or("browsing");
            let incoming_media = meta
                .and_then(|m| m.get("media_key").and_then(Value::as_str))
                .unwrap_or("");
            let status = normalize_incoming_status(raw_status, incoming_media, &party_media);
            if inner.member_statuses.get(member_id) != Some(&status) {
                inner.member_statuses.insert(member_id.to_string(), status.clone());
                for m in &mut inner.members {
                    if m.id == member_id {
                        m.status = status.clone();
                    }
                }
                changed = true;
            }
        }
        changed
    }

    fn apply_presence_state(&self, payload: &Value) -> bool {
        payload
            .as_object()
            .is_some_and(|map| self.apply_presence_joins(map))
    }

    fn apply_presence_diff(&self, payload: &Value) -> bool {
        let mut changed = false;
        if let Some(leaves) = payload.get("leaves").and_then(Value::as_object) {
            let joins = payload.get("joins").and_then(Value::as_object);
            let mut inner = self.inner.lock().expect("party lock");
            if inner.active {
                let self_id = inner.member_id.clone();
                for left_id in leaves.keys() {
                    if left_id == &self_id || joins.is_some_and(|j| j.contains_key(left_id)) {
                        continue;
                    }
                    if inner.member_statuses.remove(left_id).is_some() {
                        for m in &mut inner.members {
                            if &m.id == left_id {
                                m.status = "browsing".to_string();
                            }
                        }
                        changed = true;
                    }
                }
            }
        }
        if let Some(joins) = payload.get("joins").and_then(Value::as_object) {
            changed |= self.apply_presence_joins(joins);
        }
        changed
    }

    fn current_presence_payload(&self) -> Option<Value> {
        let inner = self.inner.lock().expect("party lock");
        if !inner.active || inner.member_id.is_empty() {
            return None;
        }
        let status = if inner.self_status.is_empty() {
            "browsing"
        } else {
            wire_status(&inner.self_status)
        };
        let mk = if !inner.self_media_key.is_empty() {
            Some(inner.self_media_key.as_str())
        } else if status != "browsing" && !inner.media_key.is_empty() {
            Some(inner.media_key.as_str())
        } else {
            None
        };
        Some(json!({
            "member_id": inner.member_id,
            "status": status,
            "media_key": mk,
            "behind_ms": Value::Null,
        }))
    }

    fn set_self_status(&self, status: &str, media_key: Option<&str>) -> Option<Value> {
        let mut inner = self.inner.lock().expect("party lock");
        if !inner.active || inner.member_id.is_empty() {
            return None;
        }
        let changed = inner.self_status != status
            || media_key.is_some_and(|k| k != inner.self_media_key);
        inner.self_status = status.to_string();
        if let Some(mk) = media_key {
            inner.self_media_key = mk.to_string();
        }
        let self_id = inner.member_id.clone();
        inner
            .member_statuses
            .insert(self_id.clone(), status.to_string());
        for m in &mut inner.members {
            if m.id == self_id {
                m.status = status.to_string();
            }
        }
        if !changed {
            return None;
        }
        let wire = wire_status(status);
        let mk = if !inner.self_media_key.is_empty() {
            Some(inner.self_media_key.as_str())
        } else if wire != "browsing" && !inner.media_key.is_empty() {
            Some(inner.media_key.as_str())
        } else {
            None
        };
        Some(json!({
            "member_id": self_id,
            "status": wire,
            "media_key": mk,
            "behind_ms": Value::Null,
        }))
    }

    fn evaluate_waiting_coordinator(&self) -> Option<Value> {
        let mut inner = self.inner.lock().expect("party lock");
        let is_host = !inner.member_id.is_empty() && inner.member_id == inner.host_member_id;
        if !inner.active || !is_host {
            return None;
        }
        let anyone_waiting = inner.members.iter().any(|m| is_waiting_status(&m.status));
        let now_srv = now_ms() + inner.server_offset_ms;
        let current = inner.playback.clone();
        if anyone_waiting {
            if let Some(ref pb) = current {
                if pb.paused && pb.waiting {
                    return None;
                }
                if !pb.paused {
                    inner.was_playing_before_wait = true;
                }
                let elapsed = if pb.paused {
                    0
                } else {
                    ((now_srv - pb.at_ms).max(0) as f64 * pb.rate).round() as i64
                };
                let pos_ms = (pb.position_ms + elapsed).max(0);
                let rate = pb.rate;
                let media_key = if pb.media_key.is_empty() {
                    inner.media_key.clone()
                } else {
                    pb.media_key.clone()
                };
                let next_seq = inner.playback_seq + 1;
                inner.playback_seq = next_seq;
                let member_id = inner.member_id.clone();
                inner.playback = Some(PartyPlaybackView {
                    seq: next_seq,
                    member_id: member_id.clone(),
                    position_ms: pos_ms,
                    paused: true,
                    rate,
                    media_key: media_key.clone(),
                    waiting: true,
                    at_ms: now_srv,
                });
                return Some(json!({
                    "seq": next_seq,
                    "member_id": member_id,
                    "paused": true,
                    "position_ms": pos_ms,
                    "rate": rate,
                    "at": now_srv,
                    "media_key": media_key,
                    "waiting": true,
                }));
            }
            return None;
        }
        if let Some(ref pb) = current
            && pb.waiting
        {
            let should_resume = inner.was_playing_before_wait;
            inner.was_playing_before_wait = false;
            let next_seq = inner.playback_seq + 1;
            inner.playback_seq = next_seq;
            let member_id = inner.member_id.clone();
            let pos_ms = pb.position_ms;
            let rate = pb.rate;
            let media_key = if pb.media_key.is_empty() {
                inner.media_key.clone()
            } else {
                pb.media_key.clone()
            };
            inner.playback = Some(PartyPlaybackView {
                seq: next_seq,
                member_id: member_id.clone(),
                position_ms: pos_ms,
                paused: !should_resume,
                rate,
                media_key: media_key.clone(),
                waiting: false,
                at_ms: now_srv,
            });
            return Some(json!({
                "seq": next_seq,
                "member_id": member_id,
                "paused": !should_resume,
                "position_ms": pos_ms,
                "rate": rate,
                "at": now_srv,
                "media_key": media_key,
                "waiting": false,
            }));
        }
        None
    }

    fn apply_realtime_control(&self, payload: &Value) -> bool {
        let Some(host_only) = payload.get("host_only").and_then(Value::as_bool) else {
            return false;
        };
        let mut inner = self.inner.lock().expect("party lock");
        if !inner.active || inner.host_only == host_only {
            return false;
        }
        inner.host_only = host_only;
        true
    }

    fn apply_realtime_member_left(
        &self,
        member_id: &str,
        host_member_id: Option<&str>,
    ) -> bool {
        let mut inner = self.inner.lock().expect("party lock");
        if !inner.active || member_id.is_empty() {
            return false;
        }
        if member_id == inner.member_id {
            return false;
        }
        let before = inner.members.len();
        inner.members.retain(|m| m.id != member_id);
        if let Some(new_host) = host_member_id.filter(|h| !h.is_empty()) {
            inner.host_member_id = new_host.to_string();
        }
        let host_id = inner.host_member_id.clone();
        for m in &mut inner.members {
            m.is_host = m.id == host_id;
        }
        inner.members.len() != before || host_member_id.is_some()
    }

    fn broadcast(&self, event: &str, payload: Value) {
        let rt = self.realtime.lock().expect("rt lock").clone();
        let party_id = self.inner.lock().expect("party lock").party_id.clone();
        if let (Some(rt), false) = (rt, party_id.is_empty())
            && let Ok(handle) = tokio::runtime::Handle::try_current()
        {
            let event = event.to_string();
            handle.spawn(async move {
                let _ = rt.broadcast(&realtime_topic(&party_id), &event, payload).await;
            });
        }
    }

    fn track_presence(&self, payload: Value) {
        let rt = self.realtime.lock().expect("rt lock").clone();
        let party_id = self.inner.lock().expect("party lock").party_id.clone();
        if let (Some(rt), false) = (rt, party_id.is_empty())
            && let Ok(handle) = tokio::runtime::Handle::try_current()
        {
            handle.spawn(async move {
                let _ = rt.track_presence(&realtime_topic(&party_id), payload).await;
            });
        }
    }

    fn apply_chat_messages(&self, rows: &[Value], notify_unread: bool) {
        let mut inner = self.inner.lock().expect("party lock");
        let self_member = inner.member_id.clone();
        let mut confirmed: std::collections::BTreeMap<i64, PartyChatMessageView> = inner
            .chat_messages
            .iter()
            .filter_map(|m| m.id.map(|id| (id, m.clone())))
            .collect();
        let mut pending: Vec<PartyChatMessageView> = inner
            .chat_messages
            .iter()
            .filter(|m| m.id.is_none())
            .cloned()
            .collect();
        let mut newly_added_other = 0usize;

        for row in rows {
            let Some(id) = row.get("id").and_then(Value::as_i64) else {
                continue;
            };
            if confirmed.contains_key(&id) {
                continue;
            }
            let member_id = row
                .get("member_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let mine = row
                .get("mine")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                || (!self_member.is_empty() && member_id == self_member);
            let client_id = row
                .get("client_id")
                .and_then(Value::as_str)
                .map(str::to_string);
            let mut msg = PartyChatMessageView {
                key: format!("id:{id}"),
                id: Some(id),
                member_id,
                name: row
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                avatar_url: row
                    .get("avatar_url")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                avatar_color: row
                    .get("avatar_color")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                body: row
                    .get("body")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                sent_at: row.get("sent_at").and_then(Value::as_i64).unwrap_or(0),
                mine,
                sent: true,
                failed: false,
            };
            let local_pos = if mine {
                client_id
                    .as_deref()
                    .and_then(|cid| pending.iter().position(|p| p.key == cid))
            } else {
                None
            };
            if let Some(pos) = local_pos {
                let local = pending.remove(pos);
                msg.key = local.key;
            } else if !mine {
                newly_added_other += 1;
            }
            confirmed.insert(id, msg);
        }

        let mut merged: Vec<PartyChatMessageView> = confirmed.into_values().collect();
        merged.extend(pending);
        if merged.len() > CHAT_LIMIT {
            let drain = merged.len() - CHAT_LIMIT;
            merged.drain(0..drain);
        }
        inner.chat_messages = merged;
        inner.chat_ready = true;
        if notify_unread && !inner.chat_open && newly_added_other > 0 {
            inner.chat_unread += newly_added_other;
        }
    }

    /// Polls the party and emits `party:state` after every snapshot.
    fn start(self: &Arc<Self>, client: SupabaseClient, app: AppHandle) {
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let runtime = Arc::clone(self);
        let poll_client = client.clone();
        let poll_app = app.clone();
        tokio::spawn(async move {
            let client = poll_client;
            let app = poll_app;
            let (party_id, device_id) = {
                let inner = runtime.inner.lock().expect("party lock");
                (inner.party_id.clone(), inner.device_id.clone())
            };
            if !party_id.is_empty()
                && !device_id.is_empty()
                && let Ok(history) = rpc(
                    &client,
                    "watch_party_list_messages",
                    json!({ "p_party_id": party_id, "p_device_id": device_id }),
                )
                .await
                && history.get("error").is_none_or(Value::is_null)
            {
                let rows = history
                    .get("messages")
                    .and_then(Value::as_array)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                runtime.apply_chat_messages(rows, false);
                runtime.emit_view(&app);
            }

            let start_at = tokio::time::Instant::now() + Duration::from_millis(POLL_MS);
            let mut ticker = tokio::time::interval_at(start_at, Duration::from_millis(POLL_MS));
            loop {
                ticker.tick().await;
                if runtime.generation.load(Ordering::SeqCst) != generation {
                    break;
                }
                let (party_id, device_id, chat_ready, after_id) = {
                    let inner = runtime.inner.lock().expect("party lock");
                    let last = if inner.chat_ready {
                        inner.chat_messages.iter().rev().find_map(|m| m.id)
                    } else {
                        None
                    };
                    (
                        inner.party_id.clone(),
                        inner.device_id.clone(),
                        inner.chat_ready,
                        last,
                    )
                };
                if party_id.is_empty() || device_id.is_empty() {
                    break;
                }
                let snapshot = rpc(
                    &client,
                    "watch_party_heartbeat",
                    json!({ "p_party_id": party_id, "p_device_id": device_id }),
                )
                .await;
                match snapshot {
                    Ok(snapshot) => {
                        let fatal = snapshot
                            .get("error")
                            .and_then(Value::as_str)
                            .filter(|code| {
                                matches!(
                                    *code,
                                    "party_ended" | "removed" | "not_member" | "party_not_found"
                                )
                            });
                        if let Some(code) = fatal {
                            let message = party_error_message(code);
                            runtime.stop(&message);
                            runtime.emit_view(&app);
                            break;
                        }
                        if !runtime.apply(&snapshot) {
                            runtime.stop("A sala foi encerrada.");
                            runtime.emit_view(&app);
                            break;
                        }
                        let mut chat_params = json!({
                            "p_party_id": party_id,
                            "p_device_id": device_id
                        });
                        if let Some(after) = after_id {
                            chat_params["p_after"] = json!(after);
                        }
                        if let Ok(history) =
                            rpc(&client, "watch_party_list_messages", chat_params).await
                            && history.get("error").is_none_or(Value::is_null)
                        {
                            let rows = history
                                .get("messages")
                                .and_then(Value::as_array)
                                .map(Vec::as_slice)
                                .unwrap_or(&[]);
                            runtime.apply_chat_messages(rows, chat_ready);
                        }
                        runtime.emit_view(&app);
                    }
                    Err(_) => {
                        runtime.inner.lock().expect("party lock").connected = false;
                        runtime.emit_view(&app);
                    }
                }
            }
        });

        let clk_runtime = Arc::clone(self);
        let clk_client = client.clone();
        let clk_app = app.clone();
        tokio::spawn(async move {
            let mut best_rtt = i64::MAX;
            let mut best_offset = None;
            for _ in 0..3 {
                if clk_runtime.generation.load(Ordering::SeqCst) != generation {
                    return;
                }
                let t0 = now_ms();
                if let Ok(server_ms) = clk_client.rpc::<i64>("watch_party_clock", &json!({})).await {
                    let t1 = now_ms();
                    let rtt = (t1 - t0).max(0);
                    let mid = t0 + rtt / 2;
                    if rtt < best_rtt {
                        best_rtt = rtt;
                        best_offset = Some(server_ms - mid);
                    }
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            if let Some(offset) = best_offset
                && clk_runtime.generation.load(Ordering::SeqCst) == generation
            {
                {
                    let mut inner = clk_runtime.inner.lock().expect("party lock");
                    inner.server_offset_ms = offset;
                    inner.clock_calibrated = true;
                }
                clk_runtime.emit_view(&clk_app);
            }
        });

        let rt_runtime = Arc::clone(self);
        let rt_client = client;
        let rt_app = app;
        tokio::spawn(async move {
            loop {
                if rt_runtime.generation.load(Ordering::SeqCst) != generation {
                    break;
                }
                let (party_id, member_id, device_id) = {
                    let inner = rt_runtime.inner.lock().expect("party lock");
                    (
                        inner.party_id.clone(),
                        inner.member_id.clone(),
                        inner.device_id.clone(),
                    )
                };
                if party_id.is_empty() || member_id.is_empty() {
                    break;
                }
                let _ = rt_client.ensure_fresh_session().await;
                let token = rt_client.current_session().map(|s| s.access_token);
                let conn_res =
                    RealtimeConnection::connect(rt_client.config(), token.as_deref()).await;
                let (conn, mut rx) = match conn_res {
                    Ok(pair) => pair,
                    Err(e) => {
                        tracing::warn!("Watch party realtime connect failed: {e}");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                };
                if rt_runtime.generation.load(Ordering::SeqCst) != generation {
                    break;
                }
                let conn = Arc::new(conn);
                *rt_runtime.realtime.lock().expect("rt lock") = Some(Arc::clone(&conn));
                let topic = realtime_topic(&party_id);
                let mut join_payload = json!({
                    "config": {
                        "broadcast": { "self": false, "ack": false },
                        "presence": { "key": member_id },
                        "private": true
                    }
                });
                if let Some(ref jwt) = token {
                    join_payload["access_token"] = json!(jwt);
                }
                let _ = conn.join_channel(&topic, join_payload).await;

                let mut hb = tokio::time::interval(Duration::from_secs(REALTIME_HEARTBEAT_SECS));
                hb.tick().await;
                let mut coord = tokio::time::interval(Duration::from_millis(COORDINATOR_MS));
                coord.tick().await;
                loop {
                    tokio::select! {
                        _ = hb.tick() => {
                            if rt_runtime.generation.load(Ordering::SeqCst) != generation {
                                break;
                            }
                            if conn.heartbeat().await.is_err() {
                                break;
                            }
                        }
                        _ = coord.tick() => {
                            if rt_runtime.generation.load(Ordering::SeqCst) != generation {
                                break;
                            }
                            if let Some(pb) = rt_runtime.evaluate_waiting_coordinator() {
                                let _ = conn.broadcast(&topic, "state", pb.clone()).await;
                                rt_runtime.emit_view(&rt_app);
                                let _ = rpc(
                                    &rt_client,
                                    "watch_party_update",
                                    json!({
                                        "p_party_id": party_id,
                                        "p_device_id": device_id,
                                        "p_playback": pb
                                    }),
                                ).await;
                            }
                        }
                        frame = rx.recv() => {
                            if rt_runtime.generation.load(Ordering::SeqCst) != generation {
                                break;
                            }
                            let Some(frame) = frame else {
                                break;
                            };
                            if frame.topic != topic {
                                continue;
                            }
                            match frame.event.as_str() {
                                "phx_reply" => {
                                    let status = frame
                                        .payload
                                        .get("status")
                                        .and_then(Value::as_str)
                                        .unwrap_or("");
                                    if status == "ok" {
                                        if let Some(sp) = rt_runtime.current_presence_payload() {
                                            let _ = conn.track_presence(&topic, sp.clone()).await;
                                            let _ = conn.broadcast(&topic, "status", sp).await;
                                        }
                                        let _ = conn
                                            .broadcast(
                                                &topic,
                                                "state_request",
                                                json!({ "member_id": member_id }),
                                            )
                                            .await;
                                        if let Some(pb) = rt_runtime.current_playback_json() {
                                            let _ = conn.broadcast(&topic, "state", pb).await;
                                        }
                                    } else if status == "error" {
                                        tracing::warn!("Watch party phx_reply error: {}", frame.payload);
                                        let _ = rt_client.refresh_session().await;
                                        break;
                                    }
                                }
                                "presence_state" => {
                                    if rt_runtime.apply_presence_state(&frame.payload) {
                                        if let Some(pb) = rt_runtime.evaluate_waiting_coordinator() {
                                            let _ = conn.broadcast(&topic, "state", pb).await;
                                        }
                                        rt_runtime.emit_view(&rt_app);
                                    }
                                }
                                "presence_diff" => {
                                    if rt_runtime.apply_presence_diff(&frame.payload) {
                                        if let Some(pb) = rt_runtime.evaluate_waiting_coordinator() {
                                            let _ = conn.broadcast(&topic, "state", pb).await;
                                        }
                                        rt_runtime.emit_view(&rt_app);
                                    }
                                }
                                "broadcast" => {
                                    let ev = frame
                                        .payload
                                        .get("event")
                                        .and_then(Value::as_str)
                                        .unwrap_or("");
                                    let payload = &frame.payload["payload"];
                                    match ev {
                                        "state" => {
                                            if rt_runtime.apply_realtime_playback(payload) {
                                                rt_runtime.emit_view(&rt_app);
                                            }
                                        }
                                        "status" => {
                                            if rt_runtime.apply_realtime_status(payload) {
                                                if let Some(pb) = rt_runtime.evaluate_waiting_coordinator() {
                                                    let _ = conn.broadcast(&topic, "state", pb).await;
                                                }
                                                rt_runtime.emit_view(&rt_app);
                                            }
                                        }
                                        "state_request" => {
                                            let requester = payload
                                                .get("member_id")
                                                .and_then(Value::as_str);
                                            if requester != Some(member_id.as_str()) {
                                                if let Some(sp) = rt_runtime.current_presence_payload() {
                                                    let _ = conn.track_presence(&topic, sp.clone()).await;
                                                    let _ = conn.broadcast(&topic, "status", sp).await;
                                                }
                                                if let Some(pb) = rt_runtime.current_playback_json() {
                                                    let _ = conn.broadcast(&topic, "state", pb).await;
                                                }
                                            }
                                        }
                                        "control" => {
                                            if rt_runtime.apply_realtime_control(payload) {
                                                rt_runtime.emit_view(&rt_app);
                                            }
                                        }
                                        "chat" => {
                                            if payload.is_object() {
                                                rt_runtime.apply_chat_messages(&[payload.clone()], true);
                                                rt_runtime.emit_view(&rt_app);
                                            }
                                        }
                                        "member_joined" => {
                                            if let Ok(snap) = rpc(
                                                &rt_client,
                                                "watch_party_heartbeat",
                                                json!({ "p_party_id": party_id, "p_device_id": device_id }),
                                            ).await
                                                && rt_runtime.apply(&snap)
                                            {
                                                rt_runtime.emit_view(&rt_app);
                                            }
                                        }
                                        "member_left" => {
                                            let left_id = payload
                                                .get("member_id")
                                                .and_then(Value::as_str)
                                                .unwrap_or("");
                                            let reason = payload
                                                .get("reason")
                                                .and_then(Value::as_str)
                                                .unwrap_or("");
                                            if left_id == member_id {
                                                let msg = if reason == "removed" {
                                                    "Você foi removido da sala."
                                                } else {
                                                    "Você não está mais nessa sala."
                                                };
                                                rt_runtime.stop(msg);
                                                rt_runtime.emit_view(&rt_app);
                                                break;
                                            }
                                            let new_host = payload
                                                .get("host_member_id")
                                                .and_then(Value::as_str);
                                            if rt_runtime.apply_realtime_member_left(left_id, new_host) {
                                                if let Some(pb) = rt_runtime.evaluate_waiting_coordinator() {
                                                    let _ = conn.broadcast(&topic, "state", pb).await;
                                                }
                                                rt_runtime.emit_view(&rt_app);
                                            }
                                        }
                                        "ended" => {
                                            rt_runtime.stop("A sala foi encerrada.");
                                            rt_runtime.emit_view(&rt_app);
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                if rt_runtime.generation.load(Ordering::SeqCst) != generation {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });
    }
}

async fn rpc(client: &SupabaseClient, name: &str, params: Value) -> Result<Value, String> {
    client.rpc::<Value>(name, &params).await.map_err(|e| {
        let msg = format!("{e}");
        let lower = msg.to_ascii_lowercase();
        if msg.contains("PGRST303")
            || lower.contains("jwt expired")
            || lower.contains("invalid_grant")
            || lower.contains("refresh token")
            || lower.contains("not authenticated")
        {
            "Sua sessão expirou. Saia e entre novamente na sua conta Nuvio.".to_string()
        } else {
            msg
        }
    })
}

fn party_error_message(code: &str) -> String {
    match code {
        "party_not_found" => "Nenhuma sala com esse código.".into(),
        "party_full" => "Essa sala está cheia.".into(),
        "removed" | "not_member" => "Você não está mais nessa sala.".into(),
        "party_ended" => "A sala foi encerrada.".into(),
        "rate_limited" => "Muitas tentativas. Espere um instante.".into(),
        "unavailable" => "Watch party indisponível neste servidor.".into(),
        "sign_in_required" => "Entre na sua conta Nuvio para assistir junto.".into(),
        other => other.to_string(),
    }
}

fn identity_params(
    device_id: &str,
    display_name: &str,
    avatar_url: Option<&str>,
    avatar_color: Option<&str>,
) -> Value {
    json!({
        "p_device_id": device_id,
        "p_display_name": display_name,
        "p_avatar_url": avatar_url,
        "p_avatar_color": avatar_color,
    })
}

fn device_id(state: &AppState) -> String {
    let existing = state
        .kv
        .get_string("party", "device_id")
        .ok()
        .flatten()
        .filter(|id| id.len() >= 8);
    if let Some(id) = existing {
        return id;
    }
    let id = format!("d{}{}", now_ms(), std::process::id());
    let _ = state.kv.put_string("party", "device_id", Some(&id));
    id
}

async fn prepare(state: &AppState) -> Result<(SupabaseClient, String), String> {
    let (client, _) = ensure_supabase_client(state).await?;
    if client.current_session().is_none() {
        return Err("Entre na sua conta Nuvio para usar o watch party.".into());
    }
    Ok((client, device_id(state)))
}

fn finish(
    state: &AppState,
    client: &SupabaseClient,
    app: &AppHandle,
    device: &str,
    snapshot: Value,
) -> Result<PartyView, String> {
    if let Some(code) = snapshot.get("error").and_then(Value::as_str) {
        return Err(party_error_message(code));
    }
    if !state.party.apply(&snapshot) {
        return Err("O servidor não devolveu a sala.".into());
    }
    state.party.set_device_id(device);
    state.party.start(client.clone(), app.clone());
    Ok(state.party.emit_view(app))
}

fn current_client(state: &AppState) -> Option<SupabaseClient> {
    state.supabase.read().expect("sb lock").clone()
}

#[tauri::command]
pub async fn party_create(
    state: State<'_, AppState>,
    app: AppHandle,
    display_name: String,
    avatar_url: Option<String>,
    avatar_color: Option<String>,
    media: Value,
) -> Result<PartyView, String> {
    let (client, device) = prepare(&state).await?;
    let mut params = identity_params(
        &device,
        &display_name,
        avatar_url.as_deref(),
        avatar_color.as_deref(),
    );
    params["p_media"] = media;
    params["p_playback"] = Value::Null;
    let snapshot = rpc(&client, "watch_party_create", params).await?;
    finish(&state, &client, &app, &device, snapshot)
}

#[tauri::command]
pub async fn party_join(
    state: State<'_, AppState>,
    app: AppHandle,
    code: String,
    display_name: String,
    avatar_url: Option<String>,
    avatar_color: Option<String>,
) -> Result<PartyView, String> {
    let (client, device) = prepare(&state).await?;
    let mut params = identity_params(
        &device,
        &display_name,
        avatar_url.as_deref(),
        avatar_color.as_deref(),
    );
    params["p_code"] = json!(code.trim().to_uppercase());
    let snapshot = rpc(&client, "watch_party_join", params).await?;
    finish(&state, &client, &app, &device, snapshot)
}

#[tauri::command]
pub async fn party_preview(state: State<'_, AppState>, code: String) -> Result<Value, String> {
    let (client, _) = prepare(&state).await?;
    rpc(
        &client,
        "watch_party_preview",
        json!({ "p_code": code.trim().to_uppercase() }),
    )
    .await
}

#[tauri::command]
pub async fn party_leave(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let (client, device) = prepare(&state).await?;
    let (party_id, dev_id) = {
        let inner = state.party.inner.lock().expect("party lock");
        let dev = if inner.device_id.is_empty() {
            device
        } else {
            inner.device_id.clone()
        };
        (inner.party_id.clone(), dev)
    };
    state.party.stop("");
    if !party_id.is_empty() {
        let _ = rpc(
            &client,
            "watch_party_leave",
            json!({ "p_party_id": party_id, "p_device_id": dev_id }),
        )
        .await;
    }
    state.party.emit_view(&app);
    Ok(())
}

#[tauri::command]
pub async fn party_end(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let (client, device) = prepare(&state).await?;
    let (party_id, dev_id) = {
        let inner = state.party.inner.lock().expect("party lock");
        let dev = if inner.device_id.is_empty() {
            device
        } else {
            inner.device_id.clone()
        };
        (inner.party_id.clone(), dev)
    };
    state.party.stop("");
    if !party_id.is_empty() {
        let _ = rpc(
            &client,
            "watch_party_end",
            json!({ "p_party_id": party_id, "p_device_id": dev_id }),
        )
        .await;
    }
    state.party.emit_view(&app);
    Ok(())
}

#[tauri::command]
pub fn party_status(state: State<'_, AppState>) -> PartyView {
    state.party.view()
}

#[tauri::command]
pub async fn party_set_host_only(
    state: State<'_, AppState>,
    app: AppHandle,
    host_only: bool,
) -> Result<PartyView, String> {
    let (client, device) = prepare(&state).await?;
    let (party_id, dev_id) = {
        let inner = state.party.inner.lock().expect("party lock");
        let dev = if inner.device_id.is_empty() {
            device
        } else {
            inner.device_id.clone()
        };
        (inner.party_id.clone(), dev)
    };
    let snapshot = rpc(
        &client,
        "watch_party_set_host_only",
        json!({
            "p_party_id": party_id,
            "p_device_id": dev_id,
            "p_host_only": host_only
        }),
    )
    .await?;
    if let Some(code) = snapshot.get("error").and_then(Value::as_str) {
        return Err(party_error_message(code));
    }
    state.party.apply(&snapshot);
    state
        .party
        .broadcast("control", json!({ "host_only": host_only }));
    Ok(state.party.emit_view(&app))
}

/// Reports local member player selection/loading/ready status and notifies the room.
#[tauri::command]
pub async fn party_set_status(
    state: State<'_, AppState>,
    app: AppHandle,
    status: String,
    media_key: Option<String>,
) -> Result<PartyView, String> {
    if let Some(payload) = state.party.set_self_status(&status, media_key.as_deref()) {
        state.party.track_presence(payload.clone());
        state.party.broadcast("status", payload);
        if let Some(pb) = state.party.evaluate_waiting_coordinator() {
            state.party.broadcast("state", pb.clone());
            if let Some(client) = current_client(&state) {
                let (party_id, device_id) = {
                    let inner = state.party.inner.lock().expect("party lock");
                    (inner.party_id.clone(), inner.device_id.clone())
                };
                if !party_id.is_empty() && !device_id.is_empty() {
                    let _ = rpc(
                        &client,
                        "watch_party_update",
                        json!({
                            "p_party_id": party_id,
                            "p_device_id": device_id,
                            "p_playback": pb
                        }),
                    )
                    .await;
                }
            }
        }
        return Ok(state.party.emit_view(&app));
    }
    Ok(state.party.view())
}

/// Playback position pushed by the controlling client (host, or any member when unlocked).
#[tauri::command]
pub async fn party_report_playback(
    state: State<'_, AppState>,
    app: AppHandle,
    position_ms: i64,
    paused: bool,
    rate: Option<f64>,
    media_key: String,
) -> Result<(), String> {
    if !state.party.can_control() {
        return Ok(());
    }
    let playback_rate = rate.unwrap_or(1.0).clamp(0.25, 4.0);
    let playback_json =
        state
            .party
            .set_local_playback(position_ms, paused, playback_rate, &media_key, false);
    let seq = playback_json.get("seq").and_then(Value::as_i64).unwrap_or(0);
    state.party.broadcast("state", playback_json.clone());
    state.party.emit_view(&app);

    let Some(client) = current_client(&state) else {
        return Ok(());
    };
    let (party_id, device_id) = {
        let inner = state.party.inner.lock().expect("party lock");
        (inner.party_id.clone(), inner.device_id.clone())
    };
    if party_id.is_empty() || device_id.is_empty() {
        return Ok(());
    }
    let runtime = Arc::clone(&state.party);
    tokio::spawn(async move {
        let _guard = runtime.persist_gate.lock().await;
        if runtime.inner.lock().expect("party lock").playback_seq > seq {
            return;
        }
        match rpc(
            &client,
            "watch_party_update",
            json!({
                "p_party_id": party_id,
                "p_device_id": device_id,
                "p_playback": playback_json
            }),
        )
        .await
        {
            Ok(snap) => {
                if let Some(code) = snap.get("error").and_then(Value::as_str) {
                    tracing::warn!("watch_party_update returned error: {code}");
                } else if snap.get("party").is_some_and(Value::is_object) {
                    runtime.apply(&snap);
                    runtime.emit_view(&app);
                }
            }
            Err(err) => {
                tracing::warn!("watch_party_update failed: {err}");
            }
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn party_send_message(
    state: State<'_, AppState>,
    app: AppHandle,
    body: String,
    client_id: String,
) -> Result<PartyView, String> {
    let trimmed = body.trim().chars().take(500).collect::<String>();
    if trimmed.is_empty() {
        return Ok(state.party.view());
    }
    let (client, fallback_dev) = prepare(&state).await?;
    let (party_id, dev_id) = {
        let mut inner = state.party.inner.lock().expect("party lock");
        if !inner.active || inner.party_id.is_empty() {
            return Err("Você não está em nenhuma sala.".into());
        }
        let dev = if inner.device_id.is_empty() {
            fallback_dev
        } else {
            inner.device_id.clone()
        };
        let self_member = inner
            .members
            .iter()
            .find(|m| m.id == inner.member_id)
            .cloned();
        let msg = PartyChatMessageView {
            key: client_id.clone(),
            id: None,
            member_id: inner.member_id.clone(),
            name: self_member
                .as_ref()
                .map(|m| m.name.clone())
                .unwrap_or_default(),
            avatar_url: self_member
                .as_ref()
                .map(|m| m.avatar.clone())
                .unwrap_or_default(),
            avatar_color: self_member
                .as_ref()
                .map(|m| m.color.clone())
                .unwrap_or_default(),
            body: trimmed.clone(),
            sent_at: now_ms() + inner.server_offset_ms,
            mine: true,
            sent: false,
            failed: false,
        };
        inner.chat_messages.push(msg);
        inner.chat_ready = true;
        (inner.party_id.clone(), dev)
    };
    let _ = app.emit("party:state", state.party.view());

    let res = rpc(
        &client,
        "watch_party_send_message",
        json!({
            "p_party_id": party_id,
            "p_device_id": dev_id,
            "p_body": trimmed,
            "p_client_id": client_id,
        }),
    )
    .await;

    match res {
        Ok(val) if val.get("message").is_some_and(Value::is_object) => {
            state
                .party
                .apply_chat_messages(&[val["message"].clone()], false);
        }
        _ => {
            let mut inner = state.party.inner.lock().expect("party lock");
            if let Some(m) = inner
                .chat_messages
                .iter_mut()
                .find(|m| m.key == client_id && m.id.is_none())
            {
                m.failed = true;
                m.sent = false;
            }
        }
    }
    let view = state.party.view();
    let _ = app.emit("party:state", view.clone());
    Ok(view)
}

#[tauri::command]
pub async fn party_retry_message(
    state: State<'_, AppState>,
    app: AppHandle,
    key: String,
) -> Result<PartyView, String> {
    let (client, fallback_dev) = prepare(&state).await?;
    let (party_id, dev_id, body) = {
        let mut inner = state.party.inner.lock().expect("party lock");
        let party_id = inner.party_id.clone();
        let dev = if inner.device_id.is_empty() {
            fallback_dev
        } else {
            inner.device_id.clone()
        };
        let Some(msg) = inner
            .chat_messages
            .iter_mut()
            .find(|m| m.key == key && m.id.is_none())
        else {
            return Ok(state.party.view());
        };
        msg.failed = false;
        msg.sent = false;
        (party_id, dev, msg.body.clone())
    };
    let _ = app.emit("party:state", state.party.view());

    let res = rpc(
        &client,
        "watch_party_send_message",
        json!({
            "p_party_id": party_id,
            "p_device_id": dev_id,
            "p_body": body,
            "p_client_id": key,
        }),
    )
    .await;

    match res {
        Ok(val) if val.get("message").is_some_and(Value::is_object) => {
            state
                .party
                .apply_chat_messages(&[val["message"].clone()], false);
        }
        _ => {
            let mut inner = state.party.inner.lock().expect("party lock");
            if let Some(m) = inner
                .chat_messages
                .iter_mut()
                .find(|m| m.key == key && m.id.is_none())
            {
                m.failed = true;
            }
        }
    }
    let view = state.party.view();
    let _ = app.emit("party:state", view.clone());
    Ok(view)
}

#[tauri::command]
pub fn party_set_chat_open(state: State<'_, AppState>, app: AppHandle, open: bool) -> PartyView {
    {
        let mut inner = state.party.inner.lock().expect("party lock");
        inner.chat_open = open;
        if open {
            inner.chat_unread = 0;
        }
    }
    let view = state.party.view();
    let _ = app.emit("party:state", view.clone());
    view
}

#[tauri::command]
pub async fn party_remove_member(
    state: State<'_, AppState>,
    app: AppHandle,
    member_id: String,
) -> Result<PartyView, String> {
    let (client, fallback_dev) = prepare(&state).await?;
    let (party_id, dev_id) = {
        let inner = state.party.inner.lock().expect("party lock");
        let dev = if inner.device_id.is_empty() {
            fallback_dev
        } else {
            inner.device_id.clone()
        };
        (inner.party_id.clone(), dev)
    };
    let snapshot = rpc(
        &client,
        "watch_party_remove_member",
        json!({
            "p_party_id": party_id,
            "p_device_id": dev_id,
            "p_member_id": member_id,
        }),
    )
    .await?;
    if let Some(code) = snapshot.get("error").and_then(Value::as_str) {
        return Err(party_error_message(code));
    }
    state.party.apply(&snapshot);
    let view = state.party.view();
    let _ = app.emit("party:state", view.clone());
    Ok(view)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_keeps_device_id_separate_from_member_id() {
        let runtime = PartyRuntime::new();
        runtime.set_device_id("d17000000000001234");
        let applied = runtime.apply(&json!({
            "member_id": "member-uuid-1",
            "members": [
                { "id": "member-uuid-1", "display_name": "Host" },
                { "id": "member-uuid-2", "display_name": "Convidado" }
            ],
            "party": {
                "id": "party-uuid-1",
                "code": "ABC123",
                "host_member_id": "member-uuid-1",
                "host_only": false,
                "max_members": 4,
                "media": { "title": "Lupin" }
            }
        }));
        assert!(applied);
        {
            let inner = runtime.inner.lock().expect("party lock");
            assert_eq!(inner.device_id, "d17000000000001234");
            assert_eq!(inner.member_id, "member-uuid-1");
            assert_eq!(inner.party_id, "party-uuid-1");
        }
        let view = runtime.view();
        assert!(view.active);
        assert!(view.is_host);
        assert_eq!(view.code, "ABC123");
        assert_eq!(view.members.len(), 2);
        assert!(view.members[0].is_host);
        assert!(!view.members[1].is_host);

        runtime.apply_chat_messages(
            &[json!({
                "id": 10,
                "member_id": "member-uuid-2",
                "name": "Convidado",
                "body": "Oi!",
                "sent_at": 1700000000000i64,
                "mine": false
            })],
            true,
        );
        let after_chat = runtime.view();
        assert!(after_chat.chat_ready);
        assert_eq!(after_chat.chat_unread, 1);
        assert_eq!(after_chat.chat_messages.len(), 1);
        assert_eq!(after_chat.chat_messages[0].body, "Oi!");

        runtime.stop("Você não está mais nessa sala.");
        let stopped = runtime.view();
        assert!(!stopped.active);
        assert_eq!(stopped.error, "Você não está mais nessa sala.");
    }

    #[test]
    fn realtime_state_respects_host_only_media_key_and_control() {
        let runtime = PartyRuntime::new();
        runtime.apply(&json!({
            "member_id": "guest-uuid",
            "members": [
                { "id": "host-uuid", "display_name": "Host" },
                { "id": "guest-uuid", "display_name": "Convidado" }
            ],
            "party": {
                "id": "party-uuid-2",
                "code": "XYZ789",
                "host_member_id": "host-uuid",
                "host_only": true,
                "max_members": 4,
                "media": { "meta_type": "movie", "meta_id": "tt0133093", "title": "Matrix" }
            }
        }));

        // Guest broadcasts are ignored when host_only is true.
        assert!(!runtime.apply_realtime_playback(&json!({
            "seq": 2,
            "member_id": "guest-uuid",
            "paused": true,
            "position_ms": 15000,
            "rate": 1.0,
            "at": 1700000001000i64,
            "media_key": "movie|tt0133093||"
        })));

        // Mismatched media key is ignored.
        assert!(!runtime.apply_realtime_playback(&json!({
            "seq": 2,
            "member_id": "host-uuid",
            "paused": true,
            "position_ms": 15000,
            "rate": 1.0,
            "at": 1700000001000i64,
            "media_key": "movie|tt9999999||"
        })));

        // Valid host pause update applies immediately.
        assert!(runtime.apply_realtime_playback(&json!({
            "seq": 3,
            "member_id": "host-uuid",
            "paused": true,
            "position_ms": 22500,
            "rate": 1.0,
            "at": 1700000002000i64,
            "media_key": "movie|tt0133093||"
        })));
        let pb = runtime.view().playback.expect("playback updated");
        assert!(pb.paused);
        assert_eq!(pb.position_ms, 22500);
        assert_eq!(pb.seq, 3);

        // Realtime control unlock propagates.
        assert!(runtime.apply_realtime_control(&json!({ "host_only": false })));
        assert!(!runtime.view().host_only);
        assert!(runtime.view().can_control);
    }

    #[test]
    fn coordinator_auto_pauses_while_member_choosing_and_resumes_when_ready() {
        let runtime = PartyRuntime::new();
        runtime.apply(&json!({
            "member_id": "host-uuid",
            "members": [
                { "id": "host-uuid", "display_name": "Anfitrião" },
                { "id": "guest-uuid", "display_name": "Convidado" }
            ],
            "party": {
                "id": "party-uuid-3",
                "code": "WAIT01",
                "host_member_id": "host-uuid",
                "host_only": true,
                "max_members": 4,
                "media": { "meta_type": "movie", "meta_id": "tt1392170", "title": "Jogos Vorazes" }
            }
        }));
        runtime.set_self_status("ready", Some("movie|tt1392170||"));
        runtime.set_local_playback(12000, false, 1.0, "movie|tt1392170||", false);

        // Guest enters stream/player selection -> coordinator pauses room immediately.
        assert!(runtime.apply_realtime_status(&json!({
            "member_id": "guest-uuid",
            "status": "choosing"
        })));
        let paused_pb = runtime
            .evaluate_waiting_coordinator()
            .expect("coordinator should pause while guest is choosing");
        assert_eq!(paused_pb["paused"], true);
        assert_eq!(paused_pb["waiting"], true);
        assert_eq!(runtime.view().waiting_for, vec!["Convidado".to_string()]);

        // Attempting to unpause while guest is still loading keeps playback paused.
        let forced_stay = runtime.set_local_playback(12000, false, 1.0, "movie|tt1392170||", false);
        assert_eq!(forced_stay["paused"], true);
        assert_eq!(forced_stay["waiting"], true);

        // Guest finishes loading -> coordinator resumes together synchronously.
        assert!(runtime.apply_realtime_status(&json!({
            "member_id": "guest-uuid",
            "status": "ready"
        })));
        let resumed_pb = runtime
            .evaluate_waiting_coordinator()
            .expect("coordinator should auto-resume once all members are ready");
        assert_eq!(resumed_pb["paused"], false);
        assert_eq!(resumed_pb["waiting"], false);
        assert!(runtime.view().waiting_for.is_empty());
        assert!(media_keys_compatible("movie|tt1392170||", "movie|tmdb:70160||"));
        assert!(!media_keys_compatible("series|tt1|1|1", "series|tt1|1|2"));

        // Wire presence stays Kotlin-compatible (`choosing` serializes as `browsing` + `media_key`).
        let wire_payload = runtime
            .set_self_status("choosing", Some("movie|tt1392170||"))
            .expect("wire payload emitted");
        assert_eq!(wire_payload["status"], "browsing");
        assert_eq!(wire_payload["media_key"], "movie|tt1392170||");
        assert!(runtime.apply_presence_state(&json!({
            "guest-uuid": {
                "metas": [{
                    "member_id": "guest-uuid",
                    "status": "browsing",
                    "media_key": "movie|tt1392170||"
                }]
            }
        })));
        assert_eq!(runtime.view().members[1].status, "choosing");
        assert!(runtime.apply_presence_diff(&json!({
            "joins": {},
            "leaves": { "guest-uuid": { "metas": [{}] } }
        })));
        assert_eq!(runtime.view().members[1].status, "browsing");
    }
}
