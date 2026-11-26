use serde::Serialize;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU8, Ordering};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio::time::{Duration, timeout};

trait Transport: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Transport for T {}
type Socket = Box<dyn Transport>;

/// Outcome of one `discord_presence` / `discord_probe` call, including what the
/// IPC layer actually saw so the UI can show why nothing reached Discord.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordReport {
    pub ok: bool,
    pub message: String,
    /// Every `discord*-ipc-*` endpoint that exists in the runtime/temp dirs.
    pub sockets: Vec<String>,
    /// Endpoint that answered the handshake, when one did.
    pub endpoint: Option<String>,
    /// `READY` or the handshake failure for the chosen endpoint.
    pub handshake: String,
    /// Whether the activity was accepted with or without artwork.
    pub activity: String,
    /// Step-by-step log: per-socket connect/handshake result and activity attempts.
    pub details: Vec<String>,
}

impl DiscordReport {
    fn new() -> Self {
        Self {
            ok: false,
            message: String::new(),
            sockets: Vec::new(),
            endpoint: None,
            handshake: String::new(),
            activity: String::new(),
            details: Vec::new(),
        }
    }
}

// 0 = artwork not tested yet, 1 = accepted, 2 = rejected (send text-only first).
static ASSET_SUPPORT: AtomicU8 = AtomicU8::new(0);

// Discord forks (Canary, PTB, Vesktop, Serein and other clients) reuse the
// `discord-ipc-N` contract, either as official sockets or inside their own
// runtime directory. Collect every candidate instead of giving up on the first
// endpoint that accepts a connection but does not answer the handshake.
#[cfg(unix)]
fn socket_paths() -> Vec<std::path::PathBuf> {
    use std::path::PathBuf;
    let mut roots: Vec<PathBuf> = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"]
        .iter()
        .filter_map(std::env::var_os)
        .map(PathBuf::from)
        .collect();
    roots.push(PathBuf::from("/tmp"));
    // Flatpak clients keep their runtime dir nested inside the sandbox prefix:
    //   /run/user/1000/.flatpak/<app-id>/xdg-run/discord-ipc-0
    //   ~/.var/app/<app-id>/config/.../discord-ipc-0
    // Without these roots a sandboxed client (Serein, Vesktop, GoofCord…) is
    // never discovered even while it is running with game activity enabled.
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        roots.push(home.join(".var/app"));
        roots.push(home.join(".config"));
    }
    #[cfg(target_os = "linux")]
    if let Ok(uid_raw) = std::fs::read_to_string("/proc/self/loginuid")
        && let Ok(uid) = uid_raw.trim().parse::<u32>()
    {
        let fallback_run = PathBuf::from(format!("/run/user/{uid}"));
        if !roots.contains(&fallback_run) {
            roots.push(fallback_run);
        }
    }
    let mut paths = Vec::new();
    for root in roots {
        collect_sockets(&root, 0, &mut paths);
    }
    paths.sort();
    paths.dedup();
    paths
}

/// Walks the runtime roots and their `app/*` subdirectories (Flatpak, Snap and
/// fork runtimes such as Serein) so every `discord*-ipc-N` endpoint is found.
#[cfg(unix)]
fn collect_sockets(dir: &std::path::Path, depth: usize, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("discord") && name.contains("-ipc-") {
            out.push(entry.path());
            continue;
        }
        if depth < 4
            && !matches!(name.as_ref(), "node_modules" | "target" | "cache" | ".git")
            && entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false)
        {
            collect_sockets(&entry.path(), depth + 1, out);
        }
    }
}

#[cfg(unix)]
async fn connect_candidates() -> Vec<(String, Result<Socket, String>)> {
    let mut out = Vec::new();
    for path in socket_paths() {
        let label = path.display().to_string();
        let result = match tokio::net::UnixStream::connect(&path).await {
            Ok(stream) => Ok(Box::new(stream) as Socket),
            Err(err) => Err(err.to_string()),
        };
        out.push((label, result));
    }
    out
}

#[cfg(windows)]
async fn connect_candidates() -> Vec<(String, Result<Socket, String>)> {
    let mut out = Vec::new();
    for prefix in [
        "discord-ipc",
        "discordcanary-ipc",
        "discordptb-ipc",
        "discord-development-ipc",
    ] {
        for i in 0..10 {
            let name = format!(r"\\.\pipe\{prefix}-{i}");
            match tokio::net::windows::named_pipe::ClientOptions::new().open(&name) {
                Ok(pipe) => out.push((name, Ok(Box::new(pipe) as Socket))),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => out.push((name, Err(err.to_string()))),
            }
        }
    }
    out
}

