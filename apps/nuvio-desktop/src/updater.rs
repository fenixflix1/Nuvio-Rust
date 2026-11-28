use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const GITHUB_LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/fenixflix1/Nuvio-Rust/releases/latest";
const MIN_BINARY_BYTES: u64 = 1_048_576;

#[derive(Debug, Clone, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    html_url: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_name: String,
    pub release_notes: String,
    pub release_url: String,
    pub asset_name: Option<String>,
    pub asset_url: Option<String>,
    pub asset_size: Option<u64>,
    pub can_self_install: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percent: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemVer {
    major: u64,
    minor: u64,
    patch: u64,
    pre: Option<String>,
}

fn parse_semver(raw: &str) -> Option<SemVer> {
    let trimmed = raw.trim().trim_start_matches(['v', 'V']);
    let (core, pre) = match trimmed.split_once('-') {
        Some((c, p)) => (c, Some(p.trim().to_string()).filter(|s| !s.is_empty())),
        None => (trimmed, None),
    };
    let core = core.split('+').next().unwrap_or(core);
    let mut parts = core.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    let patch = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(SemVer {
        major,
        minor,
        patch,
        pre,
    })
}

fn is_newer_version(current: &str, candidate: &str) -> bool {
    let (Some(cur), Some(cand)) = (parse_semver(current), parse_semver(candidate)) else {
        return false;
    };
    match (cand.major, cand.minor, cand.patch).cmp(&(cur.major, cur.minor, cur.patch)) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => match (&cur.pre, &cand.pre) {
            (Some(_), None) => true,
            (Some(a), Some(b)) => b > a,
            _ => false,
        },
    }
}

fn select_platform_asset<'a>(
    assets: &'a [GitHubAsset],
    os: &str,
    arch: &str,
) -> Option<&'a GitHubAsset> {
    match os {
        "linux" => {
            let appimages: Vec<&GitHubAsset> = assets
                .iter()
                .filter(|a| a.name.to_ascii_lowercase().ends_with(".appimage"))
                .collect();
            if arch == "x86_64" {
                appimages
                    .iter()
                    .copied()
                    .find(|a| {
                        let lower = a.name.to_ascii_lowercase();
                        lower.contains("x86_64") || lower.contains("amd64")
                    })
                    .or_else(|| appimages.first().copied())
            } else {
                appimages
                    .iter()
                    .copied()
                    .find(|a| a.name.to_ascii_lowercase().contains(arch))
                    .or_else(|| appimages.first().copied())
            }
        }
        "windows" => {
            let exes: Vec<&GitHubAsset> = assets
                .iter()
                .filter(|a| a.name.to_ascii_lowercase().ends_with(".exe"))
                .collect();
            exes.iter()
                .copied()
                .find(|a| a.name.eq_ignore_ascii_case("NuvioDesktop-Rust.exe"))
                .or_else(|| exes.first().copied())
        }
        "android" => {
            let apks: Vec<&GitHubAsset> = assets
                .iter()
                .filter(|a| a.name.to_ascii_lowercase().ends_with(".apk"))
                .collect();
            if arch == "aarch64" {
                apks.iter()
                    .copied()
                    .find(|a| {
                        let lower = a.name.to_ascii_lowercase();
                        lower.contains("arm64") || lower.contains("aarch64")
                    })
                    .or_else(|| {
                        apks.iter()
                            .copied()
                            .find(|a| a.name.to_ascii_lowercase().contains("universal"))
                    })
                    .or_else(|| apks.first().copied())
            } else {
                apks.iter()
                    .copied()
                    .find(|a| a.name.to_ascii_lowercase().contains("universal"))
                    .or_else(|| apks.first().copied())
            }
        }
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn current_appimage_path() -> Option<PathBuf> {
    std::env::var_os("APPIMAGE")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .filter(|p| p.exists())
}

fn can_self_install_on_host(asset: Option<&GitHubAsset>) -> bool {
    let Some(a) = asset else {
        return false;
    };
    if !is_allowed_download_url(&a.browser_download_url) {
        return false;
    }
    #[cfg(target_os = "linux")]
    {
        current_appimage_path().is_some()
    }
    #[cfg(target_os = "windows")]
    {
        std::env::current_exe().is_ok_and(|p| p.exists())
    }
    #[cfg(all(not(target_os = "linux"), not(target_os = "windows")))]
    {
        false
    }
}

fn is_allowed_download_url(raw: &str) -> bool {
    let lower = raw.trim().to_ascii_lowercase();
    lower.starts_with("https://github.com/")
        || lower.starts_with("https://objects.githubusercontent.com/")
        || lower.starts_with("https://release-assets.githubusercontent.com/")
}

fn validate_downloaded_binary(head: &[u8], total_len: u64, os: &str) -> Result<(), String> {
    if total_len < MIN_BINARY_BYTES {
        return Err(format!(
            "Arquivo de atualização incompleto ({total_len} bytes)."
        ));
    }
    match os {
        "linux" if !head.starts_with(b"\x7fELF") => {
            return Err("O arquivo baixado não é um AppImage ELF válido.".to_string());
        }
        "windows" if !head.starts_with(b"MZ") => {
            return Err("O arquivo baixado não é um executável Windows válido.".to_string());
        }
        _ => {}
    }
    Ok(())
}

fn build_http_client(timeout_secs: u64) -> Result<reqwest::Client, String> {
    nuvio_core::storage::dns::client_builder()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent(concat!("NuvioDesktop-Rust/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())
}

pub async fn check_for_updates() -> Result<AppUpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let client = build_http_client(12)?;
    let resp = client
        .get(GITHUB_LATEST_RELEASE_URL)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("Falha ao consultar atualizações: {e}"))?
        .error_for_status()
        .map_err(|e| format!("GitHub Releases retornou erro: {e}"))?;

    let release: GitHubRelease = resp
        .json()
        .await
        .map_err(|e| format!("Resposta inválida do GitHub Releases: {e}"))?;

    if release.draft || release.prerelease {
        return Ok(AppUpdateInfo {
            current_version: current_version.clone(),
            latest_version: current_version,
            update_available: false,
            release_name: String::new(),
            release_notes: String::new(),
            release_url: release.html_url,
            asset_name: None,
            asset_url: None,
            asset_size: None,
            can_self_install: false,
        });
    }

    let latest_version = release
        .tag_name
        .trim()
        .trim_start_matches(['v', 'V'])
        .to_string();
    let update_available = is_newer_version(&current_version, &release.tag_name);
    let asset = select_platform_asset(&release.assets, std::env::consts::OS, std::env::consts::ARCH);
    let can_self_install = update_available && can_self_install_on_host(asset);

    Ok(AppUpdateInfo {
        current_version,
        latest_version,
        update_available,
        release_name: release
            .name
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| release.tag_name.clone()),
        release_notes: release.body.unwrap_or_default().trim().to_string(),
        release_url: release.html_url,
        asset_name: asset.map(|a| a.name.clone()),
        asset_url: asset.map(|a| a.browser_download_url.clone()),
        asset_size: asset.map(|a| a.size),
        can_self_install,
    })
}

