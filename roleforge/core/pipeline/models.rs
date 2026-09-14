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
    pub(crate) index: usize,
    pub(crate) name: String,
    pub(crate) body: String,
}
