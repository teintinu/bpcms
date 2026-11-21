//! Stable identifier ids and the application lock file.
//!
//! The lock lives next to the application file with only the extension changed
//! (`<app>-application.archol` → `<app>-application.lock`). It is a plain CSV
//! with two columns, `id,path`, one identifier per line. Removing the file
//! resets every id.
//!
//! Ids are allocated **at the end of compilation, only for used identifiers**,
//! most referenced first.
//! Generated ids are sequential lowercase base-36 strings (letters and digits,
//! first character always a letter) starting at `a`.
//! A `*` at the end of the path's identifier name pins the entry: it keeps its
//! id, stays at the top of the file, and is never pruned.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const LOCK_EXTENSION: &str = "lock";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockEntry {
    /// Stable id used by the AST and by builders.
    pub id: String,
    /// Safe path, optionally ending with `*` when pinned.
    pub path: String,
}

impl LockEntry {
    pub fn base_path(&self) -> &str {
        self.path.strip_suffix('*').unwrap_or(&self.path)
    }

    pub fn pinned(&self) -> bool {
        self.path.ends_with('*')
    }
}

#[derive(Clone, Debug, Default)]
pub struct Lock {
    pub application: String,
    /// base path -> entry
    pub entries: BTreeMap<String, LockEntry>,
    /// Pinned base paths in file order; written first on save.
    pinned_order: Vec<String>,
    pub dirty: bool,
}

impl Lock {
    pub fn new(application: impl Into<String>) -> Self {
        Self {
            application: application.into(),
            entries: BTreeMap::new(),
            pinned_order: Vec::new(),
            dirty: false,
        }
    }

    /// Reads a CSV lock; a missing file yields an empty lock.
    pub fn load(lock_path: &Path, application: &str) -> Result<Self, String> {
        let mut lock = Lock::new(application);
        let text = match std::fs::read_to_string(lock_path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(lock),
            Err(error) => return Err(format!("cannot read {}: {error}", lock_path.display())),
        };
        let mut ids = BTreeSet::new();
        for (index, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line == "id,path" {
                continue;
            }
            // Older versions wrote JSON Lines; read them once and save CSV later.
            let (id, path, legacy) = if line.starts_with('{') {
                let value: serde_json::Value = serde_json::from_str(line).map_err(|error| {
                    format!(
                        "{}:{}: invalid legacy lock line: {error}",
                        lock_path.display(),
                        index + 1
                    )
                })?;
                let id = value
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let mut path = value
                    .get("path")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
                    .to_string();
                if value
                    .get("manual")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
                    && !path.ends_with('*')
                {
                    path.push('*');
                }
                (id, path, true)
            } else {
                let (id, path) = line.split_once(',').ok_or_else(|| {
                    format!(
                        "{}:{}: lock line must be `id,path`",
                        lock_path.display(),
                        index + 1
                    )
                })?;
                (id.trim().to_string(), path.trim().to_string(), false)
            };
            if legacy {
                lock.dirty = true;
            }
            if path.contains(',') {
                return Err(format!(
                    "{}:{}: path must not contain a comma",
                    lock_path.display(),
                    index + 1
                ));
            }
            let (id, path) = (id.trim(), path.trim());
            if id.is_empty() || path.is_empty() {
                return Err(format!(
                    "{}:{}: lock entry needs `id` and `path`",
                    lock_path.display(),
                    index + 1
                ));
            }
            if !is_safe_id(id) {
                return Err(format!(
                    "{}:{}: id `{id}` is not identifier-safe",
                    lock_path.display(),
                    index + 1
                ));
            }
            if !ids.insert(id.to_string()) {
                return Err(format!(
                    "{}:{}: duplicate id `{id}` (resolve the merge, then rerun)",
                    lock_path.display(),
                    index + 1
                ));
            }
            let base = path.trim_end_matches('*').to_string();
            if base.is_empty() {
                return Err(format!(
                    "{}:{}: lock entry needs a path",
                    lock_path.display(),
                    index + 1
                ));
            }
            let entry = LockEntry {
                id: id.to_string(),
                path: path.to_string(),
            };
            if lock.entries.insert(base.clone(), entry.clone()).is_some() {
                return Err(format!(
                    "{}:{}: duplicate path `{base}`",
                    lock_path.display(),
                    index + 1
                ));
            }
            if entry.pinned() {
                lock.pinned_order.push(base);
            }
        }
        Ok(lock)
    }

