// Input: A source path, an already loaded registry, and temporary development output.
// Output: Ordered final Core results before real handoff, or a pipeline error.

use std::{io, path::Path};

use super::{
    dispatcher::{DispatchResult, dispatch},
    final_core_debug,
    loader::load_file,
    tokenizer::{TokenizeError, tokenize},
};
use crate::core::registry::Registry;

#[derive(Debug)]
pub(crate) enum RuntimeError {
    Load(io::Error),
    Tokenize(TokenizeError),
    DebugOutput(io::Error),
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

    // Stage 06: inspect the COMPLETE result set before processing any resolved
    // Role. Report all conflicts and abort the whole delivery phase, preserving
    // the original results for the Python caller. Never choose a registry winner.
    let mut has_conflict = false;
    for result in &results {
        if matches!(result, DispatchResult::Conflict { .. }) {
            has_conflict = true;
            final_core_debug::inspect(result, output).map_err(RuntimeError::DebugOutput)?;
        }
    }
    if has_conflict {
        final_core_debug::handoff_aborted(output).map_err(RuntimeError::DebugOutput)?;
        return Ok(results);
    }

    for result in &results {
        // Unknown is reported and skipped; resolved data remains in source order.
        // TODO: Deliver resolved CleanRole + entry once the receiving protocol is
        // defined. Debug inspection is NOT physical handoff or Role execution.
        final_core_debug::inspect(result, output).map_err(RuntimeError::DebugOutput)?;
    }
    Ok(results)
}

#[cfg(test)]
mod tests;
