use crate::{AppState, active_profile_idx, load_session_inner};
use nuvio_core::integrations::simkl::SimklClient;
use reqwest::Method;
use serde_json::{Value, json};
use tauri::State;

fn profile_key(state: &AppState) -> String {
    let user = load_session_inner(&state.kv)
        .and_then(|s| s.user)
        .map(|u| u.id)
        .unwrap_or_else(|| "guest".into());
    format!("simkl/{user}/{}", active_profile_idx(&state.kv))
}

fn client_id(saved: Option<String>) -> String {
    saved
        .filter(|v| !v.trim().is_empty())
        .or_else(|| std::env::var("SIMKL_CLIENT_ID").ok())
        .unwrap_or_else(|| option_env!("SIMKL_CLIENT_ID").unwrap_or_default().into())
}

#[tauri::command]
pub fn simkl_status(state: State<'_, AppState>) -> Result<Value, String> {
    let profile = profile_key(&state);
    let store = state.kv.namespace(&profile);
    let get = |k| store.get_string(k).map_err(|e| e.to_string());
    Ok(json!({
        "profile":profile, "connected":get("token")?.is_some(),
        "clientId":client_id(get("client_id")?), "username":get("username")?,
        "sources":get("sources")?.and_then(|v|serde_json::from_str::<Value>(&v).ok()),
        "snapshot":get("snapshot")?.and_then(|v|serde_json::from_str::<Value>(&v).ok())
    }))
}

