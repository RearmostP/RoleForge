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
    fn deliver(&self, target: &Path, role: RoleInput) -> Result<(), BridgeError>;
}

#[derive(Default)]
pub(crate) struct Bridges {
    entries: HashMap<String, Box<dyn Bridge>>,
}

impl Bridges {
    pub(crate) fn with_builtins() -> Self {
        let mut bridges = Self::default();
        bridges.register("python", python::PythonBridge);
        bridges
    }

    // Registration is internal construction, not a plugin or public install API.
    // Like HashMap::insert, explicitly return any previous registration.
    pub(crate) fn register(
        &mut self,
        identifier: impl Into<String>,
        bridge: impl Bridge + 'static,
    ) -> Option<Box<dyn Bridge>> {
        self.entries.insert(identifier.into(), Box::new(bridge))
    }

    // Exact identifier lookup only. No defaults, extension checks, or inference.
    pub(crate) fn resolve(&self, identifier: &str) -> Option<&dyn Bridge> {
        self.entries.get(identifier).map(Box::as_ref)
    }
}

#[cfg(test)]
mod tests;
