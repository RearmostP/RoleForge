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
mod tests {
    use super::*;

    #[test]
    fn relative_entries_use_separate_absolute_default_directories() {
        let registry = Registry::from_json(
            r#"{"Builtin":{"entry":{"via":"python","target":"Builtin/entry"}}}"#,
            r#"{"Dynamic":{"entry":{"via":"python","target":"Dynamic/entry"}}}"#,
        )
        .unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("roleforge");
        assert!(root.is_absolute());
        assert_eq!(
            registry.get_entry("Builtin"),
            LookupResult::Resolved(&RoleEntry {
                via: "python".into(),
                target: root.join("builtin_roles/Builtin/entry")
            })
        );
        assert_eq!(
            registry.get_entry("Dynamic"),
            LookupResult::Resolved(&RoleEntry {
                via: "python".into(),
                target: root.join("roles/Dynamic/entry")
            })
        );
        assert_eq!(registry.get_entry("Missing"), LookupResult::Unknown);
    }

    #[test]
    fn absolute_entries_are_used_unchanged_in_both_registries() {
        let path = std::env::temp_dir().join("external-role/entry");
        assert!(path.is_absolute());
        let json = serde_json::json!({"External": {"entry": {"via": "python", "target": path}}})
            .to_string();
        for registry in [
            Registry::from_json(&json, "{}"),
            Registry::from_json("{}", &json),
        ] {
            assert_eq!(
                registry.unwrap().get_entry("External"),
                LookupResult::Resolved(&RoleEntry {
                    via: "python".into(),
                    target: path.clone()
                })
            );
        }
    }

    #[test]
    fn cross_registry_name_collision_preserves_name_and_both_resolved_entries() {
        let registry = Registry::from_json(
            r#"{"Same":{"entry":{"via":"python","target":"builtin/entry"}}}"#,
            r#"{"Same":{"entry":{"via":"python","target":"dynamic/entry"}}}"#,
        )
        .unwrap();
        assert_eq!(
            registry.get_entry("Same"),
            LookupResult::Conflict {
                name: "Same",
                builtin_entry: &RoleEntry {
                    via: "python".into(),
                    target: roleforge_root().join("builtin_roles/builtin/entry")
                },
                dynamic_entry: &RoleEntry {
                    via: "python".into(),
                    target: roleforge_root().join("roles/dynamic/entry")
                },
            }
        );
    }

    #[test]
    fn malformed_metadata_returns_invalid_data() {
        for json in [
            "{",
            r#"{"Role":{}}"#,
            r#"{"Role":{"entry":42}}"#,
            r#"{"Role":{"entry":"legacy/path"}}"#,
            r#"{"Role":{"entry":{"target":"main.py"}}}"#,
            r#"{"Role":{"entry":{"via":"python"}}}"#,
            r#"{"Role":{"entry":{"via":123,"target":"main.py"}}}"#,
        ] {
            for result in [
                Registry::from_json(json, "{}"),
                Registry::from_json("{}", json),
            ] {
                assert!(matches!(result, Err(error) if error.kind() == io::ErrorKind::InvalidData));
            }
        }
    }

    #[test]
    fn loads_separate_storage_files() {
        Registry::load().unwrap();
    }
}
