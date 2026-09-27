use std::{fmt, path::PathBuf};

/// Receipt failures describe the common delivery contract, never Role internals.
/// Their affected destination and Role identity are supplied by HandoffFailed.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BridgeError {
    PythonTargetLoadFailure,
    MissingReceiver,
    ReceiverNotCallable,
    ReceiverRaised,
    InputConversion,
    DeliveryCleanupFailed,
}
impl BridgeError {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::PythonTargetLoadFailure => "PythonTargetLoadFailure",
            Self::MissingReceiver => "MissingReceiver",
            Self::ReceiverNotCallable => "ReceiverNotCallable",
            Self::ReceiverRaised => "ReceiverRaised",
            Self::InputConversion => "InputConversion",
            Self::DeliveryCleanupFailed => "DeliveryCleanupFailed",
        }
    }
}
impl fmt::Display for BridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::PythonTargetLoadFailure => "Failed to load the Python Role target.",
            Self::MissingReceiver => "The Role target must define roleforge_receive(role).",
            Self::ReceiverNotCallable => "The Role target's roleforge_receive must be callable.",
            Self::ReceiverRaised => "The Role receiver failed to complete receipt.",
            Self::InputConversion => "Failed to create the live Role from its input.",
            Self::DeliveryCleanupFailed => {
                "Failed to restore the delivery environment after receipt."
            }
        };
        write!(f, "{}: {message}", self.name())
    }
}
impl std::error::Error for BridgeError {}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum HandoffError {
    UnknownBridge(String),
    Delivery(BridgeError),
}
impl HandoffError {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::UnknownBridge(_) => "UnknownBridge",
            Self::Delivery(error) => error.name(),
        }
    }
}
impl fmt::Display for HandoffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownBridge(identifier) => {
                write!(f, "UnknownBridge: No Bridge registered as {identifier:?}.")
            }
            Self::Delivery(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for HandoffError {}

/// Identity of the delivery that failed. No body, traceback, or internal cause.
#[derive(Debug)]
pub(crate) struct HandoffFailed {
    pub(crate) name: String,
    pub(crate) index: usize,
    pub(crate) role_index: usize,
    pub(crate) declaration_line: usize,
    pub(crate) target: PathBuf,
    pub(crate) error: HandoffError,
}
impl fmt::Display for HandoffFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Role {} (index {}), {}: {}",
            self.name,
            self.index,
            self.target.display(),
            self.error
        )
    }
}
impl std::error::Error for HandoffFailed {}