async fn send(socket: &mut Socket, op: u32, body: &[u8]) -> std::io::Result<()> {
    socket.write_u32_le(op).await?;
    socket.write_u32_le(body.len() as u32).await?;
    socket.write_all(body).await
}

async fn exchange(socket: &mut Socket, op: u32, msg: Value) -> Result<Value, ()> {
    send(socket, op, &serde_json::to_vec(&msg).map_err(|_| ())?)
        .await
        .map_err(|_| ())?;
    loop {
        let opcode = socket.read_u32_le().await.map_err(|_| ())?;
        let size = socket.read_u32_le().await.map_err(|_| ())?;
        if size > 65536 {
            return Err(());
        }
        let mut body = vec![0; size as usize];
        socket.read_exact(&mut body).await.map_err(|_| ())?;
        match opcode {
            0 => continue,
            1 => {
                let response: Value = serde_json::from_slice(&body).map_err(|_| ())?;
                // Discord can interleave dispatch events with a command response.
                if op == 0 || response["nonce"] == msg["nonce"] {
                    return Ok(response);
                }
            }
            3 => send(socket, 4, &body).await.map_err(|_| ())?,
            _ => return Err(()),
        }
    }
}

fn summarize(frame: &Value) -> String {
    if frame["evt"] == "ERROR" {
        let code = frame["data"]["code"].as_i64().unwrap_or(0);
        let message = frame["data"]["message"].as_str().unwrap_or("");
        return format!("erro {code}: {message}");
    }
    let text = frame.to_string();
    if text.chars().count() > 300 {
        format!("{}…", text.chars().take(300).collect::<String>())
    } else {
        text
    }
}

/// Tries the v1 handshake on every candidate and keeps the first one that answers `READY`.
async fn open_connection(report: &mut DiscordReport) -> Option<(String, Socket)> {
    open_connection_with(connect_candidates().await, report).await
}

async fn open_connection_with(
    candidates: Vec<(String, Result<Socket, String>)>,
    report: &mut DiscordReport,
) -> Option<(String, Socket)> {
    if candidates.is_empty() {
        report.details.push("Nenhum arquivo discord*-ipc-* encontrado nos diretórios de runtime.".into());
        return None;
    }
    for (path, result) in candidates {
        report.sockets.push(path.clone());
        let mut socket = match result {
            Ok(socket) => socket,
            Err(err) => {
                report.details.push(format!("{path}: falha ao conectar ({err})"));
                continue;
            }
        };
        let ready = timeout(
            Duration::from_millis(1200),
            exchange(
                &mut socket,
                0,
                json!({"v":1,"client_id":"1538974392376369212"}),
            ),
        )
        .await;
        match ready {
            Ok(Ok(frame)) if frame["evt"] == "READY" => {
                report.details.push(format!("{path}: handshake READY"));
                return Some((path, socket));
            }
            Ok(Ok(frame)) => {
                report.details.push(format!("{path}: handshake recusado ({})", summarize(&frame)));
            }
            Ok(Err(())) => {
                report.details.push(format!("{path}: conexão aceita, mas sem resposta válida"));
            }
            Err(_) => {
                report.details.push(format!("{path}: conexão aceita, mas sem resposta (timeout)"));
            }
        }
    }
    None
}

async fn set_activity(socket: &mut Socket, activity: &Value) -> Result<(), String> {
    let nonce = rand::random::<u64>().to_string();
    let response = exchange(socket, 1, json!({
        "cmd":"SET_ACTIVITY","nonce":nonce,"args":{"pid":std::process::id(),"activity":activity}
    }))
    .await
    .map_err(|_| "o cliente encerrou a conexão".to_string())?;
    if response["nonce"] != nonce {
        return Err(format!("resposta fora de ordem ({})", summarize(&response)));
    }
    if response["evt"] == "ERROR" {
        return Err(summarize(&response));
    }
    Ok(())
}

fn without_assets(activity: &Value) -> Value {
    let mut text_only = activity.clone();
    if let Some(fields) = text_only.as_object_mut() {
        fields.remove("assets");
    }
    text_only
}

static CONNECTION: Mutex<Option<Socket>> = Mutex::const_new(None);

