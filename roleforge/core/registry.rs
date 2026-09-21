// Input: Separate built-in and dynamic Role registry JSON files.
// Output: Structured Bridge entries with resolved targets, unknown names, or structured registration conflicts.

use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Deserialize)]
struct Entry {
    entry: RoleEntry,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct RoleEntry {
    pub(crate) via: String,
    pub(crate) target: PathBuf,
}

pub(crate) struct Registry {
    builtin: HashMap<String, RoleEntry>,
    dynamic: HashMap<String, RoleEntry>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LookupResult<'a> {
    Resolved(&'a RoleEntry),
    Unknown,
    Conflict {
        name: &'a str,
        builtin_entry: &'a RoleEntry,
        dynamic_entry: &'a RoleEntry,
    },
}

impl Registry {
    pub(crate) fn load() -> io::Result<Self> {
        let root = roleforge_root();
        let storage = root.join("core/storage");
        Self::from_json(
            &fs::read_to_string(storage.join("builtin_roles.json"))?,
            &fs::read_to_string(storage.join("dynamic_roles.json"))?,
        )
    }

    pub(super) fn from_json(builtin: &str, dynamic: &str) -> io::Result<Self> {
        let root = roleforge_root();
        Ok(Self {
            builtin: parse_entries(builtin, &root.join("builtin_roles"))?,
            dynamic: parse_entries(dynamic, &root.join("roles"))?,
        })
    }

    pub(crate) fn get_entry(&self, name: &str) -> LookupResult<'_> {
        match (self.builtin.get_key_value(name), self.dynamic.get(name)) {
            (Some((name, builtin_entry)), Some(dynamic_entry)) => LookupResult::Conflict {
                name,
                builtin_entry,
                dynamic_entry,
            },
            (Some((_, entry)), None) | (None, Some(entry)) => LookupResult::Resolved(entry),
            (None, None) => LookupResult::Unknown,
        }
    }
}

fn roleforge_root() -> PathBuf {
    // Stage 03 uses the source-tree location fixed at build time, never the process CWD.
    Path::new(env!("CARGO_MANIFEST_DIR")).join("roleforge")
}

fn parse_entries(json: &str, base: &Path) -> io::Result<HashMap<String, RoleEntry>> {
    let entries: HashMap<String, Entry> = serde_json::from_str(json)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(entries
        .into_iter()
        .map(|(name, entry)| {
            let path = entry.entry.target;
            let resolved = if path.is_absolute() {
                path
            } else {
                base.join(path)
            };
            (
                name,
                RoleEntry {
                    via: entry.entry.via,
                    target: resolved,
                },
            )
        })
        .collect())
}

#[cfg(test)]
mod tests;
