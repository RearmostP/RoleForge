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
    let json =
        serde_json::json!({"External": {"entry": {"via": "python", "target": path}}}).to_string();
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
