// Dedicated presentation adapter; no detection, severity, or continuation policy.
use std::io::{self, Write};

use crate::core::errors::{CoreEvent, OutputWriteFailed};

/// Preserve the existing stdout text and write timing. Runtime owns policy.
pub(crate) fn report(
    event: CoreEvent<'_>,
    output: &mut impl Write,
) -> Result<(), OutputWriteFailed> {
    render(event, output).map_err(OutputWriteFailed::from)
}

fn render(event: CoreEvent<'_>, output: &mut impl Write) -> io::Result<()> {
    match event {
        CoreEvent::ResolvedRole { role, entry } => writeln!(
            output,
            "[CORE DEBUG]\nRole: {}\nGlobal Index: {}\nRole Index: {}\nDeclaration Line: {}\nBody:\n{}\nStatus: Resolved\nEntry: {}",
            role.name,
            role.index,
            role.role_index,
            role.source.declaration_line,
            role.body,
            entry.target.display()
        ),
        CoreEvent::UnknownRole { role } => {
            writeln!(output, "[RoleForge] Unknown Role: {}", role.name)
        }
        CoreEvent::RoleConflict {
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
        CoreEvent::HandoffAborted => writeln!(output, "[RoleForge] Handoff aborted."),
    }
}
