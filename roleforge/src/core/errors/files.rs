//! File paths identify the affected resource. Original I/O and parser errors
//! are discarded. ErrorKind is retained only to preserve boundary exception
//! compatibility, never rendered as an internal explanation.
use std::{
    fmt, io,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FileError {
    FileNotFound { path: PathBuf },
    FileReadFailed { path: PathBuf, kind: io::ErrorKind },
}

impl FileError {
    pub(crate) fn reading(path: &Path, error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::NotFound => Self::FileNotFound { path: path.into() },
            kind => Self::FileReadFailed {
                path: path.into(),
                kind,
            },
        }
    }
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::FileNotFound { .. } => "FileNotFound",
            Self::FileReadFailed { .. } => "FileReadFailed",
        }
    }
    pub(crate) fn path(&self) -> &Path {
        match self {
            Self::FileNotFound { path } | Self::FileReadFailed { path, .. } => path,
        }
    }
    pub(crate) fn kind(&self) -> io::ErrorKind {
        match self {
            Self::FileNotFound { .. } => io::ErrorKind::NotFound,
            Self::FileReadFailed { kind, .. } => *kind,
        }
    }
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileNotFound { path } => {
                write!(f, "FileNotFound: File not found: {}", path.display())
            }
            Self::FileReadFailed { path, .. } => write!(
                f,
                "FileReadFailed: Failed to read UTF-8 file: {}",
                path.display()
            ),
        }
    }
}
impl std::error::Error for FileError {}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RegistryError {
    File(FileError),
    InvalidRegistry { path: PathBuf },
}
impl RegistryError {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::File(error) => error.name(),
            Self::InvalidRegistry { .. } => "InvalidRegistry",
        }
    }
    pub(crate) fn path(&self) -> &Path {
        match self {
            Self::File(error) => error.path(),
            Self::InvalidRegistry { path } => path,
        }
    }
    pub(crate) fn kind(&self) -> io::ErrorKind {
        match self {
            Self::File(error) => error.kind(),
            Self::InvalidRegistry { .. } => io::ErrorKind::InvalidData,
        }
    }
}
impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(error) => error.fmt(f),
            Self::InvalidRegistry { path } => write!(
                f,
                "InvalidRegistry: Invalid Role registry: {}",
                path.display()
            ),
        }
    }
}
impl std::error::Error for RegistryError {}
