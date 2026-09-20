// Temporary Stage 05 inspection of Core results, not Role delivery or execution.
use std::io::{self, Write};

use super::dispatcher::DispatchResult;

pub(super) fn inspect(result: &DispatchResult, output: &mut impl Write) -> io::Result<()> {
    let role = match result {
        DispatchResult::Resolved { role, .. }
        | DispatchResult::Unknown { role }
        | DispatchResult::Conflict { role, .. } => role,
    };
    writeln!(
        output,
        "[CORE DEBUG]\nRole: {}\nGlobal Index: {}\nRole Index: {}\nDeclaration Line: {}\nBody:\n{}",
        role.name, role.index, role.role_index, role.source.declaration_line, role.body
    )?;
    match result {
        DispatchResult::Resolved { entry, .. } => {
            writeln!(output, "Status: Resolved\nEntry: {}", entry.display())
        }
        DispatchResult::Unknown { .. } => writeln!(output, "Status: Unknown"),
        DispatchResult::Conflict {
            builtin_entry,
            dynamic_entry,
            ..
        } => writeln!(
            output,
            "Status: Conflict\nBuilt-in Entry: {}\nDynamic Entry: {}",
            builtin_entry.display(),
            dynamic_entry.display()
        ),
    }
}
