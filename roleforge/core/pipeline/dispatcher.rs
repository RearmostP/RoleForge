// Input: Ordered clean Roles and a loaded Role registry.
// Output: Ordered resolved destinations or structured unknown Roles, without execution.

use std::path::PathBuf;

use super::models::CleanRole;
use crate::core::registry::Registry;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DispatchResult {
    Resolved { role: CleanRole, entry: PathBuf },
    Unknown { role: CleanRole },
}

pub(crate) fn dispatch(roles: Vec<CleanRole>, registry: &Registry) -> Vec<DispatchResult> {
    roles
        .into_iter()
        .map(|role| match registry.get_entry(&role.name) {
            Some(entry) => DispatchResult::Resolved {
                role,
                entry: entry.to_path_buf(),
            },
            None => DispatchResult::Unknown { role },
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
            r#"{"Zebra":{"entry":"Zebra/entry"}}"#,
            r#"{"Alpha":{"entry":"Alpha/entry"}}"#,
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
                    entry: registry.get_entry("Zebra").unwrap().to_path_buf(),
                },
                DispatchResult::Unknown {
                    role: expected_roles.next().unwrap()
                },
                DispatchResult::Resolved {
                    role: expected_roles.next().unwrap(),
                    entry: registry.get_entry("Alpha").unwrap().to_path_buf(),
                },
            ]
        );
        let DispatchResult::Unknown { role } = &results[1] else {
            panic!("expected unknown Role")
        };
        assert_eq!(role.name, "Missing");
        assert_eq!(role.index, 2);
        assert_eq!(role.source.declaration_line, 5);
    }

    #[test]
    fn empty_input_has_no_results() {
        let registry = Registry::from_json("{}", "{}").unwrap();
        assert!(dispatch(Vec::new(), &registry).is_empty());
    }
}
