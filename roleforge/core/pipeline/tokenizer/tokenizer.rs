// Input: A LoadedFile, already read into memory.
// Output: Ordered clean Roles, or a local tokenizer error.

use std::collections::HashMap;

use super::{TokenizeError, role_content, role_scanner};
use crate::core::pipeline::models::{CleanRole, LoadedFile, SourceInfo};

pub(crate) fn tokenize(file: &LoadedFile) -> Result<Vec<CleanRole>, TokenizeError> {
    let raw_roles = role_scanner::scan(&file.content)?;
    let mut role_counts = HashMap::new();
    Ok(raw_roles
        .into_iter()
        .enumerate()
        .map(|(position, raw)| {
            let count = role_counts.entry(raw.name).or_insert(0);
            let role_index = *count;
            *count += 1;
            CleanRole {
                index: position,
                role_index,
                name: raw.name.to_owned(),
                body: role_content::clean_body(raw.body),
                source: SourceInfo {
                    declaration_line: raw.declaration_line,
                },
            }
        })
        .collect())
}

#[cfg(test)]
mod tests;
