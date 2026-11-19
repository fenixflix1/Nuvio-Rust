pub mod ffi;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const DEFAULT_TRACKERS: &[&str] = &[
    "udp://tracker.opentrackr.org:1337/announce",
    "udp://open.stealth.si:80/announce",
    "udp://tracker.torrent.eu.org:451/announce",
    "udp://tracker.bittor.pw:1337/announce",
    "udp://public.popcorn-tracker.org:6969/announce",
    "udp://tracker.dler.org:6969/announce",
    "udp://exodus.desync.com:6969/announce",
    "udp://open.demonii.com:1337/announce",
];

#[derive(Debug, thiserror::Error)]
pub enum P2pError {
    #[error("Native library load error: {0}")]
    LibraryLoad(#[from] libloading::Error),
    #[error("Native library not found at any expected location")]
    LibraryNotFound,
    #[error("Invalid infoHash: {0}")]
    InvalidInfoHash(String),
    #[error("Engine error: {0}")]
    Engine(String),
}

pub type Result<T> = std::result::Result<T, P2pError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum P2pTorrentProfile {
    Soft,
    Balanced,
    Fast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum P2pCacheSize {
    None,
    Gb2,
    Gb5,
    Gb10,
}

impl P2pCacheSize {
    pub const fn bytes(self) -> u64 {
        match self {
            Self::None => 0,
            Self::Gb2 => 2 * 1024 * 1024 * 1024,
            Self::Gb5 => 5 * 1024 * 1024 * 1024,
            Self::Gb10 => 10 * 1024 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pSettings {
    pub p2p_enabled: bool,
    pub enable_upload: bool,
    pub hide_torrent_stats: bool,
    pub torrent_profile: P2pTorrentProfile,
    pub cache_size: P2pCacheSize,
}

impl Default for P2pSettings {
    fn default() -> Self {
        Self {
            p2p_enabled: false,
            enable_upload: true,
            hide_torrent_stats: false,
            torrent_profile: P2pTorrentProfile::Balanced,
            cache_size: P2pCacheSize::Gb2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pStreamRequest {
    pub info_hash: String,
    pub file_idx: Option<i32>,
    pub filename: Option<String>,
    pub trackers: Vec<String>,
}

/// Normalizes a 40-char hex or base32 infoHash into lowercase.
pub fn canonical_info_hash(raw: &str) -> Result<String> {
    let clean = raw.trim().to_ascii_lowercase();
    if clean.len() != 40 && clean.len() != 32 {
        return Err(P2pError::InvalidInfoHash(raw.to_string()));
    }
    Ok(clean)
}

/// Builds a magnet URI combining `DEFAULT_TRACKERS` and request-specific trackers.
pub fn build_magnet_uri(info_hash: &str, extra_trackers: &[String]) -> Result<String> {
    let hash = canonical_info_hash(info_hash)?;
    let mut uri = format!("magnet:?xt=urn:btih:{hash}");
    let mut seen = Vec::<String>::new();
    for tr in DEFAULT_TRACKERS
        .iter()
        .copied()
        .map(str::to_string)
        .chain(extra_trackers.iter().cloned())
    {
        let trimmed = tr.trim().to_string();
        if !trimmed.is_empty() && !seen.contains(&trimmed) {
            seen.push(trimmed.clone());
            let encoded = url::form_urlencoded::byte_serialize(trimmed.as_bytes()).collect::<String>();
            uri.push_str("&tr=");
            uri.push_str(&encoded);
        }
    }
    Ok(uri)
}

/// Resolves the platform-specific `nuvio_engine` dynamic library filename.
pub fn platform_library_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "nuvio_engine.dll"
    }
    #[cfg(target_os = "macos")]
    {
        "libnuvio_engine.dylib"
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        "libnuvio_engine.so"
    }
}

/// Searches standard Nuvio desktop native paths (walking parent directories) for
/// `libnuvio_engine.so` / `nuvio_engine.dll`.
pub fn find_bundled_engine_library(start_dir: impl AsRef<Path>) -> Option<PathBuf> {
    let lib_name = platform_library_name();
    let os_dir = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };

    let rel_path = PathBuf::from("composeApp/src/desktopMain/native")
        .join(os_dir)
        .join("engine")
        .join(lib_name);

    for ancestor in start_dir.as_ref().ancestors() {
        let candidate = ancestor.join(&rel_path);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    let direct = PathBuf::from(lib_name);
    direct.is_file().then_some(direct)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_magnet_with_default_and_extra_trackers() {
        let uri = build_magnet_uri(
            "0123456789ABCDEF0123456789ABCDEF01234567",
            &["udp://custom.tracker:6969/announce".to_string()],
        )
        .unwrap();
        assert!(uri.starts_with("magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567"));
        assert!(uri.contains("tracker.opentrackr.org"));
        assert!(uri.contains("custom.tracker"));
    }
}
