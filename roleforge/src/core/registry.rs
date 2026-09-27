// Input: Separate built-in and dynamic Role registry JSON files.
// Output: Structured Bridge entries with resolved targets, unknown names, or structured registration conflicts.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::core::errors::{FileError, RegistryError};
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
    pub(crate) fn load(root: &Path) -> Result<Self, RegistryError> {
        let storage = root.join("core/storage");
        Self::from_json(
            root,
            &read_registry(&storage.join("builtin_roles.json"))?,
            &read_registry(&storage.join("dynamic_roles.json"))?,
        )
    }

    pub(super) fn from_json(
        root: &Path,
        builtin: &str,
        dynamic: &str,
    ) -> Result<Self, RegistryError> {
        Ok(Self {
            builtin: parse_entries(
                builtin,
                &root.join("builtin_roles"),
                &root.join("core/storage/builtin_roles.json"),
            )?,
            dynamic: parse_entries(
                dynamic,
                &root.join("roles"),
                &root.join("core/storage/dynamic_roles.json"),
            )?,
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

fn read_registry(path: &Path) -> Result<String, RegistryError> {
    fs::read_to_string(path).map_err(|error| RegistryError::File(FileError::reading(path, error)))
}

fn parse_entries(
    json: &str,
    base: &Path,
    path: &Path,
) -> Result<HashMap<String, RoleEntry>, RegistryError> {
    let entries: HashMap<String, Entry> = serde_json::from_str(json)
        .map_err(|_| RegistryError::InvalidRegistry { path: path.into() })?;
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
