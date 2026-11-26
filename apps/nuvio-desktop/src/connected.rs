use super::{AppState, active_profile_idx, load_session_inner, load_settings_inner};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Duration;
use tauri::State;

const TB: &str = "https://api.torbox.app/v1/api";
const PM: &str = "https://www.premiumize.me";

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Torbox,
    Premiumize,
}

impl Provider {
    fn key(self) -> &'static str {
        match self {
            Self::Torbox => "torbox",
            Self::Premiumize => "premiumize",
        }
    }
}

fn scope(s: &AppState) -> String {
    let user = load_session_inner(&s.kv)
        .and_then(|sess| sess.user)
        .map(|u| u.id)
        .unwrap_or_else(|| "guest".into());
    let profile = active_profile_idx(&s.kv);
    format!("connected_services:{user}:{profile}")
}

fn token(s: &AppState, p: Provider) -> Result<String, String> {
    s.kv.get_string(&scope(s), p.key())
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Conecte sua conta primeiro.".into())
}

fn client() -> Result<Client, String> {
    nuvio_core::storage::dns::client_builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|_| "Falha ao iniciar conexão.".into())
}

async fn raw(req: RequestBuilder) -> Result<(reqwest::StatusCode, Value), String> {
    let resp = req
        .send()
        .await
        .map_err(|_| "Falha de conexão com o serviço.")?;
    let status = resp.status();
    let v = resp
        .json()
        .await
        .map_err(|_| "Resposta inválida do serviço.")?;
    Ok((status, v))
}

async fn call(req: RequestBuilder) -> Result<Value, String> {
    let (status, v) = raw(req).await?;
    if !status.is_success()
        || v["success"] == false
        || v["status"] == "error"
        || v["error"].as_str().is_some_and(|s| !s.is_empty())
    {
        return Err(message(&v));
    }
    Ok(v)
}

fn message(v: &Value) -> String {
    ["detail", "message", "error_description", "error"]
        .iter()
        .find_map(|k| v[k].as_str())
        .unwrap_or("Solicitação recusada pelo serviço.")
        .to_string()
}

fn field<'a>(v: &'a Value, k: &str) -> Result<&'a str, String> {
    v[k].as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("Resposta sem {k}."))
}

fn client_id(id: &str) -> Result<&str, String> {
    if !id.trim().is_empty() {
        return Ok(id.trim());
    }
    option_env!("PREMIUMIZE_CLIENT_ID")
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            "Informe o Client ID do seu aplicativo Premiumize ou conecte com sua chave de API."
                .into()
        })
}

#[tauri::command]
pub fn connected_accounts(state: State<'_, AppState>) -> Value {
    json!({"torbox":token(&state,Provider::Torbox).is_ok(),"premiumize":token(&state,Provider::Premiumize).is_ok()})
}