async fn download_update_to(
    app: &AppHandle,
    url: &str,
    dest: &Path,
    expected_size: Option<u64>,
) -> Result<(), String> {
    if !is_allowed_download_url(url) {
        return Err("URL de download não permitida para atualização automática.".to_string());
    }
    let client = build_http_client(300)?;
    let mut resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Falha ao baixar atualização: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Erro no download da atualização: {e}"))?;

    let total_bytes = resp
        .content_length()
        .or(expected_size)
        .unwrap_or(0);

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = File::create(dest).map_err(|e| format!("Falha ao criar arquivo temporário: {e}"))?;

    let mut downloaded_bytes: u64 = 0;
    let mut head = Vec::with_capacity(16);
    let mut last_pct: i64 = -1;

    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| format!("Erro durante download: {e}"))?
    {
        if head.len() < 16 {
            let take = (16 - head.len()).min(chunk.len());
            head.extend_from_slice(&chunk[..take]);
        }
        file.write_all(&chunk)
            .map_err(|e| format!("Erro ao gravar atualização: {e}"))?;
        downloaded_bytes += chunk.len() as u64;

        let percent = if total_bytes > 0 {
            ((downloaded_bytes as f64 / total_bytes as f64) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };
        let pct_int = percent.floor() as i64;
        if pct_int != last_pct || downloaded_bytes == total_bytes {
            last_pct = pct_int;
            let _ = app.emit(
                "app:update_progress",
                AppUpdateProgress {
                    downloaded_bytes,
                    total_bytes,
                    percent,
                },
            );
        }
    }

    file.flush().map_err(|e| e.to_string())?;
    drop(file);

    if let Err(err) = validate_downloaded_binary(&head, downloaded_bytes, std::env::consts::OS) {
        let _ = std::fs::remove_file(dest);
        return Err(err);
    }

    Ok(())
}

