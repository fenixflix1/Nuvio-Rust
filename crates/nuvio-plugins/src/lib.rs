pub mod dom;
pub mod runtime;

pub use dom::DomBridge;
pub use runtime::{
    PluginCallArgs, PluginExecOptions, PluginRuntime, PluginStreamResult, TmdbCacheStats,
    clear_tmdb_cache, prefetch_tmdb_for_plugins, reset_tmdb_cache_stats, resolve_imdb_to_tmdb,
    tmdb_cache_stats,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("JS execution error: {0}")]
    Js(String),
    #[error("Storage error: {0}")]
    Storage(#[from] nuvio_storage::StorageError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Execution timed out after {0}ms")]
    Timeout(u64),
}

pub type Result<T> = std::result::Result<T, PluginError>;
