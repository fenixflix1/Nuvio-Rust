pub mod cache;
pub mod crypto;
pub mod paths;
pub mod store;

pub use cache::CacheStore;
pub use crypto::{
    AesMode, DigestAlgorithm, aes_decrypt, aes_encrypt, digest, get_random_values, pbkdf2_derive,
    sha256_hex,
};
pub use paths::AppPaths;
pub use store::KvStore;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Redb error: {0}")]
    Redb(String),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Crypto error: {0}")]
    Crypto(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;