pub async fn download_and_install_update(
    app: &AppHandle,
    cache_dir: &Path,
    asset_url: &str,
    expected_size: Option<u64>,
) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let _ = cache_dir;
        let appimage_path = current_appimage_path()
            .ok_or_else(|| "Instalação automática disponível apenas rodando via AppImage.".to_string())?;
        let parent = appimage_path
            .parent()
            .ok_or_else(|| "Diretório do AppImage inválido.".to_string())?;
        let file_name = appimage_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("NuvioDesktop-Rust.AppImage");
        let tmp_path = parent.join(format!(".{file_name}.{}.tmp", std::process::id()));

        if let Err(err) = download_update_to(app, asset_url, &tmp_path, expected_size).await {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(err);
        }

        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("Falha ao definir permissão executável: {e}"))?;

        if let Err(err) = std::fs::rename(&tmp_path, &appimage_path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("Falha ao substituir o AppImage: {err}"));
        }

        Command::new(&appimage_path)
            .spawn()
            .map_err(|e| format!("Atualizado, mas falhou ao reiniciar AppImage: {e}"))?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let current_exe =
            std::env::current_exe().map_err(|e| format!("Falha ao localizar executável: {e}"))?;
        let updates_dir = cache_dir.join("updates");
        let new_exe = updates_dir.join(format!("NuvioDesktop-Rust.{}.new.exe", std::process::id()));

        if let Err(err) = download_update_to(app, asset_url, &new_exe, expected_size).await {
            let _ = std::fs::remove_file(&new_exe);
            return Err(err);
        }

        let script_path = updates_dir.join(format!("apply-update-{}.cmd", std::process::id()));
        let pid = std::process::id();
        let script = format!(
            "@echo off\r\n\
            set \"PID={pid}\"\r\n\
            set \"SRC={}\"\r\n\
            set \"DST={}\"\r\n\
            set /a TRIES=0\r\n\
            :waitloop\r\n\
            set /a TRIES+=1\r\n\
            if %TRIES% GTR 40 goto replace\r\n\
            tasklist /FI \"PID eq %PID%\" 2>NUL | find /I \"%PID%\" >NUL\r\n\
            if not errorlevel 1 (\r\n\
              timeout /T 1 /NOBREAK >NUL\r\n\
              goto waitloop\r\n\
            )\r\n\
            :replace\r\n\
            move /Y \"%SRC%\" \"%DST%\" >NUL 2>&1\r\n\
            if errorlevel 1 (\r\n\
              copy /Y \"%SRC%\" \"%DST%\" >NUL 2>&1\r\n\
              del /F /Q \"%SRC%\" >NUL 2>&1\r\n\
            )\r\n\
            start \"\" \"%DST%\"\r\n\
            del /F /Q \"%~f0\" >NUL 2>&1\r\n",
            new_exe.display(),
            current_exe.display()
        );
        std::fs::write(&script_path, script)
            .map_err(|e| format!("Falha ao preparar script de atualização: {e}"))?;

        Command::new("cmd.exe")
            .args(["/C", &script_path.to_string_lossy()])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("Falha ao iniciar atualização do Windows: {e}"))?;
        Ok(())
    }

    #[cfg(all(not(target_os = "linux"), not(target_os = "windows")))]
    {
        let _ = (app, cache_dir, asset_url, expected_size);
        Err("Atualização automática não suportada nesta plataforma.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GitHubAsset, is_allowed_download_url, is_newer_version, parse_semver,
        select_platform_asset, validate_downloaded_binary,
    };

    #[test]
    fn parses_and_compares_semver_tags() {
        assert!(parse_semver("v0.0.4").is_some());
        assert!(is_newer_version("0.0.4", "v0.0.5"));
        assert!(is_newer_version("0.0.4", "0.1.0"));
        assert!(is_newer_version("0.0.4-rc.1", "0.0.4"));
        assert!(!is_newer_version("0.0.4", "v0.0.4"));
        assert!(!is_newer_version("0.0.4", "v0.0.3"));
        assert!(!is_newer_version("0.0.4", "v0.0.4-beta"));
        assert!(is_newer_version("0.0.4", "v0.0.5-beta"));
    }

    #[test]
    fn selects_expected_linux_and_windows_assets() {
        let assets = vec![
            GitHubAsset {
                name: "NuvioDesktop-Rust.exe".to_string(),
                browser_download_url: "https://github.com/fenixflix1/Nuvio-Rust/releases/download/v0.0.5/NuvioDesktop-Rust.exe".to_string(),
                size: 12_000_000,
            },
            GitHubAsset {
                name: "NuvioDesktop-Rust-x86_64.AppImage".to_string(),
                browser_download_url: "https://github.com/fenixflix1/Nuvio-Rust/releases/download/v0.0.5/NuvioDesktop-Rust-x86_64.AppImage".to_string(),
                size: 40_000_000,
            },
        ];

        assert_eq!(
            select_platform_asset(&assets, "linux", "x86_64").map(|a| a.name.as_str()),
            Some("NuvioDesktop-Rust-x86_64.AppImage")
        );
        assert_eq!(
            select_platform_asset(&assets, "windows", "x86_64").map(|a| a.name.as_str()),
            Some("NuvioDesktop-Rust.exe")
        );
    }

    #[test]
    fn validates_download_urls_and_binary_headers() {
        assert!(is_allowed_download_url(
            "https://github.com/fenixflix1/Nuvio-Rust/releases/download/v0.0.5/NuvioDesktop-Rust.exe"
        ));
        assert!(!is_allowed_download_url("http://github.com/foo"));
        assert!(!is_allowed_download_url("https://evil.example.com/payload.exe"));

        assert!(validate_downloaded_binary(b"\x7fELF\x02\x01", 2_000_000, "linux").is_ok());
        assert!(validate_downloaded_binary(b"<html>404", 2_000_000, "linux").is_err());
        assert!(validate_downloaded_binary(b"MZ\x90\x00", 2_000_000, "windows").is_ok());
        assert!(validate_downloaded_binary(b"MZ\x90\x00", 128, "windows").is_err());
    }
}
