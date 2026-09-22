// Input: Source content, paths, and discovered Role data.
// Output: Neutral LoadedFile and CleanRole pipeline models.

use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct LoadedFile {
    pub(crate) content: String,
    pub(crate) path: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CleanRole {
    // Zero-based position among all discovered Roles.
    pub(crate) index: usize,
    // Zero-based position among Roles with the same name.
    pub(crate) role_index: usize,
    pub(crate) name: String,
    pub(crate) body: String,
    pub(crate) source: SourceInfo,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SourceInfo {
    pub(crate) declaration_line: usize,
}
