use crate::{Result, StorageError};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

const CACHE_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("nuvio_ttl_cache");

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    expires_at_epoch_secs: Option<u64>,
    payload: Vec<u8>,
}

/// Pure-Rust embedded disk cache with TTL support backed by `redb`.
#[derive(Clone)]
pub struct CacheStore {
    db: Arc<Database>,
}

impl CacheStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::create(path).map_err(|e| StorageError::Redb(e.to_string()))?;
        let write_txn = db
            .begin_write()
            .map_err(|e| StorageError::Redb(e.to_string()))?;
        {
            let _ = write_txn
                .open_table(CACHE_TABLE)
                .map_err(|e| StorageError::Redb(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::Redb(e.to_string()))?;
        Ok(Self { db: Arc::new(db) })
    }

    pub fn set(&self, key: &str, payload: &[u8], ttl_secs: Option<u64>) -> Result<()> {
        let expires_at_epoch_secs = ttl_secs.map(|ttl| now_secs().saturating_add(ttl));
        let entry = CacheEntry {
            expires_at_epoch_secs,
            payload: payload.to_vec(),
        };
        let encoded = serde_json::to_vec(&entry)?;
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::Redb(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(CACHE_TABLE)
                .map_err(|e| StorageError::Redb(e.to_string()))?;
            table
                .insert(key, encoded.as_slice())
                .map_err(|e| StorageError::Redb(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::Redb(e.to_string()))?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::Redb(e.to_string()))?;
        let table = read_txn
            .open_table(CACHE_TABLE)
            .map_err(|e| StorageError::Redb(e.to_string()))?;
        let Some(guard) = table
            .get(key)
            .map_err(|e| StorageError::Redb(e.to_string()))?
        else {
            return Ok(None);
        };
        let entry: CacheEntry = serde_json::from_slice(guard.value())?;
        if let Some(exp) = entry.expires_at_epoch_secs
            && now_secs() > exp
        {
            drop(guard);
            drop(table);
            drop(read_txn);
            let _ = self.remove(key);
            return Ok(None);
        }
        Ok(Some(entry.payload))
    }

    pub fn remove(&self, key: &str) -> Result<()> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::Redb(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(CACHE_TABLE)
                .map_err(|e| StorageError::Redb(e.to_string()))?;
            table
                .remove(key)
                .map_err(|e| StorageError::Redb(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::Redb(e.to_string()))?;
        Ok(())
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
