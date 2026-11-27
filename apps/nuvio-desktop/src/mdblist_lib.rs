use crate::{AppState, active_profile_idx, load_session_inner, load_settings_inner};
use nuvio_core::integrations::mdblist::MdbListClient;
use serde_json::{Map, Value, json};
use tauri::State;

const WATCHLIST_KEY: &str = "watchlist";
const CACHE_TTL_SECS: u64 = 10 * 60;

fn progress_cache_key(state: &AppState) -> String {
    let user = load_session_inner(&state.kv)
        .and_then(|s| s.user)
        .map(|u| u.id)
        .unwrap_or_else(|| "guest".into());
    format!("mdblist_prog:{user}:{}", active_profile_idx(&state.kv))
}

fn cache_key(state: &AppState) -> String {
    let user = load_session_inner(&state.kv)
        .and_then(|s| s.user)
        .map(|u| u.id)
        .unwrap_or_else(|| "guest".into());
    format!("mdblist_lib:{user}:{}", active_profile_idx(&state.kv))
}

fn mdblist_image(raw: Option<&str>, size: &str) -> String {
    let Some(s) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return String::new();
    };
    if s.starts_with("http://") || s.starts_with("https://") {
        return s.to_string();
    }
    if s.starts_with('/') {
        return format!("https://image.tmdb.org/t/p/{size}{s}");
    }
    String::new()
}

fn parse_year(v: &Value) -> String {
    v.get("release_year")
        .or_else(|| v.get("year"))
        .and_then(|y| match y {
            Value::Number(n) => Some(n.to_string()),
            Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
            _ => None,
        })
        .unwrap_or_default()
}

fn normalize_media_ids(row: &Value) -> Map<String, Value> {
    let mut ids = Map::new();
    if let Some(imdb) = row.get("imdb_id").and_then(Value::as_str).filter(|s| !s.is_empty()) {
        ids.insert("imdb".into(), json!(imdb));
    }
    if let Some(tmdb) = row.get("id").and_then(Value::as_u64) {
        ids.insert("tmdb".into(), json!(tmdb));
    }
    if let Some(tvdb) = row.get("tvdb_id").and_then(Value::as_u64) {
        ids.insert("tvdb".into(), json!(tvdb));
    }
    if let Some(nested) = row.get("ids").and_then(Value::as_object) {
        for (k, v) in nested {
            if !v.is_null() {
                ids.insert(k.clone(), v.clone());
            }
        }
    }
    ids
}

fn primary_content_id(ids: &Map<String, Value>) -> Option<String> {
    if let Some(imdb) = ids.get("imdb").and_then(Value::as_str).filter(|s| !s.is_empty()) {
        return Some(imdb.to_string());
    }
    for key in ["tmdb", "tvdb", "trakt"] {
        if let Some(val) = ids.get(key) {
            if let Some(n) = val.as_u64() {
                return Some(format!("{key}:{n}"));
            }
            if let Some(s) = val.as_str().filter(|s| !s.is_empty()) {
                return Some(format!("{key}:{s}"));
            }
        }
    }
    None
}