    /// Writes `id,path` lines: pinned entries first, then generated by id.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        }
        let mut body = String::new();
        for base in &self.pinned_order {
            if let Some(entry) = self.entries.get(base) {
                if entry.pinned() {
                    write_entry(&mut body, entry)?;
                }
            }
        }
        let mut generated: Vec<&LockEntry> = self
            .entries
            .values()
            .filter(|entry| !entry.pinned())
            .collect();
        generated
            .sort_by_key(|entry| (id_rank(&entry.id).unwrap_or(usize::MAX), entry.path.clone()));
        for entry in generated {
            write_entry(&mut body, entry)?;
        }
        std::fs::write(path, body)
            .map_err(|error| format!("cannot write {}: {error}", path.display()))
    }

    /// Assigns ids only to `used` paths; drops unused generated entries.
    /// New ids go first to the most referenced paths (`usage` value), breaking
    /// ties by path. Pinned entries keep their id. Returns `path -> id`.
    pub fn assign(&mut self, usage: &BTreeMap<String, usize>) -> BTreeMap<String, String> {
        let stale: Vec<String> = self
            .entries
            .iter()
            .filter(|(path, entry)| !entry.pinned() && !usage.contains_key(*path))
            .map(|(path, _)| path.clone())
            .collect();
        for path in stale {
            self.entries.remove(&path);
            self.dirty = true;
        }

        let mut ranked: Vec<(&String, &usize)> = usage.iter().collect();
        ranked.sort_by(|(left_path, left_count), (right_path, right_count)| {
            right_count
                .cmp(left_count)
                .then_with(|| left_path.cmp(right_path))
        });

        let mut ids_by_path = BTreeMap::new();
        for (path, _) in ranked {
            if let Some(entry) = self.entries.get(path) {
                ids_by_path.insert(path.clone(), entry.id.clone());
                continue;
            }
            let id = self.next_generated_id();
            self.entries.insert(
                path.clone(),
                LockEntry {
                    id: id.clone(),
                    path: path.clone(),
                },
            );
            self.dirty = true;
            ids_by_path.insert(path.clone(), id);
        }
        ids_by_path
    }

    pub fn entry_for(&self, path: &str) -> Option<&LockEntry> {
        self.entries.get(path)
    }

    /// `path`/`path*` -> id, including pinned entries that are not used.
    pub fn id_for(&self, path: &str) -> Option<&str> {
        self.entries.get(path).map(|entry| entry.id.as_str())
    }

    fn next_generated_id(&self) -> String {
        let used: BTreeSet<&str> = self
            .entries
            .values()
            .map(|entry| entry.id.as_str())
            .collect();
        let mut candidate = self
            .entries
            .values()
            .filter(|entry| !entry.pinned())
            .filter_map(|entry| id_rank(&entry.id))
            .max()
            .unwrap_or(0)
            + 1;
        loop {
            let id = rank_id(candidate);
            if !used.contains(id.as_str()) {
                return id;
            }
            candidate += 1;
        }
    }
}

fn write_entry(body: &mut String, entry: &LockEntry) -> Result<(), String> {
    if entry.path.contains(',') {
        return Err(format!("path must not contain a comma: {}", entry.path));
    }
    body.push_str(&entry.id);
    body.push(',');
    body.push_str(&entry.path);
    body.push('\n');
    Ok(())
}

/// `<dir>/<app>-application.archol` → `<dir>/<app>-application.lock`.
pub fn lock_path_for_application(application_file: &Path) -> PathBuf {
    application_file.with_extension(LOCK_EXTENSION)
}

