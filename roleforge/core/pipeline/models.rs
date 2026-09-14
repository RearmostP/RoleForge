// Input: UTF-8 source content and its file path.
// Output: A data-only LoadedFile for the next pipeline stage.

use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct LoadedFile {
    pub(crate) content: String,
    pub(crate) path: PathBuf,
}
