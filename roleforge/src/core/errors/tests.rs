use super::*;
use std::{io, path::Path};

#[test]
fn file_failures_keep_resource_and_exception_kind_but_discard_internal_details() {
    let path = Path::new("source.rfg");
    for kind in [
        io::ErrorKind::NotFound,
        io::ErrorKind::PermissionDenied,
        io::ErrorKind::InvalidData,
    ] {
        let error = FileError::reading(path, io::Error::new(kind, "private OS detail"));
        assert_eq!(error.path(), path);
        assert_eq!(error.kind(), kind);
        assert!(!format!("{error:?} {error}").contains("private"));
        assert!(error.to_string().contains("source.rfg"));
        assert!(std::error::Error::source(&error).is_none());
    }
}

#[test]
fn delivery_cases_have_explicit_messages_without_internal_causes() {
    for (error, name) in [
        (
            BridgeError::PythonTargetLoadFailure,
            "PythonTargetLoadFailure",
        ),
        (BridgeError::MissingReceiver, "MissingReceiver"),
        (BridgeError::ReceiverNotCallable, "ReceiverNotCallable"),
        (BridgeError::ReceiverRaised, "ReceiverRaised"),
        (BridgeError::InputConversion, "InputConversion"),
        (BridgeError::DeliveryCleanupFailed, "DeliveryCleanupFailed"),
    ] {
        assert_eq!(error.name(), name);
        assert!(error.to_string().starts_with(&format!("{name}: ")));
        assert!(std::error::Error::source(&error).is_none());
    }
    let error = HandoffError::UnknownBridge("opaque-id".into());
    assert_eq!(error.name(), "UnknownBridge");
    assert!(error.to_string().contains("opaque-id"));
}

#[test]
fn syntax_messages_preserve_the_existing_contract() {
    let missing = TokenizeError::MissingRoleName { line: 7 };
    assert_eq!(missing.line(), 7);
    assert_eq!(missing.name(), "MissingRoleName");
    assert_eq!(missing.to_string(), "missing Role name");
    let before = TokenizeError::ContentBeforeRole { line: 3 };
    assert_eq!(before.line(), 3);
    assert_eq!(before.name(), "ContentBeforeRole");
    assert_eq!(before.to_string(), "content before first Role");
}