pub fn is_safe_id(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// Rank of a generated id in the sequence `a..z, aa..az, a0..a9, ba..`.
fn id_rank(id: &str) -> Option<usize> {
    let mut chars = id.chars();
    let first = chars.next()?;
    if !first.is_ascii_lowercase() {
        return None;
    }
    let first_index = (first as u8 - b'a') as usize;
    let rest: Option<Vec<usize>> = chars
        .map(|ch| {
            if ch.is_ascii_lowercase() {
                Some((ch as u8 - b'a') as usize)
            } else if ch.is_ascii_digit() {
                Some((ch as u8 - b'0') as usize + 26)
            } else {
                None
            }
        })
        .collect();
    let rest = rest?;
    let length = rest.len() + 1;
    if length == 1 {
        return Some(first_index + 1);
    }
    let mut index = first_index;
    for digit in rest {
        index = index * 36 + digit;
    }
    let mut prior = 0usize;
    let mut capacity = 36usize;
    for _ in 2..length {
        prior += 26 * capacity;
        capacity *= 36;
    }
    Some(26 + prior + index + 1)
}

/// Inverse of [`id_rank`]; first character is always a letter.
fn rank_id(mut rank: usize) -> String {
    if rank <= 26 {
        return ((b'a' + (rank.saturating_sub(1)) as u8) as char).to_string();
    }
    rank -= 26;
    let mut length = 2usize;
    let mut capacity = 26 * 36;
    while rank > capacity {
        rank -= capacity;
        length += 1;
        capacity *= 36;
    }
    let mut value = rank - 1;
    let mut suffix = vec![0usize; length - 1];
    for slot in suffix.iter_mut().rev() {
        *slot = value % 36;
        value /= 36;
    }
    const DIGITS: &[u8; 36] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut id = String::with_capacity(length);
    id.push((b'a' + value as u8) as char);
    for digit in suffix {
        id.push(DIGITS[digit] as char);
    }
    id
}

// --- canonical identifier paths -----------------------------------------

pub fn application_path(uri: &str) -> String {
    format!("{uri}#application")
}

pub fn service_path(uri: &str) -> String {
    format!("{uri}#service")
}

pub fn artifact_path(uri: &str, artifact_kind: &str, name: &str) -> String {
    format!("{uri}#{}/{}", artifact_kind.to_ascii_lowercase(), name)
}

/// Canonical path of an auto-created service permission.
pub fn permission_path(uri: &str, name: &str) -> String {
    format!("{uri}#permission/{name}")
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

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("archol-lock-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("demo-application.lock")
    }

    fn used(paths: &[&str]) -> BTreeMap<String, usize> {
        paths.iter().map(|path| (path.to_string(), 1)).collect()
    }

    #[test]
    fn ids_are_assigned_only_to_used_paths_starting_at_a() {
        let mut lock = Lock::new("app");
        let map = lock.assign(&used(&["svc#document/D2", "svc#document/D1"]));
        assert_eq!(map["svc#document/D1"], "a");
        assert_eq!(map["svc#document/D2"], "b");
        assert_eq!(lock.entries.len(), 2);
    }

    #[test]
    fn unused_generated_ids_are_dropped_and_used_ones_kept() {
        let mut lock = Lock::new("app");
        lock.assign(&used(&["svc#document/A", "svc#document/B"]));
        let map = lock.assign(&used(&["svc#document/B"]));
        assert_eq!(map["svc#document/B"], "b");
        assert!(lock.entry_for("svc#document/A").is_none());
        assert_eq!(lock.entries.len(), 1);
    }

    #[test]
    fn csv_roundtrip_is_id_and_path_only() {
        let path = temp_path("csv");
        let mut lock = Lock::new("app");
        lock.assign(&used(&["svc#service", "svc#document/D"]));
        lock.save(&path).unwrap();
        let lines: Vec<String> = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "a,svc#document/D");
        assert_eq!(lines[1], "b,svc#service");
        let loaded = Lock::load(&path, "app").unwrap();
        assert_eq!(loaded.id_for("svc#document/D"), Some("a"));
    }

    #[test]
    fn starred_path_is_pinned_at_top_and_never_pruned() {
        let path = temp_path("pinned");
        std::fs::write(
            &path,
            "checkoutStatus,svc#shape/Status/value/Overdue*\n\
             a,svc#service\n",
        )
        .unwrap();
        let mut lock = Lock::load(&path, "app").unwrap();
        let map = lock.assign(&used(&["svc#service"]));
        assert_eq!(map["svc#service"], "a");
        assert!(lock.entry_for("svc#shape/Status/value/Overdue").is_some());
        lock.save(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "checkoutStatus,svc#shape/Status/value/Overdue*");
        assert!(lines.contains(&"a,svc#service"));
    }

    #[test]
    fn generated_ids_skip_pinned_ids() {
        let path = temp_path("skip");
        std::fs::write(&path, "a,svc#pinned*\n").unwrap();
        let mut lock = Lock::load(&path, "app").unwrap();
        let map = lock.assign(&used(&["svc#document/D"]));
        assert_eq!(map["svc#document/D"], "b");
    }

    #[test]
    fn lock_path_changes_only_the_extension() {
        let path = Path::new("/apps/demo-application.archol");
        assert_eq!(
            lock_path_for_application(path),
            PathBuf::from("/apps/demo-application.lock")
        );
    }

    #[test]
    fn new_ids_follow_reference_count_then_path() {
        let mut lock = Lock::new("app");
        let usage = BTreeMap::from([
            ("svc#document/rare".to_string(), 1),
            ("svc#document/hot".to_string(), 7),
            ("svc#document/warm".to_string(), 3),
        ]);
        let ids = lock.assign(&usage);
        assert_eq!(ids["svc#document/hot"], "a");
        assert_eq!(ids["svc#document/warm"], "b");
        assert_eq!(ids["svc#document/rare"], "c");
        // Existing ids are stable; only new paths consume the next ids.
        let next = BTreeMap::from([
            ("svc#document/hot".to_string(), 7),
            ("svc#document/warm".to_string(), 3),
            ("svc#document/rare".to_string(), 1),
            ("svc#document/new".to_string(), 9),
        ]);
        let ids = lock.assign(&next);
        assert_eq!(ids["svc#document/hot"], "a");
        assert_eq!(ids["svc#document/new"], "d");
    }

    #[test]
    fn legacy_jsonl_lock_is_read_and_rewritten_as_csv() {
        let path = temp_path("legacy");
        std::fs::write(
            &path,
            "{\"id\":\"a\",\"path\":\"svc#service\",\"kind\":\"service\",\"name\":\"svc\",\"manual\":true}\n",
        )
        .unwrap();
        let mut lock = Lock::load(&path, "app").unwrap();
        assert!(lock.dirty);
        assert!(lock.entry_for("svc#service").unwrap().pinned());
        lock.assign(&used(&["svc#service"]));
        lock.save(&path).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "a,svc#service*\n");
    }

    #[test]
    fn rejects_duplicate_ids_on_load() {
        let path = temp_path("duplicate");
        std::fs::write(&path, "a,a#service\na,b#service\n").unwrap();
        let error = Lock::load(&path, "app").unwrap_err();
        assert!(error.contains("duplicate id"), "{error}");
    }
}

