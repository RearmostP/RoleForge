use super::{Bridge, BridgeError};
use crate::core::pipeline::models::CleanRole;

pub(super) struct PythonBridge;

impl Bridge for PythonBridge {
    fn deliver(&self, _target: &str, _role: &CleanRole) -> Result<(), BridgeError> {
        // The public Python API already uses PyO3. Do not load a Python Role or
        // invent a callable until the target and receiving contract are defined.
        Err(BridgeError::DeliveryUnavailable { bridge: "python" })
    }
}
