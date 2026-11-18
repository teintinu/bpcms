//! Stable identifier ids and the application lock file (`bpcms.lock`).
//!
//! The lock is JSON Lines: one identifier per line, sorted by `path`, so git
//! merges stay reviewable. Removing the file resets every id.
//!
//! Ids are base64url strings restricted to identifier-safe characters
//! (`[A-Za-z_][A-Za-z0-9_]*`) and never start with a digit, so a builder can
//! reuse them as names in another language.

use std::collections::{BTreeMap, BTreeSet};
use std::hash::{BuildHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const LOCK_FILE: &str = "bpcms.lock";
pub const LOCK_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockEntry {
    /// Stable id used by the AST and by builders.
    pub id: String,
    /// Safe, machine-independent path: `<service-uri>#<kind>/<Name>[/...]`.
    pub path: String,
    pub kind: String,
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct Lock {
    pub application: String,
    /// path -> entry
    pub entries: BTreeMap<String, LockEntry>,
    pub dirty: bool,
}

impl Lock {
    pub fn new(application: impl Into<String>) -> Self {
        Self {
            application: application.into(),
            entries: BTreeMap::new(),
            dirty: false,
        }
    }

    /// Reads a lock file; `application` is checked against the file when present.
    pub fn load(path: &Path, application: &str) -> Result<Self, String> {
        let mut lock = Lock::new(application);
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(lock),
            Err(error) => return Err(format!("cannot read {}: {error}", path.display())),
        };
        let mut ids = BTreeSet::new();
        for (index, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let entry: LockEntry = serde_json::from_str(line).map_err(|error| {
                format!(
                    "{}:{}: invalid lock line: {error}",
                    path.display(),
                    index + 1
                )
            })?;
            if entry.path.is_empty() || entry.id.is_empty() {
                return Err(format!(
                    "{}:{}: lock entry needs `id` and `path`",
                    path.display(),
                    index + 1
                ));
            }
            if !is_safe_id(&entry.id) {
                return Err(format!(
                    "{}:{}: id `{}` is not identifier-safe",
                    path.display(),
                    index + 1,
                    entry.id
                ));
            }
            if !ids.insert(entry.id.clone()) {
                return Err(format!(
                    "{}:{}: duplicate id `{}` (resolve the merge, then rerun)",
                    path.display(),
                    index + 1,
                    entry.id
                ));
            }
            if lock
                .entries
                .insert(entry.path.clone(), entry.clone())
                .is_some()
            {
                return Err(format!(
                    "{}:{}: duplicate path `{}`",
                    path.display(),
                    index + 1,
                    entry.path
                ));
            }
        }
        Ok(lock)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        }
        let mut body = String::new();
        for entry in self.entries.values() {
            let line = serde_json::to_string(entry)
                .map_err(|error| format!("cannot serialize lock entry: {error}"))?;
            body.push_str(&line);
            body.push('\n');
        }
        std::fs::write(path, body)
            .map_err(|error| format!("cannot write {}: {error}", path.display()))
    }

    /// Returns the id for `path`, creating it when absent.
    pub fn ensure(&mut self, path: &str, kind: &str, name: &str) -> String {
        if let Some(entry) = self.entries.get(path) {
            return entry.id.clone();
        }
        let used: BTreeSet<String> = self
            .entries
            .values()
            .map(|entry| entry.id.clone())
            .collect();
        let id = new_id(&used);
        self.entries.insert(
            path.to_string(),
            LockEntry {
                id: id.clone(),
                path: path.to_string(),
                kind: kind.to_string(),
                name: name.to_string(),
            },
        );
        self.dirty = true;
        id
    }

    /// Drops entries that are no longer declared. Returns the removed entries.
    pub fn retain(&mut self, used_paths: &BTreeSet<String>) -> Vec<LockEntry> {
        let stale: Vec<String> = self
            .entries
            .keys()
            .filter(|path| !used_paths.contains(*path))
            .cloned()
            .collect();
        let mut removed = Vec::new();
        for path in stale {
            if let Some(entry) = self.entries.remove(&path) {
                removed.push(entry);
            }
        }
        if !removed.is_empty() {
            self.dirty = true;
        }
        removed
    }

    pub fn entry_for(&self, path: &str) -> Option<&LockEntry> {
        self.entries.get(path)
    }
}

pub fn lock_path_for_application(application_file: &Path) -> PathBuf {
    application_file
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(LOCK_FILE)
}

