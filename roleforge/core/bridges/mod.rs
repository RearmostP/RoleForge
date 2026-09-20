// Input: Explicit Bridge identifiers, opaque targets, and existing Core Role data.
// Output: A selected Bridge, or an explicit unavailable-delivery result in Stage 07.

use std::collections::HashMap;

use crate::core::pipeline::models::CleanRole;

mod python;
mod rust;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BridgeError {
    // Identifies the implementation, not a language inferred from the target.
    DeliveryUnavailable { bridge: &'static str },
}

pub(crate) trait Bridge {
    // target is opaque here. Its interpretation and the receiving protocol await
    // Stage 08; this signature does not define a Registry entry, path, or ABI.
    // Borrow the original neutral data without rebuilding metadata or indexes.
    fn deliver(&self, target: &str, role: &CleanRole) -> Result<(), BridgeError>;
}

#[derive(Default)]
pub(crate) struct Bridges {
    entries: HashMap<String, Box<dyn Bridge>>,
}

impl Bridges {
    pub(crate) fn with_builtins() -> Self {
        let mut bridges = Self::default();
        bridges.register("python", python::PythonBridge);
        bridges.register("rust", rust::RustBridge);
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
