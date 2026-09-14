// Input: Loaded RoleForge source content.
// Output: Borrowed raw Role regions, or a local outer-syntax error.

use super::tokens::{RawRole, TokenizeError};

pub(super) fn scan(source: &str) -> Result<Vec<RawRole<'_>>, TokenizeError> {
    let mut roles = Vec::new();
    let mut current: Option<(&str, usize)> = None;
    let mut offset = 0;

    for (line_index, line) in source.split_inclusive('\n').enumerate() {
        let start = offset;
        offset += line.len();
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let declaration = line
            .strip_prefix("@role")
            .filter(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace));
        if let Some(rest) = declaration {
            // Inline comments are Core syntax only on declaration lines.
            let name = rest.split_once('#').map_or(rest, |(name, _)| name).trim();
            if name.is_empty() {
                return Err(TokenizeError::MissingRoleName {
                    line: line_index + 1,
                });
            }
            if let Some((previous_name, body_start)) = current {
                roles.push(RawRole {
                    name: previous_name,
                    body: &source[body_start..start],
                });
            }
            current = Some((name, offset));
        } else if current.is_none() {
            return Err(TokenizeError::ContentBeforeRole {
                line: line_index + 1,
            });
        }
    }

    if let Some((name, body_start)) = current {
        roles.push(RawRole {
            name,
            body: &source[body_start..],
        });
    }
    Ok(roles)
}
