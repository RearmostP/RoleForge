# Stage 04 — Core Runtime / Orchestrator

Implement the RoleForge **Core Runtime / Orchestrator**.

This stage should connect the existing Core components into one complete execution pipeline.

The Runtime should be intentionally small and simple.

> Components process. Runtime orchestrates.

The Runtime must not reimplement the responsibilities of the Loader, Main Tokenizer, Registry, or Dispatcher. Its job is to take the output of one Core component and pass it as the input to the next component.

---

## Goal

For the first time, RoleForge should be able to take a real `.rfg` file and run it through the complete currently implemented Core pipeline:

```text
.rfg file
    ↓
Runtime
    ↓
Loader
    ↓
LoadedFile
    ↓
Main Tokenizer
    ↓
Vec<CleanRole>
    ↓
Registry
    ↓
Dispatcher
    ↓
DispatchResult
    ↓
Temporary Handoff
```

The Runtime owns the orchestration of this sequence.

It does not own the internal processing performed by any component.

---

## Runtime Responsibility

The Runtime should:

1. Receive the initial file/path input.
2. Pass it to the existing Loader.
3. Receive the Loader output.
4. Pass that output to the existing Main Tokenizer.
5. Receive the discovered `CleanRole` values.
6. Use the existing Registry and Dispatcher to resolve the Roles.
7. Process the resulting dispatch results in source order.
8. Pass successfully resolved Roles to the temporary handoff boundary.

The Runtime should coordinate these operations, not duplicate them.

Do not move Loader logic into the Runtime.

Do not move tokenizer logic into the Runtime.

Do not move registry lookup logic into the Runtime.

Do not move dispatcher logic into the Runtime.

---

## Component Boundaries

Do not introduce artificial `input()` and `output()` functions merely for architectural symmetry.

The existing component functions already define their boundaries through their function inputs and return values.

Conceptually:

```text
Loader:
input → path
output → LoadedFile

Tokenizer:
input → LoadedFile
output → Vec<CleanRole>

Dispatcher:
input → Vec<CleanRole> + Registry
output → Vec<DispatchResult>
```

The Runtime connects these boundaries.

---

## Runtime Boundary

The Runtime orchestrates the **Core pipeline only**.

It must never understand or execute Role-specific behavior.

Keep this rule:

> Core Runtime orchestrates the Core pipeline, never Role behavior.

And:

> The Core resolves and delivers. The Role decides what happens next.

The Runtime may orchestrate delivery to the Role handoff boundary, but it must not know what the Role does after that point.

For example, the Runtime must never contain logic such as:

```text
if role == "Directory":
    run Directory parser
```

or any equivalent Role-specific behavior.

---

## Temporary Handoff

Do **not** implement the real Role handoff mechanism yet.

The actual mechanism for loading/invoking Role implementations is outside the scope of Stage 04.

Instead, create the smallest reasonable **temporary development handoff** that allows us to verify that the entire Core pipeline works end-to-end.

For a successfully resolved Role, temporarily print useful information such as:

```text
[HANDOFF]
Role: Directory
Global Index: 0
Role Index: 0
Entry: ...
```

The exact formatting is not important.

The important information is:

- Role name
- global Role index
- Role-local index
- resolved entry/destination

Preserve the original source order.

This print behavior is temporary development infrastructure and must be clearly treated as such.

It is **not** the final Console/Output architecture.

Do not build the future Console Manager in this stage.

Do not turn printing into a general logging system.

---

## Unknown and Conflict Results

Use the behavior already established by the Registry and Dispatcher.

The Runtime must not invent precedence or silently resolve conflicts.

Remember:

- registered in exactly one registry → resolved
- registered in neither registry → unknown
- registered in both registries → conflict

A conflict must never be resolved by choosing one registry over the other.

Unknown or conflicting Roles must not prevent later Roles from continuing through the pipeline unless an existing Core error already requires execution to stop.

For this stage, keep handling minimal and consistent with the existing structures.

Do not design the full Error Manager yet.

---

## Role Metadata

The Runtime must preserve the existing `CleanRole` metadata through the pipeline.

In particular:

```rust
index
```

is the zero-based **global index** among all Roles discovered in source order.

And:

```rust
role_index
```

is the zero-based **Role-local index** among Roles with the same name.

The Runtime must not recalculate either index.

They were already determined by the Main Tokenizer and should simply survive the pipeline unchanged.

---

## First End-to-End Core Test

Add an integration-style test or equivalent test coverage for the Runtime.

The purpose is to verify the complete currently implemented Core flow rather than testing each component in isolation.

Use a realistic `.rfg` input containing multiple Roles, including repeated Role names.

Conceptually:

```rfg
@role Directory
src/

@role Config
debug = true

@role Directory
assets/
```

The test should verify that the pipeline preserves:

- source order
- Role names
- global indexes
- Role-local indexes
- Role bodies
- source metadata where relevant
- correct registry resolution
- correct dispatch destination
- continued processing of multiple Roles

Where practical, also verify behavior for unknown/conflicting Roles using the existing Registry/Dispatcher semantics.

Avoid making tests depend unnecessarily on console formatting.

The important thing to test is the Runtime pipeline and the data reaching the handoff boundary.

---

## Architecture Constraint

The Runtime should remain small.

If implementing this stage requires large amounts of new processing logic inside the Runtime, reconsider the design.

The intended architecture is approximately:

```text
Runtime
  │
  ├── Loader
  │
  ├── Main Tokenizer
  │
  ├── Registry
  │
  ├── Dispatcher
  │
  └── Temporary Handoff
```

The Runtime owns **execution order**.

Each component owns **its own processing**.

---

## Do Not Implement

Do not implement any of the following in Stage 04:

- real Role execution
- Role-specific parsers
- Role-specific runtimes
- Python/Rust Role loading
- FFI
- plugin loading
- dynamic library loading
- Python public API
- final `Project` object model
- final `load()` public API design
- Console Manager
- full Error Manager
- caching
- concurrency
- async execution
- speculative performance architecture
- Role aliases
- `interaction_mode`
- unrelated refactoring

Do not redesign working Stage 01–03 components unless a small mechanical change is genuinely required to connect them to the Runtime.

---

## Optimization Rule

Follow the existing RoleForge development principle:

> Optimize obvious waste immediately. Postpone architectural optimization.

Avoid unnecessary copies, repeated registry loading, repeated scans, or other obvious local waste where they can be avoided simply.

Do not introduce architectural complexity for hypothetical future performance.

---

## Expected Result

After Stage 04, we should be able to provide RoleForge with a real `.rfg` file and have the Core automatically perform:

```text
Load
→ Tokenize
→ Resolve
→ Dispatch
→ Temporary Handoff
```

This will be the first real end-to-end execution of the RoleForge Core.

The next stage can replace or extend the temporary handoff with the real Role handoff mechanism without requiring the Runtime architecture to be redesigned.

Before finishing:

1. Run the relevant existing tests.
2. Run the new Runtime tests.
3. Confirm that previous Stage 01–03 behavior remains unchanged.
4. Report what files were created or modified.
5. Briefly explain the resulting Runtime flow.
6. Mention any architectural issue you encountered instead of silently inventing a new design decision.

Do not modify historical prompt files from previous stages.

If a Stage 04 prompt/history file is part of the existing project convention, add this stage consistently with that convention.

Respond to me in Hebrew.