#[tauri::command]
pub async fn discord_presence(activity: Option<Value>) -> DiscordReport {
    let mut slot = CONNECTION.lock().await;
    let mut report = DiscordReport::new();
    let Some(activity) = activity else {
        slot.take(); // Closing the IPC session removes its activity.
        report.ok = true;
        report.message = "Presença removida.".into();
        return report;
    };

    let inner = timeout(Duration::from_secs(12), async {
        let mut socket = match slot.take() {
            Some(socket) => {
                report.details.push("Reutilizando a conexão IPC já aberta.".into());
                socket
            }
            None => match open_connection(&mut report).await {
                Some((path, socket)) => {
                    report.endpoint = Some(path);
                    report.handshake = "READY".into();
                    socket
                }
                None => {
                    report.message = if report.sockets.is_empty() {
                        "Nenhum canal discord-ipc encontrado. Abra o Discord; no Serein, ative o compartilhamento de atividade (Game Activity) antes de abrir o Nuvio.".into()
                    } else {
                        "Encontrei canal(is) discord-ipc, mas nenhum respondeu READY ao handshake.".into()
                    };
                    return;
                }
            },
        };

        // Forks like Serein only accept artwork registered in the Discord app, so
        // try text-only first once the client has rejected external images.
        let assets_rejected = ASSET_SUPPORT.load(Ordering::Relaxed) == 2;
        let first = if assets_rejected {
            without_assets(&activity)
        } else {
            activity.clone()
        };
        match set_activity(&mut socket, &first).await {
            Ok(()) => {
                if assets_rejected {
                    report.details.push("Atividade sem imagens: aceita.".into());
                    report.activity = "aceita (sem imagens)".into();
                } else {
                    ASSET_SUPPORT.store(1, Ordering::Relaxed);
                    report.details.push("Atividade com imagens: aceita.".into());
                    report.activity = "aceita (com imagens)".into();
                }
                *slot = Some(socket);
                report.ok = true;
                report.message = format!(
                    "Presença publicada via {}.",
                    report.endpoint.clone().unwrap_or_else(|| "canal IPC".into())
                );
                return;
            }
            Err(err) => {
                report.details.push(format!("Atividade com imagens recusada: {err}"));
            }
        }

        match set_activity(&mut socket, &without_assets(&activity)).await {
            Ok(()) => {
                ASSET_SUPPORT.store(2, Ordering::Relaxed);
                report.details.push("Atividade sem imagens: aceita.".into());
                report.activity = "aceita (sem imagens)".into();
                *slot = Some(socket);
                report.ok = true;
                report.message =
                    "Presença publicada sem imagens: o cliente não aceita artes externas (comum no Serein).".into();
            }
            Err(err) => {
                report.details.push(format!("Atividade sem imagens recusada: {err}"));
                report.message = format!("O cliente recusou a atividade: {err}");
            }
        }
    })
    .await;

    if inner.is_err() {
        report.ok = false;
        report.message = "Tempo esgotado ao falar com o Discord.".into();
    }
    if !report.ok {
        slot.take();
    }
    report
}

