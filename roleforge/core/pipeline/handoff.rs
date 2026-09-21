use super::models::CleanRole;
use crate::core::{
    bridges::{BridgeError, Bridges},
    registry::RoleEntry,
    role_input::RoleInput,
};

#[derive(Debug)]
pub(crate) enum HandoffError {
    UnknownBridge(String),
    Delivery(BridgeError),
}

pub(super) fn deliver(
    bridges: &Bridges,
    entry: &RoleEntry,
    role: &CleanRole,
) -> Result<(), HandoffError> {
    let bridge = bridges
        .resolve(&entry.via)
        .ok_or_else(|| HandoffError::UnknownBridge(entry.via.clone()))?;
    bridge
        .deliver(&entry.target, RoleInput::from(role))
        .map_err(HandoffError::Delivery)
}
