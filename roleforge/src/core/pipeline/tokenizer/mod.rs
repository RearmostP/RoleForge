// Input: Tokenizer processing modules.
// Output: The crate-internal tokenize entry point and its local error type.

mod role_content;
mod role_scanner;
mod tokenizer;
mod tokens;

pub(crate) use tokenizer::tokenize;
pub(crate) use tokens::TokenizeError;
