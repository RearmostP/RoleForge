# Stage 05 — Minimal Python API

Implement the first minimal Python-facing API for RoleForge.

The Rust Core from Stages 01–04 already provides the current pipeline:

```text
.rfg
  ↓
Loader
  ↓
Main Tokenizer
  ↓
Registry
  ↓
Dispatcher
  ↓
Runtime
  ↓
Final Core Result
  ↓
Handoff boundary
```

Stage 05 should make this existing Core usable from Python for the first time.

Do not implement the real Role handoff yet.

The goal is to establish the minimal Python API and prove that a Python caller can successfully invoke the Rust Core end-to-end.

---

## Primary User Experience

The intended Python usage begins with:

```python
from roleforge import load

project = load("project.rfg")
```

`load()` is the simple public entry point.

The user must not need to know which Roles exist in the file before calling `load()`.

The source file determines which Roles exist.

Keep the existing architectural rule:

> The source determines what the Role is. The Role determines what API and behavior it exposes.

Role discovery happens during loading.

---

## Meaning of `load()`

For this stage, `load("project.rfg")` should:

1. Accept the `.rfg` file path from Python.
2. Enter the Rust Core.
3. Run the existing Core Runtime.
4. Let the Runtime orchestrate the existing Loader, Main Tokenizer, Registry, and Dispatcher.
5. Reach the final Core result immediately before the real Role handoff boundary.
6. Return a Python-side `Project` object representing the loaded RoleForge project.

Do not duplicate Core processing in Python.

Python should be a public interface over the existing Rust Core, not a second implementation of the Core pipeline.

---

## Project

Introduce the minimum `Project` abstraction required for the Python API.

Conceptually:

```python
project = load("project.rfg")
```

`project` represents the loaded `.rfg` file and the Roles discovered from it.

Eventually the intended API should support access such as:

```python
project.directory
project.config
```

and multiple instances such as:

```python
project.directory[0]
project.directory[1]
```

However, do not overbuild the final dynamic Role API in this stage if doing so requires implementing the real Role handoff or Role loading system.

Stage 05 should establish only the minimum structure necessary to support the future API cleanly.

Do not invent fake Role-specific Python classes.

---

## Default Role Instance

The intended public API semantics are:

```python
project.directory
```

refers to Role-local index `0`.

Therefore, conceptually:

```python
project.directory
```

and:

```python
project.directory[0]
```

refer to the same first Role instance.

Explicit indexing is used for additional instances:

```python
project.directory[1]
project.directory[2]
```

The existing Core metadata already provides:

```text
index       = global source-order index
role_index  = index among Roles with the same name
```

Do not recalculate these indexes in Python.

The Role-local index produced by the Core is the source of truth.

If implementing these access semantics fully requires decisions belonging to the real Handoff/Role API stage, preserve the design requirement but do not prematurely invent the missing mechanism.

Do not add warnings for implicit index `0` yet.

That may be considered later.

---

## Required Role Entry Point

The intended Role contract should reserve a standard operation named:

```python
start()
```

Every Role implementation will eventually be required to provide a `start()` entry point.

This gives Roles that are driven entirely by their `.rfg` body a common execution mechanism.

Conceptually:

```python
project.directory.start()
```

means:

```python
project.directory[0].start()
```

A Role may eventually expose additional Role-specific operations:

```python
project.database.start()
project.database.connect()
project.database.migrate()
```

Only `start()` is intended to be common RoleForge behavior.

Methods such as `connect()`, `migrate()`, `validate()`, `create()`, etc. belong to individual Roles and must never be hardcoded into the Core.

### Important Stage 05 Boundary

Do NOT implement actual Role execution or the real `start()` behavior yet.

The real Role implementation is still beyond the Handoff boundary.

Stage 05 should preserve this API direction without pretending that Role execution already exists.

---

## Temporary Final-Core Debug Output

We still do not have the real Handoff.

For Stage 05, keep a small temporary debug mechanism that allows us to inspect the final result produced by the Core immediately before Handoff.

This is conceptually:

```text
Runtime
   ↓
Final Core Result
   ↓
TEMPORARY DEBUG PRINT
   ↓
Future real Handoff
```

The existing Stage 04 `temporary_handoff` should no longer be conceptually treated as a fake Role handoff.

Its purpose is only to inspect the final Core output before the real Handoff exists.

Rename or minimally adjust it if necessary so that the code clearly communicates this responsibility.

The debug output should expose useful final Core information such as:

```text
Role: Directory
Global Index: 0
Role Index: 0
Body:
src/
tests/

Entry: ...
```

The purpose is to verify that the Python call successfully traveled through the complete Rust Core and produced the correct final Core data.

This debug mechanism is temporary development infrastructure.

It is NOT:

- the real Handoff
- a Console Manager
- a logging framework
- a public utility API
- a permanent part of RoleForge

Do not create a `tools/` package in this stage.

---

## Python → Rust Boundary

