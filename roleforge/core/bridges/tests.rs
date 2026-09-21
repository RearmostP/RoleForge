use super::*;
use crate::core::{
    pipeline::models::{CleanRole, SourceInfo},
    role_input::RoleSource,
};

fn role() -> RoleInput {
    RoleInput {
        name: "ThirdParty".into(),
        index: 7,
        role_index: 2,
        body: "opaque body".into(),
        source: RoleSource {
            declaration_line: 19,
        },
    }
}

#[test]
fn only_python_is_registered_and_unknown_identifiers_never_fall_back() {
    let bridges = Bridges::with_builtins();
    assert!(bridges.resolve("python").is_some());
    for identifier in [
        "rust",
        "hii_im_boby",
        "",
        "Python",
        "python ",
        "role.py",
        "role.rs",
    ] {
        assert!(bridges.resolve(identifier).is_none());
    }
}

struct Probe;
impl Bridge for Probe {
    fn deliver(&self, target: &Path, received: RoleInput) -> Result<(), BridgeError> {
        assert_eq!(target, Path::new("unchanged:target/with.no-convention"));
        assert_eq!(received, role());
        Ok(())
    }
}

#[test]
fn arbitrary_registration_and_replacement_preserve_the_contract() {
    let mut bridges = Bridges::default();
    assert!(bridges.register("hii_im_boby", Probe).is_none());
    bridges
        .resolve("hii_im_boby")
        .unwrap()
        .deliver(Path::new("unchanged:target/with.no-convention"), role())
        .unwrap();
    assert!(bridges.register("python", Probe).is_none());
    let previous = bridges.register("python", Probe).unwrap();
    previous
        .deliver(Path::new("unchanged:target/with.no-convention"), role())
        .unwrap();
}

#[test]
fn handoff_input_preserves_existing_metadata() {
    let discovered = CleanRole {
        name: "ThirdParty".into(),
        index: 7,
        role_index: 2,
        body: "opaque body".into(),
        source: SourceInfo {
            declaration_line: 19,
        },
    };
    assert_eq!(RoleInput::from(&discovered), role());
}
