# Stage 06 — Role Handoff

Implement the first real Role Handoff boundary in RoleForge.

The current Core pipeline is:

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
```

Stage 06 adds the next boundary:

```text
Final Core Result
  ↓
Handoff
  ↓
Resolved Role destination
```

Keep this stage intentionally small.

The Handoff must not understand Role behavior, Role syntax, or Role implementation language.

Keep the existing architectural rules:

> The Core resolves and delivers. The Role decides what happens next.

> The Core resolves destinations, not implementations.

> Core knows the protocol, never the roles.

---

## Handoff Responsibility

The Handoff has one primary responsibility:

> Deliver a resolved Role and its Core-produced data to the destination already resolved by the Registry/Dispatcher.

Conceptually:

```text
handoff(role, entry)
```

The Handoff must NOT:

- parse Role bodies
- interpret Role instructions
- know Role-specific commands
- contain Role-specific behavior
- determine what programming language a Role uses
- contain `if python`, `if rust`, or equivalent language routing
- resolve Registry conflicts
- search the registries again
- recalculate indexes
- decide what a Role should do after receiving its input

The Role implementation is responsible for being compatible with the input contract it receives.

Its implementation language is irrelevant to the Core.

---

## Dispatcher Results

The Dispatcher already produces the three states required by this stage:

```rust
DispatchResult::Resolved {
    role,
    entry,
}

DispatchResult::Unknown {
    role,
}

DispatchResult::Conflict {
    role,
    builtin_entry,
    dynamic_entry,
}
```

Do not duplicate Registry resolution logic inside Handoff.

The Handoff/Runtime must respect these existing results.

---

# Resolved

For:

```text
Resolved
```

the Role is eligible for Handoff.

The Handoff receives:

- the discovered Role data
- its resolved destination/entry
- all existing Core metadata required by the future Role implementation

Preserve at least:

- Role name
- global index
- Role-local index
- body
- source metadata
- resolved entry

Do not create another indexing system.

Do not discard Core metadata at the boundary.

---

# Unknown — Temporary Behavior

For:

```text
Unknown
```

there is no destination to hand the Role to.

For Stage 06, temporarily print a clear console message such as:

```text
[RoleForge] Unknown Role: ExampleRole
```

Then skip Handoff for that Role and continue processing the remaining Roles.

Example:

```text
Directory → Resolved → Handoff
Missing   → Unknown  → temporary message → skip
Config    → Resolved → Handoff
```

Unknown is NOT a fatal pipeline condition in Stage 06.

This console output is temporary development behavior.

Do NOT build the final Error Manager or Console Manager yet.

Add a clear comment/TODO explaining that this direct output must later be replaced by the structured error/event system.

---

# Conflict — Temporary Fatal Behavior

Conflict behaves differently from Unknown in Stage 06.

If ANY:

```text
DispatchResult::Conflict
```

exists in the results for the loaded file, the entire Role Handoff operation must be aborted.

No Role from that operation may be handed off.

This is intentionally conservative temporary behavior until the real error system exists.

Example:

```text
Directory → Resolved
Config    → Resolved
Shared    → Conflict
Database  → Resolved
```

The result must NOT be:

```text
Directory → Handoff
Config    → Handoff
Shared    → Conflict
Database  → skipped
```

because that would allow partial execution before discovering the conflict.

Instead:

```text
Dispatcher results
        ↓
Check for conflicts across the complete result set
        ↓
Conflict exists
        ↓
Print temporary conflict message
        ↓
ABORT ALL HANDOFF
```

Therefore:

```text
Directory → NOT handed off
Config    → NOT handed off
Shared    → Conflict
Database  → NOT handed off
```

This means conflict detection must happen before the first real Handoff occurs.

Do not begin executing/delivering resolved Roles and then discover a later conflict.

For now, print a clear temporary message containing useful information, for example:

```text
[RoleForge] Role conflict: Shared
Builtin entry: ...
Dynamic entry: ...
Handoff aborted.
```

If multiple conflicts exist, it is acceptable and preferable to report all conflicts that are already present in the Dispatcher results before aborting Handoff.

Do not invent Registry precedence.

A conflicting Role must never be resolved by choosing Builtin or Dynamic automatically.

---

## Important Temporary Rule

For Stage 06:

```text
Unknown
    → report
    → skip that Role
    → continue

Conflict
    → report
    → abort ALL Role Handoff for this load operation

Resolved
    → hand off only if no Conflict exists anywhere in the result set
```

This conflict behavior is temporary.

In a future stage, after the structured Error Manager exists, we may change the behavior so that only conflicting Roles are prevented from running while unrelated valid Roles can continue.

Do NOT implement that future behavior now.

---

# Runtime Orchestration

Runtime remains the orchestrator.

Keep:

> Components process. Runtime orchestrates.

The Runtime should conceptually perform:

```text
load
 ↓
tokenize
 ↓
dispatch
 ↓
inspect complete DispatchResult collection
 ↓
if any Conflict:
    report conflict(s)
    abort Handoff
else:
    process results in source order
        ↓
        Resolved → Handoff
        Unknown  → report and skip
