use super::{FileError, HandoffFailed, TokenizeError};
use std::{fmt, io};

/// Kind preserves the existing Python I/O exception class; no original I/O
/// error, OS message, or cause chain is retained.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct OutputWriteFailed {
    pub(crate) kind: io::ErrorKind,
}
impl From<io::Error> for OutputWriteFailed {
    fn from(error: io::Error) -> Self {
        Self { kind: error.kind() }
    }
}
impl fmt::Display for OutputWriteFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OutputWriteFailed: Failed to write Core output.")
    }
}
impl std::error::Error for OutputWriteFailed {}

#[derive(Debug)]
pub(crate) enum RuntimeError {
    Load(FileError),
    Handoff(HandoffFailed),
    Tokenize(TokenizeError),
    Output(OutputWriteFailed),
}

/// Discovery/delivery succeeded, but Core could not build the public Project.
/// The Python boundary adds the affected load request, not an internal cause.
#[derive(Debug)]
pub(crate) struct ProjectConstructionFailed;
impl fmt::Display for ProjectConstructionFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProjectConstructionFailed: Failed to construct the loaded Project.")
    }
}
impl std::error::Error for ProjectConstructionFailed {}
