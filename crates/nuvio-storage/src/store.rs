use crate::Result;
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Persistent namespaced key-value store backed by SQLite (bundled, zero external deps).
/// Mirrors the API of `DesktopStorage.Store`.
#[derive(Clone)]
pub struct KvStore {
    conn: Arc<Mutex<Connection>>,
}

impl KvStore {
    pub fn open(db_path: impl AsRef<Path>) -> Result<Self> {
        if let Some(parent) = db_path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            CREATE TABLE IF NOT EXISTS kv_store (
                namespace TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
                PRIMARY KEY (namespace, key)
            );
            ",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS kv_store (
                namespace TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
                PRIMARY KEY (namespace, key)
            );
            ",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn namespace(&self, name: impl Into<String>) -> NamespacedStore {
        NamespacedStore {
            store: self.clone(),
            namespace: name.into(),
        }
    }

    pub fn contains(&self, namespace: &str, key: &str) -> Result<bool> {
        let conn = self.conn.lock().expect("kv mutex poisoned");
        let exists: Option<i64> = conn
            .query_row(
                "SELECT 1 FROM kv_store WHERE namespace = ?1 AND key = ?2 LIMIT 1",
                params![namespace, key],
                |row| row.get(0),
            )
            .optional()?;
        Ok(exists.is_some())
    }

    pub fn get_string(&self, namespace: &str, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().expect("kv mutex poisoned");
        conn.query_row(
            "SELECT value FROM kv_store WHERE namespace = ?1 AND key = ?2",
            params![namespace, key],
            |row| row.get(0),
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn put_string(&self, namespace: &str, key: &str, value: Option<&str>) -> Result<()> {
        match value {
            Some(val) => {
                let conn = self.conn.lock().expect("kv mutex poisoned");
                conn.execute(
                    "
                    INSERT INTO kv_store (namespace, key, value, updated_at)
                    VALUES (?1, ?2, ?3, unixepoch())
                    ON CONFLICT(namespace, key) DO UPDATE SET
                        value = excluded.value,
                        updated_at = unixepoch()
                    ",
                    params![namespace, key, val],
                )?;
            }
            None => self.remove(namespace, key)?,
        }
        Ok(())
    }

    pub fn remove(&self, namespace: &str, key: &str) -> Result<()> {
        let conn = self.conn.lock().expect("kv mutex poisoned");
        conn.execute(
            "DELETE FROM kv_store WHERE namespace = ?1 AND key = ?2",
            params![namespace, key],
        )?;
        Ok(())
    }

    pub fn clear_namespace(&self, namespace: &str) -> Result<()> {
        let conn = self.conn.lock().expect("kv mutex poisoned");
        conn.execute(
            "DELETE FROM kv_store WHERE namespace = ?1",
            params![namespace],
        )?;
        Ok(())
    }

    pub fn wipe_all(&self) -> Result<()> {
        let conn = self.conn.lock().expect("kv mutex poisoned");
        conn.execute("DELETE FROM kv_store", [])?;
        Ok(())
    }

    /// Imports key=value entries from a legacy Java `.properties` file created by `DesktopStorage`.
    pub fn import_properties_file(&self, namespace: &str, path: impl AsRef<Path>) -> Result<usize> {
        let content = std::fs::read_to_string(path)?;
        let mut count = 0usize;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
                continue;
            }
            if let Some((raw_k, raw_v)) = trimmed.split_once('=') {
                let key = unescape_properties(raw_k.trim());
                let val = unescape_properties(raw_v);
                self.put_string(namespace, &key, Some(&val))?;
                count += 1;
            }
        }
        Ok(count)
    }
}

#[derive(Clone)]
pub struct NamespacedStore {
    store: KvStore,
    namespace: String,
}

impl NamespacedStore {
    pub fn contains(&self, key: &str) -> Result<bool> {
        self.store.contains(&self.namespace, key)
    }

    pub fn get_string(&self, key: &str) -> Result<Option<String>> {
        self.store.get_string(&self.namespace, key)
    }

    pub fn put_string(&self, key: &str, value: Option<&str>) -> Result<()> {
        self.store.put_string(&self.namespace, key, value)
    }

    pub fn get_bool(&self, key: &str) -> Result<Option<bool>> {
        Ok(self.get_string(key)?.and_then(|v| v.parse::<bool>().ok()))
    }

    pub fn put_bool(&self, key: &str, value: bool) -> Result<()> {
        self.put_string(key, Some(if value { "true" } else { "false" }))
    }

    pub fn get_int(&self, key: &str) -> Result<Option<i64>> {
        Ok(self.get_string(key)?.and_then(|v| v.parse::<i64>().ok()))
    }

    pub fn put_int(&self, key: &str, value: i64) -> Result<()> {
        self.put_string(key, Some(&value.to_string()))
    }

    pub fn get_float(&self, key: &str) -> Result<Option<f64>> {
        Ok(self.get_string(key)?.and_then(|v| v.parse::<f64>().ok()))
    }

    pub fn put_float(&self, key: &str, value: f64) -> Result<()> {
        self.put_string(key, Some(&value.to_string()))
    }

    pub fn get_string_set(&self, key: &str) -> Result<Option<BTreeSet<String>>> {
        let Some(raw) = self.get_string(key)? else {
            return Ok(None);
        };
        let list: Vec<String> = serde_json::from_str(&raw)?;
        Ok(Some(list.into_iter().collect()))
    }

    pub fn put_string_set(&self, key: &str, values: &BTreeSet<String>) -> Result<()> {
        let list: Vec<&String> = values.iter().collect();
        let encoded = serde_json::to_string(&list)?;
        self.put_string(key, Some(&encoded))
    }

    pub fn remove(&self, key: &str) -> Result<()> {
        self.store.remove(&self.namespace, key)
    }

    pub fn clear(&self) -> Result<()> {
        self.store.clear_namespace(&self.namespace)
    }
}

fn unescape_properties(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some(other) => out.push(other),
                None => break,
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_kv_types() {
        let kv = KvStore::open_in_memory().unwrap();
        let ns = kv.namespace("player_settings");
        ns.put_bool("hardware_decoding", true).unwrap();
        ns.put_int("volume", 85).unwrap();
        ns.put_float("speed", 1.25).unwrap();
        let set: BTreeSet<String> = ["pt-BR", "en"].into_iter().map(str::to_string).collect();
        ns.put_string_set("sub_langs", &set).unwrap();

        assert_eq!(ns.get_bool("hardware_decoding").unwrap(), Some(true));
        assert_eq!(ns.get_int("volume").unwrap(), Some(85));
        assert_eq!(ns.get_float("speed").unwrap(), Some(1.25));
        assert_eq!(ns.get_string_set("sub_langs").unwrap(), Some(set));
        assert!(ns.contains("volume").unwrap());

        ns.remove("volume").unwrap();
        assert!(!ns.contains("volume").unwrap());
    }
}
