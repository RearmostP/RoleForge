use super::*;
use crate::core::pipeline::models::SourceInfo;

fn role() -> CleanRole {
    CleanRole {
        name: "ThirdParty_Example".to_owned(),
        index: 7,
        role_index: 2,
        body: "opaque # body\r\nשלום\n".to_owned(),
        source: SourceInfo {
            declaration_line: 19,
        },
    }
}

#[test]
fn builtins_resolve_to_distinct_implementations_through_the_same_contract() {
    let bridges = Bridges::with_builtins();
    let role = role();
    for identifier in ["python", "rust"] {
        let bridge: &dyn Bridge = bridges.resolve(identifier).unwrap();
        assert_eq!(
            bridge.deliver("opaque-target", &role),
            Err(BridgeError::DeliveryUnavailable { bridge: identifier })
        );
    }
}

#[test]
fn unknown_identifiers_never_fall_back_or_infer_a_bridge() {
    let bridges = Bridges::with_builtins();
    for identifier in ["hii_im_boby", "", "Python", "python ", "role.py", "role.rs"] {
        assert!(bridges.resolve(identifier).is_none(), "{identifier}");
    }
}

#[test]
fn arbitrary_identifiers_select_the_registered_implementation() {
    let mut bridges = Bridges::default();
    assert!(
        bridges
            .register("hii_im_boby", python::PythonBridge)
            .is_none()
    );
    // Even a familiar identifier has no intrinsic meaning to the resolver.
    assert!(bridges.register("python", rust::RustBridge).is_none());
    assert_eq!(
        bridges
            .resolve("hii_im_boby")
            .unwrap()
            .deliver("file.rs", &role()),
        Err(BridgeError::DeliveryUnavailable { bridge: "python" })
    );
    assert_eq!(
        bridges
            .resolve("python")
            .unwrap()
            .deliver("file.py", &role()),
        Err(BridgeError::DeliveryUnavailable { bridge: "rust" })
    );
    assert!(bridges.resolve("rust").is_none());
}

#[test]
fn builtins_never_claim_delivery_or_change_role_data() {
    let bridges = Bridges::with_builtins();
    let original = role();
    for identifier in ["python", "rust"] {
        for target in ["", "missing.py", "missing.dll", "arbitrary:target"] {
            assert!(matches!(
                bridges
                    .resolve(identifier)
                    .unwrap()
                    .deliver(target, &original),
                Err(BridgeError::DeliveryUnavailable { .. })
            ));
            assert_eq!(original, role());
        }
    }
}

#[test]
fn contract_passes_original_metadata_and_opaque_target_to_an_arbitrary_bridge() {
    // A contract probe only; it does not model an executed Role or real delivery.
    struct Probe;
    impl Bridge for Probe {
        fn deliver(&self, target: &str, received: &CleanRole) -> Result<(), BridgeError> {
            assert_eq!(target, "unchanged:target/with.no-convention");
            assert_eq!(received, &role());
            Err(BridgeError::DeliveryUnavailable { bridge: "probe" })
        }
    }
    let mut bridges = Bridges::default();
    bridges.register("opaque-id", Probe);
    assert_eq!(
        bridges
            .resolve("opaque-id")
            .unwrap()
            .deliver("unchanged:target/with.no-convention", &role()),
        Err(BridgeError::DeliveryUnavailable { bridge: "probe" })
    );
}

#[test]
fn replacing_a_registration_returns_the_previous_bridge() {
    let mut bridges = Bridges::with_builtins();
    let previous = bridges.register("python", rust::RustBridge).unwrap();
    assert_eq!(
        previous.deliver("target", &role()),
        Err(BridgeError::DeliveryUnavailable { bridge: "python" })
    );
    assert_eq!(
        bridges
            .resolve("python")
            .unwrap()
            .deliver("target", &role()),
        Err(BridgeError::DeliveryUnavailable { bridge: "rust" })
    );
}