#[tauri::command]
pub fn connected_disconnect(state: State<'_, AppState>, provider: Provider) -> Result<(), String> {
    state
        .kv
        .remove(&scope(&state), provider.key())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn connected_use_key(
    state: State<'_, AppState>,
    provider: Provider,
    key: String,
) -> Result<(), String> {
    let ns = scope(&state);
    let key = key.trim();
    if key.is_empty() {
        return Err("Informe a chave.".into());
    }
    let path = match provider {
        Provider::Torbox => format!("{TB}/user/me"),
        Provider::Premiumize => format!("{PM}/api/account/info"),
    };
    call(client()?.get(path).bearer_auth(key)).await?;
    state
        .kv
        .put_string(&ns, provider.key(), Some(key))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn connected_start(
    state: State<'_, AppState>,
    provider: Provider,
    app_id: String,
) -> Result<Value, String> {
    let ns = scope(&state);
    let c = client()?;
    let (v, url, code) = match provider {
        Provider::Torbox => (
            call(
                c.get(format!("{TB}/user/auth/device/start"))
                    .query(&[("app", "Nuvio Desktop Rust")]),
            )
            .await?["data"]
                .clone(),
            "verification_url",
            "code",
        ),
        Provider::Premiumize => (
            call(c.post(format!("{PM}/token")).form(&[
                ("response_type", "device_code"),
                ("client_id", client_id(&app_id)?),
            ]))
            .await?,
            "verification_uri",
            "user_code",
        ),
    };
    let device = field(&v, "device_code")?;
    let verification = field(&v, url)?;
    let user_code = field(&v, code)?;
    state
        .kv
        .put_string(&ns, &format!("{}_pending", provider.key()), Some(device))
        .map_err(|e| e.to_string())?;
    Ok(
        json!({"url":verification,"code":user_code,"interval":v["interval"].as_u64().unwrap_or(5).max(1),"expires":v["expires_in"].as_u64().unwrap_or(600)}),
    )
}

#[tauri::command]
pub async fn connected_poll(
    state: State<'_, AppState>,
    provider: Provider,
    app_id: String,
) -> Result<String, String> {
    let ns = scope(&state);
    let key = format!("{}_pending", provider.key());
    let device = state
        .kv
        .get_string(&ns, &key)
        .map_err(|e| e.to_string())?
        .ok_or("Autorização expirada.")?;
    let c = client()?;
    let req = match provider {
        Provider::Torbox => c
            .post(format!("{TB}/user/auth/device/token"))
            .json(&json!({"device_code":device})),
        Provider::Premiumize => c.post(format!("{PM}/token")).form(&[
            ("grant_type", "device_code"),
            ("code", &device),
            ("client_id", client_id(&app_id)?),
        ]),
    };
    let (status, v) = raw(req).await?;
    let data = if provider == Provider::Torbox {
        &v["data"]
    } else {
        &v
    };
    if let Some(t) = data["access_token"]
        .as_str()
        .filter(|t| status.is_success() && !t.is_empty())
    {
        state
            .kv
            .put_string(&ns, provider.key(), Some(t))
            .map_err(|e| e.to_string())?;
        state.kv.remove(&ns, &key).map_err(|e| e.to_string())?;
        return Ok("connected".into());
    }
    let err = message(&v);
    let lower = format!("{} {}", err, v["error"]).to_lowercase();
    if provider == Provider::Torbox && matches!(status.as_u16(), 404 | 409 | 425) {
        return Ok("pending".into());
    }
    if lower.contains("slow_down") {
        return Ok("slow".into());
    }
    if [
        "pending",
        "not authorized",
        "not been used",
        "not used yet",
        "scan the code",
        "waiting",
        "not yet",
    ]
    .iter()
    .any(|s| lower.contains(s))
    {
        return Ok("pending".into());
    }
    Err(err)
}

fn scalar(v: &Value) -> Option<String> {
    v.as_str()
        .map(str::to_owned)
        .or_else(|| v.as_u64().map(|n| n.to_string()))
}

fn video(f: &Value) -> bool {
    let name = ["name", "path", "short_name", "absolute_path"]
        .iter()
        .filter_map(|k| f[k].as_str());
    name.into_iter().any(|s| {
        s.rsplit_once('.').is_some_and(|(_, ext)| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "mp4"
                    | "mkv"
                    | "webm"
                    | "avi"
                    | "mov"
                    | "mpeg"
                    | "mpg"
                    | "ts"
                    | "m2ts"
                    | "mts"
                    | "wmv"
                    | "ogv"
                    | "ogm"
                    | "flv"
                    | "m4v"
                    | "3gp"
                    | "3g2"
                    | "divx"
            )
        })
    }) || ["mime_type", "mimetype"]
        .iter()
        .any(|k| f[k].as_str().is_some_and(|s| s.starts_with("video/")))
}

#[derive(Serialize, Deserialize)]
pub struct CloudFile {
    provider: Provider,
    kind: String,
    item: String,
    file: String,
    name: String,
}

