// Input: A source path, an already loaded registry, and temporary development output.
// Output: Ordered dispatch results after temporary handoff, or a pipeline error.

use std::{io, path::Path};

use super::{
    dispatcher::{DispatchResult, dispatch},
    loader::load_file,
    temporary_handoff,
    tokenizer::{TokenizeError, tokenize},
};
use crate::core::registry::Registry;

#[derive(Debug)]
pub(crate) enum RuntimeError {
    Load(io::Error),
    Tokenize(TokenizeError),
    TemporaryHandoff(io::Error),
}

// The caller can reuse a registry across files and pass stdout().lock() for
// development output. This is internal orchestration, not the final public API.
pub(crate) fn run_file(
    path: impl AsRef<Path>,
    registry: &Registry,
    output: &mut impl io::Write,
) -> Result<Vec<DispatchResult>, RuntimeError> {
    let file = load_file(path).map_err(RuntimeError::Load)?;
    let roles = tokenize(&file).map_err(RuntimeError::Tokenize)?;
    let results = dispatch(roles, registry);
    for result in &results {
        if let DispatchResult::Resolved { role, entry } = result {
            temporary_handoff::deliver(role, entry, output)
                .map_err(RuntimeError::TemporaryHandoff)?;
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests;
