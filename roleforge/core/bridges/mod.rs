// Input: Explicit Bridge identifiers, opaque targets, and existing Core Role data.
// Output: Explicit Bridge resolution and structured delivery failures.

use std::collections::HashMap;

use crate::core::role_input::RoleInput;
use std::path::Path;

mod python;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BridgeError {
    PythonTargetLoadFailure(String),
    MissingReceiver,
    ReceiverNotCallable,
    ReceiverRaised(String),
    InputConversion(String),
}

pub(crate) trait Bridge {
    type Live;
    fn deliver(&self, target: &Path, role: RoleInput) -> Result<Self::Live, BridgeError>;
}

pub(crate) struct Bridges<T> {
    entries: HashMap<String, Box<dyn Bridge<Live = T>>>,
}

impl<T> Default for Bridges<T> {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

impl Bridges<python::LiveRole> {
    pub(crate) fn with_builtins() -> Self {
        let mut bridges = Self::default();
        bridges.register("python", python::PythonBridge);
        bridges
    }
}

impl<T> Bridges<T> {
    // Registration is internal construction, not a plugin or public install API.
    // Like HashMap::insert, explicitly return any previous registration.
    pub(crate) fn register(
        &mut self,
        identifier: impl Into<String>,
        bridge: impl Bridge<Live = T> + 'static,
    ) -> Option<Box<dyn Bridge<Live = T>>> {
        self.entries.insert(identifier.into(), Box::new(bridge))
    }

    // Exact identifier lookup only. No defaults, extension checks, or inference.
    pub(crate) fn resolve(&self, identifier: &str) -> Option<&dyn Bridge<Live = T>> {
        self.entries.get(identifier).map(Box::as_ref)
    }
}

#[cfg(test)]
mod tests;
