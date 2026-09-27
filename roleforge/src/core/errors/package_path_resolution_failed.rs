// Input: Failure to discover the RoleForge package directory.
// Output: One Core error identity and its focused explanation, without internal causes.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PackagePathResolutionFailed;

impl PackagePathResolutionFailed {
    pub(crate) const NAME: &'static str = "PackagePathResolutionFailed";
    pub(crate) const CATEGORY: &'static str = "RoleForge Error";

    pub(crate) const fn message(self) -> &'static str {
        "Failed to resolve the RoleForge package path."
    }
}