#[tauri::command]
pub async fn connected_files(
    state: State<'_, AppState>,
    provider: Provider,
) -> Result<Vec<CloudFile>, String> {
    if !load_settings_inner(&state.kv).connected.library {
        return Err("Biblioteca na nuvem desativada.".into());
    }
    let t = token(&state, provider)?;
    let c = client()?;
    let mut result = Vec::new();
    if provider == Provider::Premiumize {
        let data = call(c.get(format!("{PM}/api/item/listall")).bearer_auth(&t)).await?;
        for f in data["files"]
            .as_array()
            .ok_or("Lista de arquivos inválida.")?
            .iter()
            .filter(|f| video(f))
        {
            if let (Some(id), Some(name)) =
                (scalar(&f["id"]), f["path"].as_str().or(f["name"].as_str()))
            {
                result.push(CloudFile {
                    provider,
                    kind: "file".into(),
                    item: String::new(),
                    file: id,
                    name: name.to_owned(),
                });
            }
        }
    } else {
        for kind in ["torrents", "usenet", "webdl"] {
            let mut seen = std::collections::HashSet::new();
            let mut offset = 0;
            loop {
                let data = call(
                    c.get(format!("{TB}/{kind}/mylist"))
                        .bearer_auth(&t)
                        .query(&[("offset", offset), ("limit", 100)]),
                )
                .await?;
                let rows = data["data"].as_array().ok_or("Lista do Torbox inválida.")?;
                let before = seen.len();
                for item in rows {
                    let Some(id) = scalar(&item["id"]) else {
                        continue;
                    };
                    if !seen.insert(id.clone()) {
                        continue;
                    }
                    for f in item["files"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter(|f| video(f))
                    {
                        if let (Some(file), Some(name)) = (
                            scalar(&f["id"]),
                            ["short_name", "name", "absolute_path"]
                                .iter()
                                .find_map(|k| f[k].as_str()),
                        ) {
                            result.push(CloudFile {
                                provider,
                                kind: kind.into(),
                                item: id.clone(),
                                file,
                                name: name.into(),
                            });
                        }
                    }
                }
                if rows.len() < 100 || before == seen.len() {
                    break;
                }
                offset += rows.len();
            }
        }
    }
    result.sort_by_cached_key(|f| f.name.to_lowercase());
    Ok(result)
}

fn playable_url(s: &str) -> Result<String, String> {
    url::Url::parse(s)
        .ok()
        .filter(|u| matches!(u.scheme(), "http" | "https") && u.host().is_some())
        .map(|u| u.to_string())
        .ok_or_else(|| "Link de reprodução inválido.".into())
}

async fn file_url(c: &Client, t: &str, f: &CloudFile) -> Result<String, String> {
    if f.provider == Provider::Premiumize {
        let v = call(
            c.get(format!("{PM}/api/item/details"))
                .bearer_auth(t)
                .query(&[("id", &f.file)]),
        )
        .await?;
        return playable_url(field(&v, "link")?);
    }
    let id_param = match f.kind.as_str() {
        "torrents" => "torrent_id",
        "usenet" => "usenet_id",
        "webdl" => "web_id",
        _ => return Err("Tipo de arquivo inválido.".into()),
    };
    let v = call(
        c.get(format!("{TB}/{}/requestdl", f.kind))
            .bearer_auth(t)
            .query(&[
                ("token", t),
                (id_param, &f.item),
                ("file_id", &f.file),
                ("zip_link", "false"),
                ("redirect", "false"),
            ]),
    )
    .await?;
    playable_url(field(&v, "data")?)
}

#[tauri::command]
pub async fn connected_play(state: State<'_, AppState>, file: CloudFile) -> Result<String, String> {
    if !load_settings_inner(&state.kv).connected.library {
        return Err("Biblioteca na nuvem desativada.".into());
    }
    file_url(&client()?, &token(&state, file.provider)?, &file).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveRequest {
    magnet: String,
    file_idx: Option<usize>,
    season: Option<u32>,
    episode: Option<u32>,
}

fn choose<'a>(files: &'a [Value], req: &ResolveRequest, torbox: bool) -> Result<&'a Value, String> {
    if let Some(i) = req.file_idx {
        let id = i.to_string();
        let entry = if torbox {
            files
                .iter()
                .find(|f| scalar(&f["id"]).as_deref() == Some(id.as_str()))
        } else {
            files.get(i)
        };
        if let Some(f) = entry.filter(|f| video(f)) {
            return Ok(f);
        }
    }
    let candidates: Vec<_> = files.iter().filter(|f| video(f)).collect();
    if let [f] = candidates.as_slice() {
        return Ok(f);
    }
    if let (Some(s), Some(e)) = (req.season, req.episode) {
        let patterns = [format!("s{s:02}e{e:02}"), format!("{s}x{e:02}")];
        if let Some(f) = candidates.iter().find(|f| {
            ["name", "path", "short_name"]
                .iter()
                .filter_map(|k| f[k].as_str())
                .any(|n| {
                    patterns.iter().any(|p| {
                        n.to_ascii_lowercase().match_indices(p).any(|(idx, _)| {
                            !n.as_bytes()
                                .get(idx + p.len())
                                .is_some_and(u8::is_ascii_digit)
                        })
                    })
                })
        }) {
            return Ok(*f);
        }
        return Err("Episódio não identificado no pacote.".into());
    }
    candidates
        .into_iter()
        .max_by_key(|f| f["size"].as_u64().unwrap_or(0))
        .ok_or_else(|| "Nenhum vídeo disponível.".into())
}

