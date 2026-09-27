//! Routing observations have no assigned severity. Borrowed discovery data
//! retains both indexes, source location, and both entries for conflicts.
use crate::core::{pipeline::models::CleanRole, registry::RoleEntry};

#[derive(Debug)]
pub(crate) enum CoreEvent<'a> {
    ResolvedRole {
        role: &'a CleanRole,
        entry: &'a RoleEntry,
    },
    UnknownRole {
        role: &'a CleanRole,
    },
    RoleConflict {
        role: &'a CleanRole,
        builtin_entry: &'a RoleEntry,
        dynamic_entry: &'a RoleEntry,
    },
    HandoffAborted,
}
