# RoleForge — Stage 02: Main Tokenizer

Implement Stage 02 of the RoleForge Core: the Main Tokenizer.

Before making any changes, read the existing project structure, existing code, and the current RoleForge Core Philosophy / Iron Rules if they are available in the repository.

Do not redesign existing architecture or implement future stages.

The RoleForge Core is written in Rust.

---

## Goal

Stage 01 already provides a `LoadedFile`.

Stage 02 must transform:

```text
LoadedFile
    ↓
Main Tokenizer
    ↓
Vec<CleanRole>
```

This stage ends at `Vec<CleanRole>`.

Do NOT implement Runtime, Dispatcher / Router, Registry, Role resolution, Role execution, Role-specific parsing, Main Parser, or Python bindings/API. Those belong to later stages.

---

## Core Principle

The Main Tokenizer understands only RoleForge's outer syntax. It must NEVER understand or interpret Role-specific syntax.

A Role body is opaque content from the Core's perspective, except for the global full-line comment rule defined below.

---

## Role Declaration

A Role begins with:

```text
@role RoleName
```

Example:

```text
@role Directory
```

Role names should conventionally use PascalCase, similar to Python class names, e.g. `Directory`, `Database`, `PlayerInventory`.

Do NOT enforce PascalCase as a strict naming rule in this stage. Do not invent additional Role naming restrictions unless a minimal restriction is technically required by the implementation.

A Role continues until the next `@role` declaration or EOF. There is no `@end` syntax.

---

## Main Tokenizer Internal Design

The Main Tokenizer should internally work in two passes.

### Pass 1 — Role/File Scanning

Scan the outer structure of the loaded `.rfg` source.

This pass is responsible for identifying full-line RoleForge comments outside/between Roles, Role declarations, and raw Role regions/bodies. It must not interpret Role-specific content.

Use a clear name such as `role_scanner.rs` rather than a vague name such as `outer.rs`.

The intermediate representation may contain types such as `RawRole` and comment tokens as appropriate. Do not expose unnecessary intermediate representations outside the tokenizer subsystem.

### Pass 2 — Role Content Processing

Process the raw body of each discovered Role. Conceptually divide Role content into Code/content and Full-line comments, then remove the content of full-line comments while preserving their newline.

The result of this pass is the clean Role body. The final Main Tokenizer output is `Vec<CleanRole>`.

---

## Comment Rules

RoleForge uses `#` for global full-line comments.

A line is considered a RoleForge full-line comment when the first non-whitespace character on that line is `#`.

Examples that ARE comments:

```text
# comment
    # comment with indentation
```

Examples that are NOT Core-level comments:

```text
value = 10 # inline comment
color = #FF0000
```

Inline `#` content must remain untouched. The Core does NOT implement inline comment parsing. If an individual Role wants inline comments, that Role will be responsible for defining and processing them later.

This rule must also eventually be documented for Role authors, but do not create a Role-authoring system as part of this stage.

### Preserve line structure

When removing a full-line comment, remove its comment content but preserve its newline.

For example:

```text
abc
# comment
def
```

must conceptually become:

```text
abc

def
```

NOT:

```text
abc
def
```

This is important so original line positions remain meaningful for future source metadata and error reporting. Do not implement a larger source-metadata system in this stage.

---

## CleanRole

The final output item should conceptually contain:

```rust
struct CleanRole {
    index: usize,
    name: String,
    body: String,
}
```

`index` represents the Role's order of appearance in the source file and starts at 1. The index is positional, NOT a persistent Role ID. If Role order changes, indexes may change.

---

## Suggested File Structure

```text
roleforge/core/pipeline/
├── loader.rs
├── models.rs
├── tokenizer/
│   ├── mod.rs
│   ├── tokenizer.rs
│   ├── role_scanner.rs
│   ├── role_content.rs
│   └── tokens.rs
└── prompts/
```

### `mod.rs`
Defines the tokenizer module boundary and exposes only what the rest of the Core needs. Do not place substantial tokenization logic here.

### `tokenizer.rs`
Internal orchestrator for the Main Tokenizer. It coordinates the tokenizer's internal passes. Major tokenizer subcomponents should not drive each other directly. For example, `role_scanner.rs` should not call `role_content.rs`; `tokenizer.rs` coordinates them. This is internal orchestration inside one component and does not require a Runtime.

### `role_scanner.rs`
Responsible for the first pass: loaded source → Role declarations / raw Role regions / outer comments. It understands RoleForge outer structure but not Role-specific syntax.

### `role_content.rs`
Responsible for the second pass. It processes a raw Role body, identifies Core-level full-line comments, removes their content while preserving line structure, and produces the clean body. It must leave inline `#` content untouched.

### `tokens.rs`
Contains tokenizer-specific data structures and intermediate token types. Keep data structures separate from processing logic.

If `CleanRole` is more appropriate in the existing neutral `models.rs` because it is the public output of the tokenizer pipeline stage, use that location instead. Choose based on ownership and architectural clarity, not convenience.

---

## File Responsibility Headers

Continue the convention established in Stage 01. Every new source file should begin with a short Input / Output responsibility comment, e.g.:

```rust
// Input: Loaded RoleForge source content.
// Output: Raw Role regions discovered from the outer RoleForge syntax.
```

The exact wording should match the actual responsibility of each file.

---

## Component Boundaries

Keep responsibilities explicit.

> Components process.  
> Orchestrators coordinate.

Internal helper calls inside a component are fine. Do not introduce Runtime merely to coordinate the tokenizer's own internal passes. Do not make Stage 02 depend on future components.

---

## Implementation Quality

Apply obvious local optimizations immediately when they are simple and preserve clarity. Avoid unnecessary file I/O, allocations, string copies, repeated scans, repeated computation, and temporary data.

Do NOT introduce speculative architectural optimization such as caching systems, concurrency, unsafe code, custom allocators, or complex performance abstractions.

Prefer simple, idiomatic Rust.

Optimize obvious waste immediately. Postpone architectural optimization until there is demonstrated need.

---

## Tests

Add focused tests for the Main Tokenizer. At minimum, cover:

1. A single Role.
2. Multiple Roles.
3. Correct indexes starting from 1.
4. Correct Role names.
5. Role body preservation.
6. Full-line comments outside Roles.
7. Full-line comments inside Role bodies.
8. Indented full-line comments.
9. Preservation of newlines when comments are removed.
10. Inline `#` content remaining untouched.
11. A final Role ending at EOF.
12. UTF-8 Role body content.

Also add reasonable edge-case tests discovered naturally during implementation, but do not invent new language semantics just to satisfy an edge case.

If an edge case requires an architectural or syntax decision that is not specified here, do not silently choose a major new rule. Keep the implementation minimal and report the unresolved decision.

---

## Scope Discipline

Do not implement future stages. Do not add abstractions merely because they may be useful later.

Do not create `scanner.rs`, `output.rs`, `comments.rs`, or similar extra files unless they have a clear current responsibility that materially improves the implementation.

Do not modify the Stage 01 historical prompt merely because the implementation has evolved. Files under `prompts/` are development history, not the live specification.

Keep the implementation focused on:

```text
LoadedFile -> Main Tokenizer -> Vec<CleanRole>
```

After implementation, summarize the files created or modified, the responsibility of each file, the final public/internal tokenizer entry point, the exact `CleanRole` representation, tests added, and any unresolved decisions or assumptions.

Respond to me in Hebrew.
