use super::*;
use crate::core::pipeline::models::{CleanRole, SourceInfo};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

struct TestFile(PathBuf);

impl TestFile {
    fn new(content: &[u8]) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        loop {
            let path = std::env::temp_dir().join(format!(
                "roleforge-runtime-{}-{}.rfg",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut file) => {
                    let fixture = Self(path);
                    file.write_all(content).unwrap();
                    return fixture;
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("could not create runtime fixture: {error}"),
            }
        }
    }
}

impl Drop for TestFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn role(name: &str, index: usize, role_index: usize, line: usize, body: &str) -> CleanRole {
    CleanRole {
        name: name.to_owned(),
        index,
        role_index,
        body: body.to_owned(),
        source: SourceInfo {
            declaration_line: line,
        },
    }
}

#[test]
fn project_file_reaches_final_result_with_order_metadata_bodies_and_destinations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("anyone_py_project/test_role.rfg");
    let registry = Registry::from_json(
        r#"{"Directory":{"entry":"Directory/entry"}}"#,
        r#"{"Config":{"entry":"Config/entry"}}"#,
    )
    .unwrap();
    let mut output = Vec::new();
    let results = run_file(&path, &registry, &mut output).unwrap();

    // Accommodate checkout newline conventions while verifying exact body bytes.
    let source = fs::read_to_string(&path).unwrap();
    let newline = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let directory = root.join("roleforge/builtin_roles/Directory/entry");
    let config = root.join("roleforge/roles/Config/entry");
    assert_eq!(
        results,
        vec![
            DispatchResult::Resolved {
                role: role(
                    "Directory",
                    0,
                    0,
                    1,
                    &format!("src/{newline}tests/{newline}{newline}")
                ),
                entry: directory.clone(),
            },
            DispatchResult::Resolved {
                role: role(
                    "Config",
                    1,
                    0,
                    5,
                    &format!("debug = true{newline}{newline}")
                ),
                entry: config,
            },
            DispatchResult::Resolved {
                role: role("Directory", 2, 1, 8, "assets/"),
                entry: directory,
            },
        ]
    );
    let trace = String::from_utf8(output).unwrap();
    let names: Vec<_> = trace
        .lines()
        .filter_map(|line| line.strip_prefix("Role: "))
        .collect();
    assert_eq!(names, ["Directory", "Config", "Directory"]);
    for result in &results {
        let DispatchResult::Resolved { role, entry } = result else {
            unreachable!()
        };
        assert!(trace.contains(&format!(
            "Global Index: {}\nRole Index: {}\nDeclaration Line: {}\nBody:\n{}\nStatus: Resolved\nEntry: {}",
            role.index,
            role.role_index,
            role.source.declaration_line,
            role.body,
            entry.display()
        )));
    }
}

#[test]
fn unknown_and_conflict_remain_visible_and_do_not_reset_indexes() {
    let fixture = TestFile::new(
        b"@role Missing\nunknown\n@role Shared\nconflict\n@role Last\nfirst\n@role Last\nsecond",
    );
    let registry = Registry::from_json(
        r#"{"Shared":{"entry":"shared/builtin"}}"#,
        r#"{"Shared":{"entry":"shared/dynamic"},"Last":{"entry":"last/entry"}}"#,
    )
    .unwrap();
    let mut output = Vec::new();
    let results = run_file(&fixture.0, &registry, &mut output).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("roleforge");
    assert_eq!(
        results,
        vec![
            DispatchResult::Unknown {
                role: role("Missing", 0, 0, 1, "unknown\n")
            },
            DispatchResult::Conflict {
                role: role("Shared", 1, 0, 3, "conflict\n"),
                builtin_entry: root.join("builtin_roles/shared/builtin"),
                dynamic_entry: root.join("roles/shared/dynamic"),
            },
            DispatchResult::Resolved {
                role: role("Last", 2, 0, 5, "first\n"),
                entry: root.join("roles/last/entry"),
            },
            DispatchResult::Resolved {
                role: role("Last", 3, 1, 7, "second"),
                entry: root.join("roles/last/entry"),
            },
        ]
    );
    let trace = String::from_utf8(output).unwrap();
    assert!(trace.contains("Status: Unknown"));
    assert!(trace.contains("Status: Conflict"));
    assert_eq!(trace.matches("[CORE DEBUG]").count(), 4);
}

#[test]
fn stored_registry_can_be_loaded_once_and_reused() {
    let registry = Registry::load().unwrap();
    let fixture = TestFile::new(b"@role UnregisteredStage04Fixture\nbody");
    for _ in 0..2 {
        let mut output = Vec::new();
        assert_eq!(
            run_file(&fixture.0, &registry, &mut output).unwrap(),
            vec![DispatchResult::Unknown {
                role: role("UnregisteredStage04Fixture", 0, 0, 1, "body")
            },]
        );
        assert!(
            String::from_utf8(output)
                .unwrap()
                .contains("Status: Unknown")
        );
    }
}

#[test]
fn empty_and_comment_only_files_produce_no_debug_output() {
    let registry = Registry::from_json("{}", "{}").unwrap();
    for content in [b"".as_slice(), b"# comment\n\n"] {
        let fixture = TestFile::new(content);
        let mut output = Vec::new();
        assert!(
            run_file(&fixture.0, &registry, &mut output)
                .unwrap()
                .is_empty()
        );
        assert!(output.is_empty());
    }
}

#[test]
fn load_failures_preserve_io_errors_without_debug_output() {
    let registry = Registry::from_json("{}", "{}").unwrap();
    let fixture = TestFile::new(&[0xff]);
    let mut output = Vec::new();
    assert!(
        matches!(run_file(&fixture.0, &registry, &mut output), Err(RuntimeError::Load(error)) if error.kind() == io::ErrorKind::InvalidData)
    );
    fs::remove_file(&fixture.0).unwrap();
    assert!(
        matches!(run_file(&fixture.0, &registry, &mut output), Err(RuntimeError::Load(error)) if error.kind() == io::ErrorKind::NotFound)
    );
    assert!(output.is_empty());
}

#[test]
fn tokenizer_failure_stops_before_any_debug_output() {
    let registry = Registry::from_json(r#"{"Valid":{"entry":"valid"}}"#, "{}").unwrap();
    let fixture = TestFile::new(b"@role Valid\nbody\n@role\n");
    let mut output = Vec::new();
    assert!(matches!(
        run_file(&fixture.0, &registry, &mut output),
        Err(RuntimeError::Tokenize(TokenizeError::MissingRoleName {
            line: 3
        }))
    ));
    assert!(output.is_empty());
}

#[test]
fn temporary_output_failure_is_returned() {
    struct BrokenOutput;
    impl Write for BrokenOutput {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let registry = Registry::from_json(r#"{"Valid":{"entry":"valid"}}"#, "{}").unwrap();
    let fixture = TestFile::new(b"@role Valid\nbody");
    assert!(
        matches!(run_file(&fixture.0, &registry, &mut BrokenOutput), Err(RuntimeError::DebugOutput(error)) if error.kind() == io::ErrorKind::BrokenPipe)
    );
}