fn decode_item(row: &Value, fallback_type: Option<&str>, list_key: &str, list_name: &str) -> Option<Value> {
    let raw_type = row
        .get("mediatype")
        .or_else(|| row.get("type"))
        .and_then(Value::as_str)
        .or(fallback_type)?;
    let lower = raw_type.to_ascii_lowercase();
    let content_type = match lower.as_str() {
        "movie" | "film" => "movie",
        "show" | "series" | "tv" | "anime" => "series",
        _ => return None,
    };

    let ids = normalize_media_ids(row);
    let content_id = primary_content_id(&ids)?;
    let name = row
        .get("title")
        .or_else(|| row.get("name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&content_id);
    let poster = mdblist_image(row.get("poster").and_then(Value::as_str), "w500");
    let background = mdblist_image(
        row.get("backdrop")
            .or_else(|| row.get("background"))
            .and_then(Value::as_str),
        "w1280",
    );
    let description = row
        .get("description")
        .or_else(|| row.get("overview"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let added_at = row
        .get("listed_at")
        .or_else(|| row.get("added_at"))
        .or_else(|| row.get("watchlist_at"))
        .and_then(Value::as_str)
        .and_then(parse_iso_millis)
        .unwrap_or(0);

    Some(json!({
        "content_id": content_id,
        "content_type": content_type,
        "name": name,
        "poster": poster,
        "background": background,
        "description": description,
        "release_info": parse_year(row),
        "added_at": added_at,
        "list_key": list_key,
        "list_name": list_name,
        "media_ids": Value::Object(ids),
    }))
}

fn parse_iso_millis(s: &str) -> Option<i64> {
    let trimmed = s.trim();
    if trimmed.len() < 10 {
        return None;
    }
    let y: i64 = trimmed.get(0..4)?.parse().ok()?;
    let m: i64 = trimmed.get(5..7)?.parse().ok()?;
    let d: i64 = trimmed.get(8..10)?.parse().ok()?;
    let days = days_from_civil(y, m, d);
    let (hh, mm, ss) = if trimmed.len() >= 19 {
        (
            trimmed.get(11..13).and_then(|v| v.parse::<i64>().ok()).unwrap_or(0),
            trimmed.get(14..16).and_then(|v| v.parse::<i64>().ok()).unwrap_or(0),
            trimmed.get(17..19).and_then(|v| v.parse::<i64>().ok()).unwrap_or(0),
        )
    } else {
        (0, 0, 0)
    };
    Some((days * 86_400 + hh * 3_600 + mm * 60 + ss) * 1000)
}

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = y - if m <= 2 { 1 } else { 0 };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn decode_items_payload(payload: &Value, list_key: &str, list_name: &str) -> Vec<Value> {
    let mut out = Vec::new();
    if let Some(arr) = payload.as_array() {
        for row in arr {
            if let Some(item) = decode_item(row, None, list_key, list_name) {
                out.push(item);
            }
        }
        return out;
    }
    if let Some(arr) = payload.get("items").and_then(Value::as_array) {
        for row in arr {
            if let Some(item) = decode_item(row, None, list_key, list_name) {
                out.push(item);
            }
        }
        return out;
    }
    if let Some(movies) = payload.get("movies").and_then(Value::as_array) {
        for row in movies {
            if let Some(item) = decode_item(row, Some("movie"), list_key, list_name) {
                out.push(item);
            }
        }
    }
    if let Some(shows) = payload.get("shows").and_then(Value::as_array) {
        for row in shows {
            if let Some(item) = decode_item(row, Some("show"), list_key, list_name) {
                out.push(item);
            }
        }
    }
    out
}

pub fn build_ids_object(id: &str) -> Result<Value, String> {
    let clean = id.trim();
    if clean.starts_with("tt") {
        return Ok(json!({ "imdb": clean }));
    }
    if let Some(v) = clean.strip_prefix("imdb:") {
        return Ok(json!({ "imdb": v }));
    }
    for prefix in ["tmdb", "tvdb", "trakt"] {
        if let Some(rest) = clean.strip_prefix(&format!("{prefix}:"))
            && let Ok(num) = rest.parse::<u64>()
        {
            return Ok(json!({ prefix: num }));
        }
    }
    Err("O título precisa de um ID IMDb, TMDB, TVDB ou Trakt para salvar no MDBList.".into())
}

#[tauri::command]
pub async fn mdblist_library_sync(
    state: State<'_, AppState>,
    force: Option<bool>,
) -> Result<Value, String> {
    let key = load_settings_inner(&state.kv).services.mdblist_key;
    if key.trim().is_empty() {
        return Err("Configure a chave de API do MDBList em Ajustes → Integrações.".into());
    }

    let ckey = cache_key(&state);
    if !force.unwrap_or(false)
        && let Some(raw) = state.cache.get(&ckey).ok().flatten()
        && let Ok(cached) = serde_json::from_slice::<Value>(&raw)
    {
        return Ok(cached);
    }

    let client = MdbListClient::new(key).map_err(|e| format!("Falha ao configurar MDBList: {e}"))?;
    let (watchlist_res, lists_res) = tokio::join!(client.watchlist_items(), client.user_lists_raw());
    let watchlist_payload = watchlist_res.map_err(|e| format!("Falha ao carregar Watchlist do MDBList: {e}"))?;
    let mut items = decode_items_payload(&watchlist_payload, WATCHLIST_KEY, "Watchlist");
    let mut lists = vec![json!({
        "key": WATCHLIST_KEY,
        "name": "Watchlist",
        "read_only": false,
    })];

    if let Ok(raw_lists) = lists_res
        && let Some(arr) = raw_lists.as_array()
    {
        let mut list_metas: Vec<(u64, String, String, bool)> = Vec::new();
        for row in arr.iter().take(20) {
            let Some(lid) = row.get("id").and_then(Value::as_u64) else {
                continue;
            };
            let name = row
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("Lista")
                .trim()
                .to_string();
            let is_dynamic = row.get("dynamic").and_then(Value::as_bool).unwrap_or(false)
                || row.get("type").and_then(Value::as_str) == Some("dynamic");
            let lkey = format!("list:{lid}");
            lists.push(json!({
                "key": lkey,
                "name": name,
                "read_only": is_dynamic,
            }));
            list_metas.push((lid, lkey, name, is_dynamic));
        }

        let mut handles = Vec::with_capacity(list_metas.len());
        for (lid, lkey, name, _) in list_metas {
            let c = client.clone();
            handles.push(tokio::spawn(async move {
                let res = c.list_items_rich(lid).await;
                (lkey, name, res)
            }));
        }
        for h in handles {
            if let Ok((lkey, name, Ok(payload))) = h.await {
                items.extend(decode_items_payload(&payload, &lkey, &name));
            }
        }
    }

    let snapshot = json!({
        "configured": true,
        "lists": lists,
        "items": items,
    });
    if let Ok(bytes) = serde_json::to_vec(&snapshot) {
        let _ = state.cache.set(&ckey, &bytes, Some(CACHE_TTL_SECS));
    }
    Ok(snapshot)
}

#[tauri::command]
pub async fn mdblist_library_mutate(
    state: State<'_, AppState>,
    list_key: Option<String>,
    action: String,
    media_type: String,
    id: String,
) -> Result<(), String> {
    let key = load_settings_inner(&state.kv).services.mdblist_key;
    if key.trim().is_empty() {
        return Err("Configure a chave de API do MDBList em Ajustes → Integrações.".into());
    }
    let client = MdbListClient::new(key).map_err(|e| format!("Falha ao configurar MDBList: {e}"))?;
    let ids = build_ids_object(&id)?;
    let bucket = if media_type == "movie" { "movies" } else { "shows" };
    let target = list_key.as_deref().unwrap_or(WATCHLIST_KEY);
    let act = if action == "remove" { "remove" } else { "add" };

    if target == WATCHLIST_KEY || target == "all" {
        let body = json!({ bucket: [{ "ids": ids }] });
        client
            .mutate_watchlist(act, &body)
            .await
            .map_err(|e| format!("MDBList Watchlist: {e}"))?;
    } else if let Some(lid) = target.strip_prefix("list:").and_then(|s| s.parse::<u64>().ok()) {
        let body = json!({ bucket: [ids] });
        client
            .mutate_list(lid, act, &body)
            .await
            .map_err(|e| format!("MDBList Lista: {e}"))?;
    } else {
        return Err("Esta lista do MDBList é somente leitura.".into());
    }

    let ckey = cache_key(&state);
    let _ = state.cache.remove(&ckey);
    Ok(())
}

fn id_aliases(ids: &Map<String, Value>) -> Vec<String> {
    let mut out = Vec::new();
    for (k, v) in ids {
        if k == "slug" || v.is_null() {
            continue;
        }
        if let Some(s) = v.as_str().filter(|s| !s.is_empty()) {
            out.push(if k == "imdb" {
                s.to_string()
            } else {
                format!("{k}:{s}")
            });
        } else if let Some(n) = v.as_u64() {
            out.push(format!("{k}:{n}"));
        }
    }
    out
}

fn decode_watched_keys(payload: &Value) -> Vec<String> {
    let mut keys = Vec::new();
    let rows = payload
        .as_array()
        .or_else(|| payload.get("items").and_then(Value::as_array));

    let push_movie = |row: &Value, keys: &mut Vec<String>| {
        let ids = normalize_media_ids(row.get("movie").unwrap_or(row));
        keys.extend(id_aliases(&ids));
    };
    let push_episode = |row: &Value, keys: &mut Vec<String>| {
        let show = row.get("show").unwrap_or(row);
        let ids = normalize_media_ids(show);
        let ep = row.get("episode").unwrap_or(row);
        let season = ep
            .get("season")
            .or_else(|| row.get("season"))
            .and_then(Value::as_i64);
        let number = ep
            .get("number")
            .or_else(|| ep.get("episode"))
            .or_else(|| row.get("episode_number"))
            .and_then(Value::as_i64);
        if let (Some(s), Some(e)) = (season, number) {
            for alias in id_aliases(&ids) {
                keys.push(format!("{alias}:{s}:{e}"));
            }
        } else {
            keys.extend(id_aliases(&ids));
        }
    };

    if let Some(arr) = rows {
        for row in arr {
            let kind = row
                .get("type")
                .or_else(|| row.get("mediatype"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_ascii_lowercase();
            if kind == "movie" || row.get("movie").is_some() {
                push_movie(row, &mut keys);
            } else if kind == "episode" || kind == "show" || row.get("episode").is_some() || row.get("show").is_some() {
                push_episode(row, &mut keys);
            }
        }
        return keys;
    }

    if let Some(movies) = payload.get("movies").and_then(Value::as_array) {
        for row in movies {
            push_movie(row, &mut keys);
        }
    }
    if let Some(episodes) = payload.get("episodes").and_then(Value::as_array) {
        for row in episodes {
            push_episode(row, &mut keys);
        }
    }
    if let Some(shows) = payload.get("shows").and_then(Value::as_array) {
        for row in shows {
            let ids = normalize_media_ids(row.get("show").unwrap_or(row));
            if let Some(seasons) = row.get("seasons").and_then(Value::as_array) {
                for season in seasons {
                    let s_num = season.get("number").and_then(Value::as_i64);
                    if let (Some(s), Some(eps)) = (s_num, season.get("episodes").and_then(Value::as_array)) {
                        for ep in eps {
                            if let Some(e) = ep.get("number").and_then(Value::as_i64) {
                                for alias in id_aliases(&ids) {
                                    keys.push(format!("{alias}:{s}:{e}"));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    keys
}

fn decode_playback_cards(payload: &Value) -> Vec<Value> {
    let mut cards = Vec::new();
    let Some(rows) = payload
        .as_array()
        .or_else(|| payload.get("items").and_then(Value::as_array))
    else {
        return cards;
    };
    for row in rows {
        let progress = row.get("progress").and_then(Value::as_f64).unwrap_or(0.0);
        if !(progress > 0.0 && progress < 80.0) {
            continue;
        }
        let is_movie = row.get("type").and_then(Value::as_str) == Some("movie")
            || row.get("movie").is_some();
        let media = row
            .get("movie")
            .or_else(|| row.get("show"))
            .unwrap_or(row);
        let ids = normalize_media_ids(media);
        let Some(content_id) = primary_content_id(&ids) else {
            continue;
        };
        let ep = row.get("episode").unwrap_or(row);
        let season = if is_movie {
            None
        } else {
            ep.get("season")
                .or_else(|| row.get("season"))
                .and_then(Value::as_i64)
        };
        let episode = if is_movie {
            None
        } else {
            ep.get("number")
                .or_else(|| ep.get("episode"))
                .or_else(|| row.get("episode_number"))
                .and_then(Value::as_i64)
        };
        if !is_movie && (season.is_none() || episode.is_none()) {
            continue;
        }
        let title = media
            .get("title")
            .or_else(|| media.get("name"))
            .and_then(Value::as_str)
            .unwrap_or(&content_id);
        let poster = mdblist_image(media.get("poster").and_then(Value::as_str), "w500");
        let background = mdblist_image(
            media
                .get("backdrop")
                .or_else(|| media.get("background"))
                .and_then(Value::as_str),
            "w1280",
        );
        let video_id = match (season, episode) {
            (Some(s), Some(e)) => format!("{content_id}:{s}:{e}"),
            _ => content_id.clone(),
        };
        let last_watched = row
            .get("paused_at")
            .or_else(|| row.get("watched_at"))
            .or_else(|| row.get("updated_at"))
            .and_then(Value::as_str)
            .and_then(parse_iso_millis)
            .unwrap_or(0);
        let episode_title = ep.get("title").or_else(|| ep.get("name")).and_then(Value::as_str);

        cards.push(json!({
            "content_id": content_id,
            "media_ids": Value::Object(ids),
            "content_type": if is_movie { "movie" } else { "series" },
            "video_id": video_id,
            "season": season,
            "episode": episode,
            "title": title,
            "episode_title": episode_title,
            "artwork": if background.is_empty() { &poster } else { &background },
            "poster": poster,
            "background": background,
            "progress_percent": progress.round().clamp(1.0, 99.0) as u8,
            "position_ms": 0,
            "duration_ms": 0,
            "last_watched": last_watched,
        }));
    }
    cards.sort_by_key(|c| std::cmp::Reverse(c["last_watched"].as_i64().unwrap_or(0)));
    cards
}

#[tauri::command]
pub async fn mdblist_account_status(state: State<'_, AppState>) -> Result<Value, String> {
    let key = load_settings_inner(&state.kv).services.mdblist_key;
    if key.trim().is_empty() {
        return Ok(json!({ "connected": false, "username": Value::Null }));
    }
    let client = MdbListClient::new(key).map_err(|e| e.to_string())?;
    match client.user_profile().await {
        Ok(user) => {
            let username = user
                .get("username")
                .or_else(|| user.get("name"))
                .and_then(Value::as_str)
                .map(str::to_string);
            Ok(json!({ "connected": true, "username": username }))
        }
        Err(e) => Err(format!("Falha ao validar chave do MDBList: {e}")),
    }
}

#[tauri::command]
pub async fn mdblist_progress_sync(
    state: State<'_, AppState>,
    force: Option<bool>,
) -> Result<Value, String> {
    let key = load_settings_inner(&state.kv).services.mdblist_key;
    if key.trim().is_empty() {
        return Err("Configure a chave de API do MDBList em Rastreamento ou Integrações.".into());
    }
    let ckey = progress_cache_key(&state);
    if !force.unwrap_or(false)
        && let Some(raw) = state.cache.get(&ckey).ok().flatten()
        && let Ok(cached) = serde_json::from_slice::<Value>(&raw)
    {
        return Ok(cached);
    }

    let client = MdbListClient::new(key).map_err(|e| format!("Falha ao configurar MDBList: {e}"))?;
    let (watched_res, playback_res) = tokio::join!(client.sync_watched(), client.sync_playback());
    let watched_keys = watched_res
        .map(|v| decode_watched_keys(&v))
        .unwrap_or_default();
    let continue_watching = playback_res
        .map(|v| decode_playback_cards(&v))
        .unwrap_or_default();

    let snapshot = json!({
        "watched_keys": watched_keys,
        "continue_watching": continue_watching,
    });
    if let Ok(bytes) = serde_json::to_vec(&snapshot) {
        let _ = state.cache.set(&ckey, &bytes, Some(CACHE_TTL_SECS));
    }
    Ok(snapshot)
}

#[tauri::command]
pub async fn mdblist_watched_mutate(
    state: State<'_, AppState>,
    remove: bool,
    media_type: String,
    id: String,
    season: Option<i64>,
    episode: Option<i64>,
) -> Result<(), String> {
    let key = load_settings_inner(&state.kv).services.mdblist_key;
    if key.trim().is_empty() {
        return Err("Configure a chave de API do MDBList.".into());
    }
    let client = MdbListClient::new(key).map_err(|e| e.to_string())?;
    let ids = build_ids_object(&id)?;
    let body = if media_type == "movie" {
        json!({ "movies": [{ "ids": ids }] })
    } else if let (Some(s), Some(e)) = (season, episode) {
        json!({
            "shows": [{
                "ids": ids,
                "seasons": [{ "number": s, "episodes": [{ "number": e }] }]
            }]
        })
    } else {
        json!({ "shows": [{ "ids": ids }] })
    };
    client
        .mutate_watched(remove, &body)
        .await
        .map_err(|e| format!("MDBList histórico: {e}"))?;
    let _ = state.cache.remove(&progress_cache_key(&state));
    Ok(())
}

#[tauri::command]
pub async fn mdblist_scrobble(
    state: State<'_, AppState>,
    action: String,
    media_type: String,
    id: String,
    season: Option<i64>,
    episode: Option<i64>,
    progress: Option<f64>,
) -> Result<(), String> {
    let key = load_settings_inner(&state.kv).services.mdblist_key;
    if key.trim().is_empty() {
        return Ok(());
    }
    let client = MdbListClient::new(key).map_err(|e| e.to_string())?;
    let ids = build_ids_object(&id)?;
    let mut body = if media_type == "movie" {
        json!({ "movie": { "ids": ids } })
    } else {
        let (Some(s), Some(e)) = (season, episode) else {
            return Ok(());
        };
        json!({ "show": { "ids": ids, "season": s, "episode": e } })
    };
    if let Some(pct) = progress {
        body["progress"] = json!(pct.clamp(0.0, 100.0));
    }
    let _ = client.scrobble(&action, &body).await;
    if action == "stop" || action == "clear" {
        let _ = state.cache.remove(&progress_cache_key(&state));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{build_ids_object, decode_items_payload, decode_playback_cards, decode_watched_keys};
    use serde_json::json;

    #[test]
    fn builds_ids_for_imdb_and_prefixed_ids() {
        assert_eq!(build_ids_object("tt0133093").unwrap(), json!({ "imdb": "tt0133093" }));
        assert_eq!(build_ids_object("tmdb:603").unwrap(), json!({ "tmdb": 603 }));
        assert!(build_ids_object("custom-id").is_err());
    }

    #[test]
    fn decodes_mdblist_watchlist_movies_and_shows() {
        let payload = json!({
            "movies": [{
                "id": 603,
                "imdb_id": "tt0133093",
                "title": "The Matrix",
                "release_year": 1999,
                "poster": "/f89U3ADr1oiB1s9GkdPOEpXUk5H.jpg",
                "watchlist_at": "2026-10-01T12:00:00Z"
            }],
            "shows": [{
                "id": 1396,
                "imdb_id": "tt0903747",
                "title": "Breaking Bad",
                "release_year": 2008
            }]
        });
        let items = decode_items_payload(&payload, "watchlist", "Watchlist");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["content_id"], "tt0133093");
        assert_eq!(items[0]["content_type"], "movie");
        assert_eq!(items[1]["content_type"], "series");
        assert!(items[0]["added_at"].as_i64().unwrap() > 0);
    }

    #[test]
    fn decodes_mdblist_watched_and_playback_cards() {
        let watched = json!({
            "movies": [{ "imdb_id": "tt0133093", "id": 603 }],
            "episodes": [{
                "show": { "imdb_id": "tt0903747", "id": 1396 },
                "season": 1,
                "episode_number": 2
            }]
        });
        let keys = decode_watched_keys(&watched);
        assert!(keys.contains(&"tt0133093".to_string()));
        assert!(keys.contains(&"tt0903747:1:2".to_string()));

        let playback = json!([
            {
                "type": "episode",
                "progress": 42.5,
                "paused_at": "2026-10-05T10:00:00Z",
                "show": { "imdb_id": "tt0903747", "title": "Breaking Bad" },
                "episode": { "season": 2, "number": 3, "title": "Bit by a Dead Bee" }
            },
            {
                "type": "movie",
                "progress": 92.0,
                "movie": { "imdb_id": "tt0133093", "title": "The Matrix" }
            }
        ]);
        let cards = decode_playback_cards(&playback);
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0]["content_id"], "tt0903747");
        assert_eq!(cards[0]["video_id"], "tt0903747:2:3");
        assert_eq!(cards[0]["progress_percent"], 43);
    }
}