#[tauri::command]
pub async fn simkl_action(
    state: State<'_, AppState>,
    profile: String,
    action: String,
    data: Value,
) -> Result<Value, String> {
    if profile != profile_key(&state) {
        return Err("O perfil mudou.".into());
    }
    let store = state.kv.namespace(&profile);
    let put = |k, v: Option<&str>| store.put_string(k, v).map_err(|e| e.to_string());
    let get = |k| store.get_string(k).map_err(|e| e.to_string());
    if action == "disconnect" {
        for key in ["token", "username", "sources", "snapshot", "pending_pin"] {
            put(key, None)?;
        }
        return Ok(Value::Null);
    }
    if action == "sources" {
        put("sources", Some(&data.to_string()))?;
        return Ok(Value::Null);
    }
    let id = if action == "connect" {
        data["clientId"]
            .as_str()
            .unwrap_or_default()
            .trim()
            .to_owned()
    } else {
        client_id(get("client_id")?)
    };
    if id.is_empty() {
        return Err("Informe o Client ID do aplicativo Simkl.".into());
    }
    let api = SimklClient::new(&id).map_err(|e| e.to_string())?;
    let fail = |e: nuvio_core::integrations::IntegrationError| e.to_string();
    match action.as_str() {
        "connect" => {
            let pin = api.request_pin().await.map_err(fail)?;
            let code = pin
                .user_code
                .as_deref()
                .filter(|_| pin.result.eq_ignore_ascii_case("OK"))
                .ok_or("O Simkl não retornou o código de ativação.")?;
            put("client_id", Some(&id))?;
            put("pending_pin", Some(code))?;
            return Ok(
                json!({"code":code,"url":pin.verification_uri.or(pin.verification_url),"expires":pin.expires_in,"interval":pin.interval}),
            );
        }
        "poll" => {
            let code = get("pending_pin")?.ok_or("Autorização cancelada.")?;
            if data.as_str() != Some(&code)
                || !code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            {
                return Err("PIN inválido.".into());
            }
            let pin = api
                .request(Method::GET, &format!("/oauth/pin/{code}"), None, &[], None)
                .await
                .map_err(fail)?;
            if pin["device_code"].is_string() {
                return Err("O PIN expirou.".into());
            }
            if let Some(token) = pin["access_token"].as_str().filter(|s| !s.is_empty()) {
                let user = api
                    .request(
                        Method::POST,
                        "/users/settings",
                        Some(token),
                        &[],
                        Some(&json!({})),
                    )
                    .await
                    .ok();
                if get("pending_pin")?.as_deref() != Some(&code) {
                    return Err("Autorização cancelada.".into());
                }
                put("token", Some(token))?;
                put(
                    "username",
                    user.as_ref()
                        .and_then(|u| u.pointer("/user/name"))
                        .and_then(Value::as_str),
                )?;
                put("pending_pin", None)?;
                return Ok(json!(true));
            }
            return Ok(json!(false));
        }
        _ => {}
    }
    let token = get("token")?.ok_or("Conecte sua conta Simkl.")?;
    if action == "sync" {
        let (items, playback) = tokio::try_join!(
            api.get_all_items(&token),
            api.request(Method::GET, "/sync/playback", Some(&token), &[], None)
        )
        .map_err(fail)?;
        if get("token")?.as_ref() != Some(&token) {
            return Err("A conta mudou.".into());
        }
        let result = json!({"items":items,"playback":playback});
        put("snapshot", Some(&result.to_string()))?;
        return Ok(result);
    }
    if action == "similar" {
        let source = data["source"].as_str().ok_or("ID ausente.")?;
        if ![
            "imdb", "tmdb", "tvdb", "simkl", "mal", "anilist", "anidb", "kitsu",
        ]
        .contains(&source)
        {
            return Ok(json!([]));
        }
        let id = data["id"].as_str().ok_or("ID ausente.")?;
        let hits = api
            .request(
                Method::GET,
                "/search/id",
                Some(&token),
                &[(source, id)],
                None,
            )
            .await
            .map_err(fail)?;
        let Some(hits) = hits.as_array() else {
            return Ok(json!([]));
        };
        let movie = data["type"] == "movie";
        let hit = hits
            .iter()
            .find(|hit| {
                if movie {
                    hit["type"] == "movie"
                } else {
                    hit["type"] == "show" || hit["type"] == "tv"
                }
            })
            .or(hits.first());
        let Some(hit) = hit else { return Ok(json!([])) };
        let kind = match hit["type"].as_str() {
            Some("movie") => "movies",
            Some("anime") => "anime",
            _ => "tv",
        };
        let Some(id) = hit.pointer("/ids/simkl").and_then(Value::as_u64) else {
            return Ok(json!([]));
        };
        let detail = api
            .request(
                Method::GET,
                &format!("/{kind}/{id}"),
                Some(&token),
                &[],
                None,
            )
            .await
            .map_err(fail)?;
        let items: Vec<_> = ["users_recommendations", "similar"]
            .iter()
            .filter_map(|key| detail[*key].as_array())
            .flatten()
            .collect();
        return Ok(json!(items));
    }
    if action == "remove_progress" {
        let id = data.as_u64().ok_or("Sessão de reprodução inválida.")?;
        return api
            .request(
                Method::DELETE,
                &format!("/sync/playback/{id}"),
                Some(&token),
                &[],
                None,
            )
            .await
            .map_err(fail);
    }
    let path = match action.as_str() {
        "library_add" => "/sync/add-to-list",
        "remove" | "history_remove" => "/sync/history/remove",
        "history_add" => "/sync/history",
        "rate" => "/sync/ratings",
        "unrate" => "/sync/ratings/remove",
        "start" => "/scrobble/start",
        "pause" => "/scrobble/pause",
        "stop" => "/scrobble/stop",
        _ => return Err("Operação Simkl desconhecida.".into()),
    };
    let result = api
        .request(Method::POST, path, Some(&token), &[], Some(&data))
        .await
        .map_err(fail)?;
    if result
        .get("not_found")
        .and_then(Value::as_object)
        .is_some_and(|v| {
            v.values()
                .any(|v| v.as_array().is_some_and(|a| !a.is_empty()))
        })
    {
        return Err("O Simkl não reconheceu este título.".into());
    }
    Ok(result)
}