/// Opens a throwaway IPC session and only reports what the client answered.
#[tauri::command]
pub async fn discord_probe() -> DiscordReport {
    let mut report = DiscordReport::new();
    let inner = timeout(Duration::from_secs(8), async {
        match open_connection(&mut report).await {
            Some((path, mut socket)) => {
                report.endpoint = Some(path.clone());
                report.handshake = "READY".into();
                // Verify the full path, not just the handshake: try to publish a
                // minimal activity and then clear it. Serein accepts SET_ACTIVITY
                // but only for registered application IDs, so a failure here
                // explains why presence never appears even with READY.
                let test_activity = json!({
                    "type": 3,
                    "details": "Nuvio",
                    "state": "Testando presença"
                });
                match set_activity(&mut socket, &test_activity).await {
                    Ok(()) => {
                        report.activity = "aceita (teste)".into();
                        report.details.push("SET_ACTIVITY de teste: aceito.".into());
                        // Clear the test activity so we don't leave junk behind.
                        let _ = set_activity(&mut socket, &Value::Null).await;
                    }
                    Err(err) => {
                        report.details.push(format!("SET_ACTIVITY de teste recusado: {err}"));
                        report.activity = format!("recusada: {err}");
                    }
                }
                report.ok = true;
                report.message = format!("Canal encontrado e handshake READY em {path}.");
            }
            None => {
                report.message = if report.sockets.is_empty() {
                    "Nenhum canal discord-ipc encontrado. Abra o Discord; no Serein, ative o compartilhamento de atividade (Game Activity) antes de abrir o Nuvio.".into()
                } else {
                    "Canal(is) discord-ipc encontrado(s), mas nenhum respondeu READY ao handshake.".into()
                };
            }
        }
    })
    .await;

    if inner.is_err() {
        report.ok = false;
        report.message = "Tempo esgotado ao procurar canais discord-ipc.".into();
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dispatch_and_ping_can_precede_activity_acknowledgement() {
        let (client, peer) = tokio::io::duplex(1024);
        let mut client: Socket = Box::new(client);
        let mut peer: Socket = Box::new(peer);
        let remote = tokio::spawn(async move {
            assert_eq!(peer.read_u32_le().await.unwrap(), 1);
            let mut request = vec![0; peer.read_u32_le().await.unwrap() as usize];
            peer.read_exact(&mut request).await.unwrap();
            send(&mut peer, 3, b"{}").await.unwrap();
            send(&mut peer, 1, br#"{"evt":"DISPATCH"}"#).await.unwrap();
            send(&mut peer, 1, br#"{"nonce":"test","cmd":"SET_ACTIVITY"}"#)
                .await
                .unwrap();
            assert_eq!(peer.read_u32_le().await.unwrap(), 4);
        });
        let ack = exchange(&mut client, 1, json!({"nonce":"test"}))
            .await
            .unwrap();
        assert_eq!(ack["cmd"], "SET_ACTIVITY");
        remote.await.unwrap();
    }

    #[test]
    fn summarizes_error_frames() {
        let frame = json!({"evt":"ERROR","data":{"code":4000,"message":"Invalid Asset"}});
        assert_eq!(summarize(&frame), "erro 4000: Invalid Asset");
        let ok = json!({"cmd":"SET_ACTIVITY"});
        assert_eq!(summarize(&ok), "{\"cmd\":\"SET_ACTIVITY\"}");
    }

    #[test]
    fn strips_assets_for_text_only_fallback() {
        let activity = json!({"type":3,"details":"Título","assets":{"large_image":"https://x/y.jpg"}});
        let stripped = without_assets(&activity);
        assert!(stripped.get("assets").is_none());
        assert_eq!(stripped["details"], "Título");
    }

    async fn fake_endpoint(reply: &'static [u8]) -> (String, Result<Socket, String>) {
        let (client, peer) = tokio::io::duplex(4096);
        let mut peer: Socket = Box::new(peer);
        tokio::spawn(async move {
            assert_eq!(peer.read_u32_le().await.unwrap(), 0);
            let size = peer.read_u32_le().await.unwrap() as usize;
            let mut body = vec![0; size];
            peer.read_exact(&mut body).await.unwrap();
            send(&mut peer, 1, reply).await.unwrap();
            tokio::time::sleep(Duration::from_millis(200)).await;
        });
        ("fake-ipc".to_string(), Ok(Box::new(client) as Socket))
    }

    #[tokio::test]
    async fn reports_endpoint_that_answers_ready() {
        let mut report = DiscordReport::new();
        let endpoint = fake_endpoint(br#"{"evt":"READY","data":{}}"#).await;
        let chosen = open_connection_with(vec![endpoint], &mut report).await;
        assert!(chosen.is_some());
        assert_eq!(report.sockets, vec!["fake-ipc"]);
        assert!(report.details.iter().any(|line| line.contains("handshake READY")));
    }

    #[tokio::test]
    async fn reports_handshake_error_from_client() {
        let mut report = DiscordReport::new();
        let endpoint = fake_endpoint(
            br#"{"evt":"ERROR","data":{"code":4000,"message":"Invalid Client ID"}}"#,
        )
        .await;
        let chosen = open_connection_with(vec![endpoint], &mut report).await;
        assert!(chosen.is_none());
        assert_eq!(report.sockets, vec!["fake-ipc"]);
        assert!(
            report
                .details
                .iter()
                .any(|line| line.contains("erro 4000: Invalid Client ID"))
        );
    }
}

#[cfg(all(test, unix))]
mod flatpak_tests {
    use super::collect_sockets;

    #[test]
    fn finds_sockets_nested_in_flatpak_runtime() {
        let root = std::env::temp_dir().join(format!("nuvio-ipc-{}", std::process::id()));
        let nested = root.join(".flatpak/cz.viceverse.serein/xdg-run");
        std::fs::create_dir_all(&nested).expect("tmp dirs");
        let socket = nested.join("discord-ipc-0");
        std::fs::write(&socket, b"").expect("fake socket");
        let mut found = Vec::new();
        collect_sockets(&root, 0, &mut found);
        assert!(
            found.contains(&socket),
            "socket do Flatpak não encontrado: {found:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
