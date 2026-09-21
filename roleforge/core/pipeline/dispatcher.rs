// Input: Ordered clean Roles and a loaded Role registry.
// Output: Ordered resolved, unknown, or conflicting Roles, without execution.

#[cfg(test)]
use std::path::PathBuf;

use super::models::CleanRole;
use crate::core::registry::{LookupResult, Registry, RoleEntry};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DispatchResult {
    Resolved {
        role: CleanRole,
        entry: RoleEntry,
    },
    Unknown {
        role: CleanRole,
    },
    Conflict {
        role: CleanRole,
        builtin_entry: RoleEntry,
        dynamic_entry: RoleEntry,
    },
}

pub(crate) fn dispatch(roles: Vec<CleanRole>, registry: &Registry) -> Vec<DispatchResult> {
    roles
        .into_iter()
        .map(|role| match registry.get_entry(&role.name) {
            LookupResult::Resolved(entry) => DispatchResult::Resolved {
                role,
                entry: entry.clone(),
            },
            LookupResult::Unknown => DispatchResult::Unknown { role },
            LookupResult::Conflict {
                builtin_entry,
                dynamic_entry,
                ..
            } => {
                let builtin_entry = builtin_entry.clone();
                let dynamic_entry = dynamic_entry.clone();
                DispatchResult::Conflict {
                    role,
                    builtin_entry,
                    dynamic_entry,
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::pipeline::{models::LoadedFile, tokenizer::tokenize};

    #[test]
    fn routes_in_source_order_and_continues_after_unknown_with_context() {
        let registry = Registry::from_json(
            r#"{"Zebra":{"entry":{"via":"python","target":"Zebra/entry"}}}"#,
            r#"{"Alpha":{"entry":{"via":"python","target":"Alpha/entry"}}}"#,
        )
        .unwrap();
        let file = LoadedFile {
            path: PathBuf::from("not-read.rfg"),
            content: "# heading\n\n@role Zebra\nopaque # content\n@role Missing\nunknown body\n\n@role Alpha\nlast body".to_owned(),
        };
        let roles = tokenize(&file).unwrap();
        let results = dispatch(roles, &registry);
        let mut expected_roles = tokenize(&file).unwrap().into_iter();
        assert_eq!(
            results,
            vec![
                DispatchResult::Resolved {
                    role: expected_roles.next().unwrap(),
                    entry: RoleEntry {
                        via: "python".into(),
                        target: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                            .join("roleforge/builtin_roles/Zebra/entry")
                    },
                },
                DispatchResult::Unknown {
                    role: expected_roles.next().unwrap()
                },
                DispatchResult::Resolved {
                    role: expected_roles.next().unwrap(),
                    entry: RoleEntry {
                        via: "python".into(),
                        target: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                            .join("roleforge/roles/Alpha/entry")
                    },
                },
            ]
        );
        let DispatchResult::Unknown { role } = &results[1] else {
            panic!("expected unknown Role")
        };
        assert_eq!(role.name, "Missing");
        assert_eq!(role.index, 1);
        assert_eq!(role.source.declaration_line, 5);
    }

    #[test]
    fn conflict_preserves_role_and_both_entries_and_continues_in_order() {
        let registry = Registry::from_json(
            r#"{"First":{"entry":{"via":"python","target":"First/entry"}},"Directory":{"entry":{"via":"python","target":"Directory/entry"}}}"#,
            r#"{"Directory":{"entry":{"via":"python","target":"MyDirectory/entry"}},"Last":{"entry":{"via":"python","target":"Last/entry"}}}"#,
        )
        .unwrap();
        let file = LoadedFile {
            path: PathBuf::from("not-read.rfg"),
            content:
                "# heading\n@role First\nfirst\n\n@role Directory\nopaque body\n@role Last\nlast"
                    .to_owned(),
        };
        let roles = tokenize(&file).unwrap();
        let results = dispatch(roles, &registry);
        let mut expected = tokenize(&file).unwrap().into_iter();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("roleforge");
        assert_eq!(
            results,
            vec![
                DispatchResult::Resolved {
                    role: expected.next().unwrap(),
                    entry: RoleEntry {
                        via: "python".into(),
                        target: root.join("builtin_roles/First/entry")
                    },
                },
                DispatchResult::Conflict {
                    role: expected.next().unwrap(),
                    builtin_entry: RoleEntry {
                        via: "python".into(),
                        target: root.join("builtin_roles/Directory/entry")
                    },
                    dynamic_entry: RoleEntry {
                        via: "python".into(),
                        target: root.join("roles/MyDirectory/entry")
                    },
                },
                DispatchResult::Resolved {
                    role: expected.next().unwrap(),
                    entry: RoleEntry {
                        via: "python".into(),
                        target: root.join("roles/Last/entry")
                    },
                },
            ]
        );
        let DispatchResult::Conflict { role, .. } = &results[1] else {
            panic!("expected conflicting Role")
        };
        assert_eq!(role.name, "Directory");
        assert_eq!(role.index, 1);
        assert_eq!(role.source.declaration_line, 5);
        assert_eq!(role.body, "opaque body\n");
    }

    #[test]
    fn empty_input_has_no_results() {
        let registry = Registry::from_json("{}", "{}").unwrap();
        assert!(dispatch(Vec::new(), &registry).is_empty());
    }
}
