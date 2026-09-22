// Temporary Core inspection and Stage 06 reports, not Role delivery or execution.
// TODO: Replace direct Unknown/Conflict output with the structured error/event
// system when defined. This is not the future Console or Error Manager.
use std::io::{self, Write};

use super::dispatcher::DispatchResult;

pub(super) fn inspect(result: &DispatchResult, output: &mut impl Write) -> io::Result<()> {
    match result {
        DispatchResult::Resolved { role, entry } => writeln!(
            output,
            "[CORE DEBUG]\nRole: {}\nGlobal Index: {}\nRole Index: {}\nDeclaration Line: {}\nBody:\n{}\nStatus: Resolved\nEntry: {}",
            role.name,
            role.index,
            role.role_index,
            role.source.declaration_line,
            role.body,
            entry.target.display()
        ),
        DispatchResult::Unknown { role } => {
            writeln!(output, "[RoleForge] Unknown Role: {}", role.name)
        }
        DispatchResult::Conflict {
            role,
            builtin_entry,
            dynamic_entry,
        } => writeln!(
            output,
            "[RoleForge] Role conflict: {}\nBuiltin entry: {}\nDynamic entry: {}",
            role.name,
            builtin_entry.target.display(),
            dynamic_entry.target.display()
        ),
    }
}

pub(super) fn handoff_aborted(output: &mut impl Write) -> io::Result<()> {
    writeln!(output, "[RoleForge] Handoff aborted.")
}
