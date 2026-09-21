// Input: File loading, tokenization, and neutral data model modules.
// Output: Internal pipeline components.

pub(crate) mod dispatcher;
mod final_core_debug;
pub(crate) mod loader;
pub(crate) mod models;
pub(crate) mod runtime;
pub(crate) mod tokenizer;

pub(crate) mod handoff;
