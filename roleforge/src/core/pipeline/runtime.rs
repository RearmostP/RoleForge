// Input: A source path, an already loaded registry, Bridges, and a structured event reporter.
// Output: Ordered Core results after receiving handoff, or a pipeline error.

use std::path::Path;

use super::handoff;
use super::{
    dispatcher::{DispatchResult, dispatch},
    loader::load_file,
    tokenizer::tokenize,
};
use crate::core::{bridges::Bridges, registry::Registry};

pub(crate) use crate::core::errors::RuntimeError;
use crate::core::errors::{CoreEvent, HandoffFailed, OutputWriteFailed};

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
    report: &mut impl FnMut(CoreEvent<'_>) -> Result<(), OutputWriteFailed>,
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
            report(result.event()).map_err(RuntimeError::Output)?;
        }
    }
    if has_conflict {
        report(CoreEvent::HandoffAborted).map_err(RuntimeError::Output)?;
        return Ok(LoadResult {
            roles: results,
            delivered: Vec::new(),
        });
    }

    let mut delivered = Vec::new();
    for result in &results {
        // Unknown is reported and skipped; resolved data remains in source order.
        report(result.event()).map_err(RuntimeError::Output)?;
        if let DispatchResult::Resolved { role, entry } = result {
            let live = handoff::deliver(bridges, entry, role).map_err(|error| {
                RuntimeError::Handoff(HandoffFailed {
                    name: role.name.clone(),
                    index: role.index,
                    role_index: role.role_index,
                    declaration_line: role.source.declaration_line,
                    target: entry.target.clone(),
                    error,
                })
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
