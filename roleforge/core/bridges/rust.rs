use super::{Bridge, BridgeError};
use crate::core::pipeline::models::CleanRole;

pub(super) struct RustBridge;

impl Bridge for RustBridge {
    fn deliver(&self, _target: &str, _role: &CleanRole) -> Result<(), BridgeError> {
        // No library loading, exported symbol, or ABI has been approved yet.
        Err(BridgeError::DeliveryUnavailable { bridge: "rust" })
    }
}
