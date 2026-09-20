// Input: A resolved Role and destination, plus a development output stream.
// Output: A temporary handoff trace; no Role implementation is loaded or invoked.

use std::{io, path::Path};

use super::models::CleanRole;

// Stage 04 development infrastructure only, not the future console or Role ABI.
pub(super) fn deliver(
    role: &CleanRole,
    entry: &Path,
    output: &mut impl io::Write,
) -> io::Result<()> {
    writeln!(
        output,
        "[HANDOFF]\nRole: {}\nGlobal Index: {}\nRole Index: {}\nEntry: {}",
        role.name,
        role.index,
        role.role_index,
        entry.display()
    )
}