```

Do not move orchestration responsibility into the Registry or Dispatcher.

Do not make the Handoff drive the Runtime.

---

# Handoff Input Contract

Create the smallest clean internal representation necessary for delivering a Role.

Do not create abstractions merely for symmetry.

If the existing:

```text
CleanRole + resolved entry
```

already provides the correct information at this stage, prefer using it rather than creating unnecessary wrapper types.

However, keep the boundary explicit enough that the Handoff can later be replaced or extended without making Role-specific behavior part of the Core.

Remember:

> An undecided detail is not a decision.

Do not invent future Role ABI details that Stage 06 does not require.

---

# `start()` Boundary

The public API direction established in Stage 05 remains:

```python
project.directory.start()
```

with:

```python
project.directory
```

representing Role-local index `0`.

Every Role will eventually be required to provide a common:

```python
start()
```

entry point.

However, Stage 06 should NOT redesign the Python Role API.

Focus on the Handoff boundary.

If actual invocation of a Role's `start()` requires an additional protocol/ABI decision that has not yet been defined, do not invent that decision.

Implement only what can be correctly implemented from the current architecture.

Clearly report any missing boundary decision.

---

# Final Core Debug

Stage 05 introduced temporary final-Core debug output.

Now that Stage 06 introduces the real Handoff boundary, reevaluate this temporary debug mechanism.

It may remain temporarily if it is still useful for verifying exactly what reaches the Handoff.

However:

- do not turn it into permanent logging
- do not create `tools/`
- do not create Console Manager
- do not duplicate the same information unnecessarily
- keep it clearly marked as temporary development infrastructure

The debug mechanism must remain conceptually separate from the Handoff itself.

---

# Python API

Do not redesign the Stage 05 Python API.

The existing basic usage remains:

```python
from roleforge import load

project = load("project.rfg")
```

Preserve the current Python/Rust boundary.

Do not reimplement Core processing in Python.

Do not hardcode Role names into Python.

Do not implement fake Role-specific classes merely to demonstrate Handoff.

---

# Testing

The primary behavioral tests for this stage should verify the Handoff rules from the Python/user-facing side where practical.

At minimum verify:

### 1. Resolved Roles

With only valid resolved Roles:

```text
Resolved
Resolved
Resolved
```

all are eligible for Handoff in source order.

### 2. Unknown Role

For:

```text
Resolved
Unknown
Resolved
```

the Unknown Role is reported and skipped.

The later Resolved Role must still reach Handoff.

### 3. Conflict

For:

```text
Resolved
Resolved
Conflict
Resolved
```

NO Role may reach Handoff.

This is critical.

The test must prove that Roles appearing before the Conflict were not handed off either.

### 4. Multiple Conflicts

If multiple conflicts exist, no Handoff occurs.

Prefer reporting all discovered conflicts before aborting.

### 5. Metadata

Verify that the Role reaching the Handoff preserves:

- name
- global index
- Role-local index
- body
- source metadata
- resolved entry

### 6. Dynamic Role Names

Do not build tests around only hardcoded Builtin Role names.

The architecture must remain generic.

---

# Do Not Implement

Do not implement in Stage 06:

- Role-specific behavior
- Role-specific parsing
- hardcoded Role names
- programming-language detection
- language-based Handoff routing
- Builtin-over-Dynamic precedence
- Dynamic-over-Builtin precedence
- full Error Manager
- Console Manager
- logging framework
- `tools/`
- Role aliases
- named Role instances
- `interaction_mode`
- caching
- concurrency
- async execution
- speculative optimization
- unrelated Core refactoring

Do not change established Stage 01–05 behavior unless required for the Handoff boundary.

---

# Architecture Check

At the end of Stage 06, the architecture should conceptually be:

```text
Python
  ↓
load()
  ↓
Rust Runtime
  ↓
Loader
  ↓
Tokenizer
  ↓
Registry
  ↓
Dispatcher
  ↓
Vec<DispatchResult>
  ↓
Runtime preflight
  │
  ├── Conflict exists
  │       ↓
  │   temporary report
  │       ↓
  │   abort ALL Handoff
  │
  └── no Conflict
          ↓
      source order
          │
          ├── Unknown
          │      ↓
          │   temporary report
          │      ↓
          │     skip
          │
          └── Resolved
                 ↓
              Handoff
                 ↓
           Role destination
```

The Handoff itself must remain small.

The Core delivers.

The Role owns everything after delivery.

---

# Before Finishing

1. Inspect the current Stage 05 implementation before changing anything.
2. Preserve the existing Core architecture and Iron Rules.
3. Implement the smallest real Handoff boundary possible.
4. Ensure the complete Dispatcher result set is checked for conflicts before ANY Handoff occurs.
5. Verify Unknown does not stop unrelated resolved Roles.
6. Verify Conflict prevents ALL Handoff for that load operation.
7. Preserve Role order and both indexes.
8. Run the relevant tests.
9. Prefer Python-facing behavioral tests for the main end-to-end behavior.
10. Report all files created, removed, renamed, or modified.
11. Explain briefly where the new Handoff sits in the Runtime pipeline.
12. Clearly identify any unresolved decision instead of inventing one.
13. Do not modify historical Stage 01–05 prompt files.
14. Save this stage prompt according to the existing project convention as:

```text
06_role_handoff.md
```

Respond to me in Hebrew.