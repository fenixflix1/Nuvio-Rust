use crate::Result;
use directories::BaseDirs;
use std::fs;
use std::path::{Path, PathBuf};

/// Resolves platform-specific config and cache directories matching `DesktopStorage.kt`.
#[derive(Debug, Clone)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl AppPaths {
    /// Resolves Nuvio directories using `XDG_*` on Linux and `APPDATA`/`LOCALAPPDATA` on Windows.
    pub fn resolve() -> Result<Self> {
        let base = BaseDirs::new();

        #[cfg(target_os = "windows")]
        let (config_dir, cache_dir) = {
            let roaming = std::env::var_os("APPDATA")
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
                .or_else(|| base.as_ref().map(|b| b.config_dir().to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."));
            let local = std::env::var_os("LOCALAPPDATA")
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
                .or_else(|| base.as_ref().map(|b| b.cache_dir().to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."));
            (roaming.join("Nuvio"), local.join("Nuvio").join("Cache"))
        };

        #[cfg(target_os = "macos")]
        let (config_dir, cache_dir) = {
            let home = base
                .as_ref()
                .map(|b| b.home_dir().to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            (
                home.join("Library/Application Support/Nuvio"),
                home.join("Library/Caches/Nuvio"),
            )
        };

        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        let (config_dir, cache_dir) = {
            let xdg_config = std::env::var_os("XDG_CONFIG_HOME")
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
                .or_else(|| base.as_ref().map(|b| b.config_dir().to_path_buf()))
                .unwrap_or_else(|| PathBuf::from(".config"));
            let xdg_cache = std::env::var_os("XDG_CACHE_HOME")
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
                .or_else(|| base.as_ref().map(|b| b.cache_dir().to_path_buf()))
                .unwrap_or_else(|| PathBuf::from(".cache"));
            (xdg_config.join("nuvio"), xdg_cache.join("nuvio"))
        };

        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            config_dir,
            cache_dir,
        })
    }

    pub fn from_root(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        Self::from_dirs(root.join("config"), root.join("cache"))
    }

    pub fn from_dirs(config_dir: PathBuf, cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(&cache_dir)?;
        Ok(Self {
            config_dir,
            cache_dir,
        })
    }

    pub fn db_path(&self) -> PathBuf {
        self.config_dir.join("nuvio.sqlite3")
    }

    pub fn cache_db_path(&self) -> PathBuf {
        self.cache_dir.join("nuvio_cache.redb")
    }
}