pub fn is_safe_id(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// A random id in base64url, mapped to identifier-safe characters.
pub fn new_id(used: &BTreeSet<String>) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    loop {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0);
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
        (nanos, counter).hash(&mut hasher);
        let first = hasher.finish();
        (counter, first).hash(&mut hasher);
        let second = hasher.finish();
        let mut bytes = [0u8; 9];
        bytes[..8].copy_from_slice(&first.to_le_bytes());
        bytes[8] = second as u8;
        let id = base64_identifier(&bytes);
        if !id.is_empty() && is_safe_id(&id) && !used.contains(&id) {
            return id;
        }
    }
}

/// Base64url without padding, with `-` mapped to `_` so the result is a valid
/// identifier in most target languages.
pub fn base64_identifier(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(bytes.len() * 2);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        let value = ((b0 as u32) << 16) | ((b1 as u32) << 8) | b2 as u32;
        out.push(ALPHABET[((value >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((value >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[((value >> 6) & 63) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(value & 63) as usize] as char);
        }
    }
    out.replace('-', "_")
}

// --- canonical identifier paths -----------------------------------------

pub fn service_path(uri: &str) -> String {
    format!("{uri}#service")
}

pub fn artifact_path(uri: &str, artifact_kind: &str, name: &str) -> String {
    format!("{uri}#{}/{}", artifact_kind.to_ascii_lowercase(), name)
}

pub fn field_path(uri: &str, document: &str, field: &str) -> String {
    format!("{uri}#document/{document}/field/{field}")
}

pub fn value_path(uri: &str, shape: &str, value: &str) -> String {
    format!("{uri}#shape/{shape}/value/{value}")
}

pub fn param_path(uri: &str, artifact_kind: &str, artifact: &str, param: &str) -> String {
    format!(
        "{uri}#{}/{artifact}/param/{param}",
        artifact_kind.to_ascii_lowercase()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_ids_are_identifier_safe() {
        let mut used = BTreeSet::new();
        for _ in 0..200 {
            let id = new_id(&used);
            assert!(is_safe_id(&id), "{id}");
            assert!(!id.chars().next().unwrap().is_ascii_digit());
            assert!(used.insert(id));
        }
    }

    #[test]
    fn base64_maps_to_identifier_characters() {
        let encoded = base64_identifier(&[0xff, 0xfe, 0xfd, 0x00]);
        assert!(encoded
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_'));
        assert!(!encoded.contains('-'));
    }

    #[test]
    fn lock_roundtrip_is_one_entry_per_line() {
        let dir = std::env::temp_dir().join(format!("archol-lock-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(LOCK_FILE);
        let mut lock = Lock::new("example.com/app/demo");
        let id_a = lock.ensure(&service_path("example.com/services/a"), "service", "a");
        let id_b = lock.ensure(
            &value_path("example.com/services/a", "Pay", "Card"),
            "value",
            "Card",
        );
        lock.save(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<_> = text.lines().filter(|line| !line.is_empty()).collect();
        assert_eq!(lines.len(), 2);
        assert!(lines.iter().all(|line| line.starts_with("{\"id\":")));
        let loaded = Lock::load(&path, "example.com/app/demo").unwrap();
        assert_eq!(loaded.entries.len(), 2);
        assert_eq!(
            loaded
                .entry_for(&service_path("example.com/services/a"))
                .unwrap()
                .id,
            id_a
        );
        assert_eq!(
            loaded
                .entry_for(&value_path("example.com/services/a", "Pay", "Card"))
                .unwrap()
                .id,
            id_b
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn retain_prunes_only_unused_paths() {
        let mut lock = Lock::new("app");
        let keep = service_path("svc/a");
        let drop = artifact_path("svc/a", "Document", "Gone");
        lock.ensure(&keep, "service", "a");
        lock.ensure(&drop, "document", "Gone");
        let used = BTreeSet::from([keep.clone()]);
        let removed = lock.retain(&used);
        assert_eq!(removed.len(), 1);
        assert!(lock.entry_for(&keep).is_some());
        assert!(lock.entry_for(&drop).is_none());
    }

    #[test]
    fn rejects_duplicate_ids_on_load() {
        let dir = std::env::temp_dir().join(format!("archol-lock-dup-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(LOCK_FILE);
        std::fs::write(
            &path,
            "{\"id\":\"Abc\",\"path\":\"a#service\",\"kind\":\"service\",\"name\":\"a\"}\n\
             {\"id\":\"Abc\",\"path\":\"b#service\",\"kind\":\"service\",\"name\":\"b\"}\n",
        )
        .unwrap();
        let error = Lock::load(&path, "app").unwrap_err();
        assert!(error.contains("duplicate id"), "{error}");
        let _ = std::fs::remove_dir_all(dir);
    }
}
