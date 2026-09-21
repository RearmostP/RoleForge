// Input: Path to a RoleForge .rfg file.
// Output: LoadedFile containing the path and UTF-8 contents, or an I/O error.

use std::{fs, io, path::Path};

use super::models::LoadedFile;

pub(crate) fn load_file(path: impl AsRef<Path>) -> io::Result<LoadedFile> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)?;

    Ok(LoadedFile {
        content,
        path: path.to_path_buf(),
    })
}

#[cfg(test)]
mod tests;
