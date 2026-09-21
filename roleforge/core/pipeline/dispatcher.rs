// Input: Ordered clean Roles and a loaded Role registry.
// Output: Ordered resolved, unknown, or conflicting Roles, without execution.

use super::models::CleanRole;
use crate::core::registry::{LookupResult, Registry, RoleEntry};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DispatchResult {
    Resolved {
        role: CleanRole,
        entry: RoleEntry,
    },
    Unknown {
        role: CleanRole,
    },
    Conflict {
        role: CleanRole,
        builtin_entry: RoleEntry,
        dynamic_entry: RoleEntry,
    },
}

pub(crate) fn dispatch(roles: Vec<CleanRole>, registry: &Registry) -> Vec<DispatchResult> {
    roles
        .into_iter()
        .map(|role| match registry.get_entry(&role.name) {
            LookupResult::Resolved(entry) => DispatchResult::Resolved {
                role,
                entry: entry.clone(),
            },
            LookupResult::Unknown => DispatchResult::Unknown { role },
            LookupResult::Conflict {
                builtin_entry,
                dynamic_entry,
                ..
            } => {
                let builtin_entry = builtin_entry.clone();
                let dynamic_entry = dynamic_entry.clone();
                DispatchResult::Conflict {
                    role,
                    builtin_entry,
                    dynamic_entry,
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