Use the smallest clean mechanism appropriate for exposing the Rust Core to Python.

Keep the boundary narrow.

Conceptually:

```text
Python

load("project.rfg")
       │
       ▼
Python/Rust boundary
       │
       ▼
Rust Runtime
       │
       ├── Loader
       ├── Main Tokenizer
       ├── Registry
       └── Dispatcher
       │
       ▼
Final Core Result
       │
       ▼
temporary debug inspection
       │
       ▼
Project returned to Python
```

Do not redesign the Rust Core around Python.

The Python layer adapts to the Core architecture, not the other way around.

If a binding library such as PyO3 is needed, use it only as the interoperability layer.

Do not allow binding-specific concerns to leak into Loader, Tokenizer, Registry, Dispatcher, or other unrelated Core components.

---

## Registry

The Python user should not normally have to manually construct or pass the Core `Registry`.

The public call should remain simple:

```python
project = load("project.rfg")
```

The API layer may load the stored RoleForge Registry as part of entering the Core pipeline.

Do not expose internal Registry mechanics unnecessarily through the public Python API.

Avoid repeatedly loading the Registry during one `load()` operation.

---

## Role Exposure

The Core does not know in advance which Roles will exist.

Therefore, do not hardcode properties such as:

```python
directory
config
database
```

into `Project`.

Role exposure must eventually be dynamic based on the Roles discovered from the source.

For example, if a third-party Role named `ExampleRole` is discovered, the architecture must not require modifying the Core just to expose it.

Keep this rule:

> Core knows the protocol, never the roles.

The exact final mechanism for converting Role names into Python attribute names may require additional design.

Do not silently invent naming/conversion rules that have not been decided.

---

## Preserve Core Metadata

The Python-facing representation should preserve access to the important identity of a discovered Role where needed:

- Role name
- global index
- Role-local index
- body
- source metadata
- resolved destination/entry

Do not throw this information away at the Python boundary.

Do not create a second indexing system.

---

## Testing Direction

This stage marks the beginning of testing RoleForge primarily from the Python user's perspective.

Create the minimum test/example necessary to prove this flow:

```python
from roleforge import load

project = load("project.rfg")
```

using a real `.rfg` file.

The test should prove that:

```text
Python
  → Rust boundary
  → Runtime
  → Loader
  → Tokenizer
  → Registry
  → Dispatcher
  → final Core result
```

works successfully.

The temporary debug output may be used during development to visually confirm the final Core result.

Prefer testing observable Python-facing behavior rather than creating another large set of Rust-only Runtime tests.

The existing Stage 04 Runtime test file may be removed if it is no longer wanted as part of the project testing strategy, but do not remove useful production code merely because its Rust tests are removed.

Do not weaken the architecture simply to make testing easier.

---

## Public API Philosophy

Keep these principles:

> Simple by default, explicit when needed.

and:

> One core operation, multiple convenience interfaces.

For the current stage, the simple public operation is:

```python
load("project.rfg")
```

Do not create a large public API yet.

Do not create a God Object.

`RoleForge`, `Project`, and individual Roles should remain conceptually separate responsibilities.

---

## Do Not Implement

Do not implement the following in Stage 05:

- real Role Handoff
- actual Role execution
- actual `start()` execution
- Role-specific parsers
- Role-specific runtime behavior
- hardcoded built-in Role APIs
- `DirectoryRole`
- `ConfigRole`
- Role aliases
- named Role instances
- `interaction_mode`
- Console Manager
- full Error Manager
- `tools/` utilities
- logging framework
- caching
- concurrency
- async execution
- speculative optimization
- unrelated Core refactoring

Do not redesign Stages 01–04 unless a small mechanical change is genuinely required for the Python boundary.

---

## Architecture Check

At the end of Stage 05, this should be true:

```text
Python user
    │
    │ load("project.rfg")
    ▼
Python API
    │
    ▼
Rust Core Runtime
    │
    ├── Loader
    ├── Main Tokenizer
    ├── Registry
    └── Dispatcher
    │
    ▼
Final Core Result
    │
    ├── temporary debug inspection
    │
    ▼
Python Project
```

The Core still does not understand Role behavior.

The Python API still does not reimplement Core behavior.

The real Handoff still remains a separate future stage.

---

## Before Finishing

1. Run the relevant Rust tests that remain.
2. Run the new Python-facing test/example.
3. Verify that a real `.rfg` file can enter through Python and reach the end of the existing Rust Core pipeline.
4. Verify that Role order and both indexes survive the Python/Rust boundary.
5. Verify that no Role-specific behavior was added to the Core.
6. Report all files created, removed, renamed, or modified.
7. Explain briefly how `load()` travels from Python into the Rust Runtime.
8. Clearly report any missing architectural decision instead of inventing one.
9. Do not modify historical Stage 01–04 prompt files.

If the project convention stores stage prompts, save this as:

```text
05_first_python_api.md
```

Respond to me in Hebrew.
