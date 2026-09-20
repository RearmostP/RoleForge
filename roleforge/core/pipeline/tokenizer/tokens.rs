// Input: Source slices and scanner failure locations.
// Output: Internal raw Role data and local tokenizer errors.

pub(super) struct RawRole<'a> {
    pub(super) name: &'a str,
    pub(super) body: &'a str,
    pub(super) declaration_line: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TokenizeError {
    MissingRoleName { line: usize },
    ContentBeforeRole { line: usize },
}
