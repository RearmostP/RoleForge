use std::fmt;

/// A one-based source line is part of an outer-syntax failure.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TokenizeError {
    MissingRoleName { line: usize },
    ContentBeforeRole { line: usize },
}
impl TokenizeError {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::MissingRoleName { .. } => "MissingRoleName",
            Self::ContentBeforeRole { .. } => "ContentBeforeRole",
        }
    }
    pub(crate) fn line(&self) -> usize {
        match self {
            Self::MissingRoleName { line } | Self::ContentBeforeRole { line } => *line,
        }
    }
}
impl fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::MissingRoleName { .. } => "missing Role name",
            Self::ContentBeforeRole { .. } => "content before first Role",
        })
    }
}
impl std::error::Error for TokenizeError {}