#[cfg(test)]
mod base36_tests {
    use super::*;

    #[test]
    fn generated_ids_use_base36_but_start_with_a_letter() {
        let mut lock = Lock::new("app");
        let used: BTreeMap<String, usize> = (0..63)
            .map(|index| (format!("svc#document/D{index:03}"), 1))
            .collect();
        let ids = lock.assign(&used);
        let assigned: Vec<String> = (0..63)
            .map(|index| ids[&format!("svc#document/D{index:03}")].clone())
            .collect();
        assert_eq!(assigned[0], "a");
        assert_eq!(assigned[25], "z");
        assert_eq!(assigned[26], "aa");
        assert_eq!(assigned[51], "az");
        assert_eq!(assigned[52], "a0");
        assert_eq!(assigned[61], "a9");
        assert_eq!(assigned[62], "ba");
        assert!(assigned.iter().all(|id| {
            id.chars().next().unwrap().is_ascii_lowercase()
                && id
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
        }));
        let unique: BTreeSet<&String> = assigned.iter().collect();
        assert_eq!(unique.len(), assigned.len());
    }

    #[test]
    fn rank_encoding_roundtrips_and_keeps_letter_first() {
        for rank in 1..5000 {
            let id = rank_id(rank);
            assert!(id.chars().next().unwrap().is_ascii_lowercase(), "{id}");
            assert_eq!(id_rank(&id), Some(rank), "{id}");
        }
    }
}
