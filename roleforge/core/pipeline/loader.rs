// Input: Path to a RoleForge .rfg file.
// Output: LoadedFile containing the path and UTF-8 contents, or an I/O error.

use std::{fs, io, path::Path};

use super::models::LoadedFile;

pub(crate) fn load_file(path: impl AsRef<Path>) -> io::Result<LoadedFile> {
    let path = path.as_ref();
    fs::metadata(path)?;
    let content = fs::read_to_string(path)?;

    Ok(LoadedFile {
        content,
        path: path.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Write,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    struct TestFile(PathBuf);

    impl TestFile {
        fn new(content: &[u8]) -> io::Result<Self> {
            static NEXT_ID: AtomicU64 = AtomicU64::new(0);
            loop {
                let path = std::env::temp_dir().join(format!(
                    "roleforge-loader-{}-{}.rfg",
                    std::process::id(),
                    NEXT_ID.fetch_add(1, Ordering::Relaxed)
                ));
                match fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                {
                    Ok(mut file) => {
                        let fixture = Self(path);
                        file.write_all(content)?;
                        return Ok(fixture);
                    }
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(error) => return Err(error),
                }
            }
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    #[test]
    fn loads_rfg_with_exact_utf8_content_and_path() -> io::Result<()> {
        let content = "arbitrary source\r\nשלום 🌍\n";
        let fixture = TestFile::new(content.as_bytes())?;

        let loaded = load_file(&fixture.0)?;

        assert_eq!(loaded.content, content);
        assert_eq!(loaded.path, fixture.0);
        Ok(())
    }

    #[test]
    fn missing_file_returns_not_found() -> io::Result<()> {
        let fixture = TestFile::new(b"")?;
        fs::remove_file(&fixture.0)?;

        let error = load_file(&fixture.0).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        Ok(())
    }

    #[test]
    fn invalid_utf8_returns_invalid_data() -> io::Result<()> {
        let fixture = TestFile::new(&[0xff, 0xfe])?;

        let error = load_file(&fixture.0).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        Ok(())
    }
}
