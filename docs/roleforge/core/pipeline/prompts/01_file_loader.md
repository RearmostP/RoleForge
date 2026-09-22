# Prompt 01 — RFG File Loader

Before making any changes:

1. Read the Iron Rules in `../../../..`.
2. Read the current temporary decisions document in `../../../..` if present.
3. Inspect the existing project structure before choosing file/module locations.
4. Do not implement anything outside the scope of this prompt.
5. The RoleForge Core is being implemented in **Rust**. Do not implement this task in Python.

## Goal

Implement the first minimal step of the RoleForge Core pipeline:

```text
.rfg file path
    ↓
file existence check
    ↓
read file
    ↓
LoadedFile
```

This task is only about loading an `.rfg` file into memory.

Do **not** implement the Tokenizer, Parser, RoleBlock, Registry, Dispatcher, Role discovery, Role-specific behavior, or Python bindings yet.

## Required model

Create a minimal Rust data structure named `LoadedFile`.

It should contain only the data currently needed by the next pipeline stage:

- the loaded file content
- the file path

Use appropriate Rust standard-library types. Keep the model simple and data-only.

Conceptually:

```text
LoadedFile
├── content
└── path
```

Do not add speculative metadata such as line maps, IDs, timestamps, Role information, or registry information.

## Required loader behavior

Implement a minimal file-loading function for RoleForge `.rfg` files.

The function must:

1. Accept a file path.
2. Check that the requested file exists.
3. Read the file contents as UTF-8 text.
4. Construct a `LoadedFile`.
5. Return the `LoadedFile` on success.

Keep the error handling minimal for this stage. Do not design the final RoleForge error hierarchy yet.

Use idiomatic Rust and the standard library unless the existing project already has an established dependency that is clearly appropriate.

Do not add dependencies unless they are genuinely necessary.

## `.rfg` scope

This loader is intended for RoleForge `.rfg` files.

Do not invent additional file-format behavior or syntax rules in this task.

If extension validation is not already defined by the project rules, do not add a new architectural rule for it silently.

## File header requirement

At the top of every Rust source file created or materially modified for this task, add a short comment describing its input and output.

Keep it concise. Do not write a long explanation.

Example style:

```rust
// Input: Path to a RoleForge .rfg file.
// Output: LoadedFile containing the file path and UTF-8 contents.
```

Adapt the wording to the actual responsibility of each file.

## Tests

Add focused tests only for this task.

At minimum, verify:

- an existing `.rfg` file can be loaded successfully
- the returned `LoadedFile` contains the expected content
- the returned `LoadedFile` contains the expected path
- a missing file produces an error

Do not add tests for Tokenizer, Parser, Registry, Dispatcher, or Role behavior.

## Scope restrictions

Do not:

- implement `RoleBlock`
- implement a tokenizer
- implement a parser
- implement Role registration
- implement Role dispatching
- implement Python bindings
- design a public end-user API
- add Role-specific logic
- refactor unrelated code
- modify the Iron Rules
- add speculative abstractions for future stages

If the existing project structure conflicts with this prompt or the Iron Rules, stop and explain the conflict instead of inventing a solution.

## Completion report

After finishing:

1. Briefly list the files created or modified.
2. State what the loader now does.
3. State which tests were added and whether they pass.
4. Mention any decision you had to avoid making because it is still undefined.

Respond to me in Hebrew.
