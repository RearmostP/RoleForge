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

pub(super) fn deliver<T>(
    bridges: &Bridges<T>,
    entry: &RoleEntry,
    role: &CleanRole,
) -> Result<T, HandoffError> {
    let bridge = bridges
        .resolve(&entry.via)
        .ok_or_else(|| HandoffError::UnknownBridge(entry.via.clone()))?;
    bridge
        .deliver(&entry.target, RoleInput::from(role))
        .map_err(HandoffError::Delivery)
}
