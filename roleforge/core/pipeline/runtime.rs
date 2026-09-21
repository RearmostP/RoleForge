// Input: A source path, an already loaded registry, and temporary development output.
// Output: Ordered Core results after receiving handoff, or a pipeline error.

use std::{io, path::Path};

use super::handoff::{self, HandoffError};
use super::{
    dispatcher::{DispatchResult, dispatch},
    final_core_debug,
    loader::load_file,
    tokenizer::{TokenizeError, tokenize},
};
use crate::core::{bridges::Bridges, registry::Registry};

#[derive(Debug)]
pub(crate) enum RuntimeError {
    Load(io::Error),
    Handoff {
        name: String,
        index: usize,
        target: std::path::PathBuf,
        error: HandoffError,
    },
    Tokenize(TokenizeError),
    DebugOutput(io::Error),
}

// The result is generic over the Bridge's native value. Neutral Core models
// and orchestration do not depend on Python or inspect native Role behavior.
pub(crate) struct LoadResult<T> {
    pub(crate) roles: Vec<DispatchResult>,
    pub(crate) delivered: Vec<(usize, T)>,
}

pub(crate) fn run_file_with_bridges<T>(
    path: impl AsRef<Path>,
    registry: &Registry,
    bridges: &Bridges<T>,
    output: &mut impl io::Write,
) -> Result<LoadResult<T>, RuntimeError> {
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
        return Ok(LoadResult {
            roles: results,
            delivered: Vec::new(),
        });
    }

    let mut delivered = Vec::new();
    for result in &results {
        // Unknown is reported and skipped; resolved data remains in source order.
        final_core_debug::inspect(result, output).map_err(RuntimeError::DebugOutput)?;
        if let DispatchResult::Resolved { role, entry } = result {
            let live =
                handoff::deliver(bridges, entry, role).map_err(|error| RuntimeError::Handoff {
                    name: role.name.clone(),
                    index: role.index,
                    target: entry.target.clone(),
                    error,
                })?;
            delivered.push((role.index, live));
        }
    }
    Ok(LoadResult {
        roles: results,
        delivered,
    })
}

#[cfg(test)]
mod tests;
