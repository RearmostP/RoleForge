// Input: Core-owned error definitions.
// Output: The internal access boundary for Core errors.

mod delivery;
mod events;
mod files;
mod package_path_resolution_failed;
mod runtime;
mod syntax;

pub(crate) use delivery::{BridgeError, HandoffError, HandoffFailed};
pub(crate) use events::CoreEvent;
pub(crate) use files::{FileError, RegistryError};
pub(crate) use runtime::{OutputWriteFailed, ProjectConstructionFailed, RuntimeError};
pub(crate) use syntax::TokenizeError;

#[cfg(test)]
mod tests;

pub(crate) use package_path_resolution_failed::PackagePathResolutionFailed;