async fn resolve(c: &Client, t: &str, p: Provider, r: &ResolveRequest) -> Result<String, String> {
    if p == Provider::Premiumize {
        let v = call(
            c.post(format!("{PM}/api/transfer/directdl"))
                .bearer_auth(t)
                .form(&[("src", &r.magnet)]),
        )
        .await?;
        let files = v["content"]
            .as_array()
            .ok_or("Nenhum arquivo disponível no Premiumize.")?;
        let f = choose(files, r, false)?;
        return playable_url(field(f, "link")?);
    }
    let boundary = format!("nuvio{:016x}", rand::random::<u64>());
    let mut body = String::new();
    for (name, value) in [
        ("magnet", r.magnet.as_str()),
        ("add_only_if_cached", "true"),
        ("allow_zip", "false"),
    ] {
        body.push_str(&format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"
        ));
    }
    body.push_str(&format!("--{boundary}--\r\n"));
    let v = call(
        c.post(format!("{TB}/torrents/createtorrent"))
            .bearer_auth(t)
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(body),
    )
    .await?;
    let item = scalar(&v["data"]["torrent_id"])
        .or_else(|| scalar(&v["data"]["id"]))
        .ok_or("Torrent indisponível no Torbox.")?;
    let v = call(
        c.get(format!("{TB}/torrents/mylist"))
            .bearer_auth(t)
            .query(&[("id", item.as_str()), ("bypass_cache", "true")]),
    )
    .await?;
    let files = v["data"]["files"]
        .as_array()
        .ok_or("Arquivos indisponíveis no Torbox.")?;
    let f = choose(files, r, true)?;
    let file = scalar(&f["id"]).ok_or("Arquivo inválido.")?;
    file_url(
        c,
        t,
        &CloudFile {
            provider: p,
            item,
            file,
            kind: "torrents".into(),
            name: String::new(),
        },
    )
    .await
}

#[tauri::command]
pub async fn connected_resolve(
    state: State<'_, AppState>,
    request: ResolveRequest,
) -> Result<String, String> {
    if !load_settings_inner(&state.kv).connected.resolve
        || !request.magnet.starts_with("magnet:?")
        || request.magnet.contains(['\r', '\n'])
    {
        return Err("Resolução de magnet indisponível.".into());
    }
    let c = client()?;
    let mut errors = Vec::new();
    for p in [Provider::Torbox, Provider::Premiumize] {
        if let Ok(t) = token(&state, p) {
            match resolve(&c, &t, p, &request).await {
                Ok(url) => return Ok(url),
                Err(e) => errors.push(format!("{}: {e}", p.key())),
            }
        }
    }
    Err(if errors.is_empty() {
        "Conecte Torbox ou Premiumize para resolver links.".into()
    } else {
        errors.join(" · ")
    })
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Preferences {
    pub library: bool,
    pub resolve: bool,
    pub premiumize_app_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Preview {
    pub enabled: bool,
    pub key: String,
    pub hover: bool,
    pub drag: bool,
    pub keyboard: bool,
    pub count: u8,
    pub size: u8,
}
impl Default for Preview {
    fn default() -> Self {
        Self {
            enabled: false,
            key: String::new(),
            hover: true,
            drag: true,
            keyboard: true,
            count: 5,
            size: 100,
        }
    }
}

#[tauri::command]
pub async fn seekr_track(
    state: State<'_, AppState>,
    id: String,
    season: Option<u32>,
    episode: Option<u32>,
    duration_ms: u64,
) -> Result<Value, String> {
    let settings = load_settings_inner(&state.kv).seekr;
    if !settings.enabled || settings.key.is_empty() || duration_ms == 0 {
        return Err("Prévias Seekr desativadas ou sem chave.".into());
    }
    let imdb = id
        .strip_prefix("tt")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|c| c.is_ascii_digit()));
    if !imdb && id.parse::<u64>().is_err() {
        return Err("Identificador indisponível para Seekr.".into());
    }
    let mut params = vec![("duration_ms", duration_ms.to_string())];
    let name = match (season, episode) {
        (Some(s), Some(e)) => {
            params.extend([("season", s.to_string()), ("episode", e.to_string())]);
            if imdb { "show_imdb_id" } else { "show_tmdb_id" }
        }
        (None, None) => {
            if imdb {
                "imdb_id"
            } else {
                "tmdb_id"
            }
        }
        _ => return Err("Episódio sem identificação.".into()),
    };
    params.push((name, id));
    let v = call(
        client()?
            .get("https://api.seekr.tv/sprites")
            .header("X-API-Key", &settings.key)
            .query(&params),
    )
    .await?;
    let url = url::Url::parse("https://sprites.seekr.tv/")
        .unwrap()
        .join(field(&v, "vtt_url")?)
        .map_err(|_| "Endereço Seekr inválido.")?;
    if url.scheme() != "https" || url.host_str() != Some("sprites.seekr.tv") {
        return Err("Servidor de miniaturas inválido.".into());
    }
    Ok(
        json!({"url":url,"scale":v["scale"].as_f64().filter(|n|*n>0.0).unwrap_or(1.0),"count":settings.count,"size":settings.size,"hover":settings.hover,"drag":settings.drag,"keyboard":settings.keyboard}),
    )
}
