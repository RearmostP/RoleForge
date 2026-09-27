# Stage 07 — Core Bridges

Implement the first Bridge layer inside the RoleForge Rust Core.

This stage introduces a Core-owned abstraction that will later allow the Handoff system to deliver Role data to different execution environments without making the Handoff itself understand those environments.

The Bridge system is part of the Core.

It MUST be implemented in Rust.

Stage 07 introduces two built-in Bridges:

- Python Bridge
- Rust Bridge

Their names describe the kind of destination they know how to communicate with.

They do NOT describe the implementation language of the Bridge itself.

Both Bridges are Rust Core components.

---

# Architectural Context

The current conceptual pipeline is:

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
Handoff preflight
```

Stage 06 established temporary Handoff preflight behavior:

```text
Unknown
    → temporary report
    → skip that Role
    → continue

Conflict
    → temporary report
    → abort ALL Handoff for the load operation

Resolved
    → eligible for future Handoff
```

Stage 06 intentionally left one question unresolved:

> How does a resolved Role actually reach its implementation?

Stage 07 begins answering that question by introducing Bridges.

However, Stage 07 does NOT yet integrate Bridges into Role Registry entries.

That integration belongs to Stage 08.

---

# Core Principle

Preserve the existing RoleForge rules:

> Core knows the protocol, never the roles.

> The Core resolves and delivers. The Role decides what happens next.

> The Core resolves destinations, not implementations.

> Components process. Runtime orchestrates.

The Handoff must eventually be able to work with Bridges without containing environment-specific delivery logic itself.

Conceptually:

```text
Handoff
   ↓
Bridge
   ↓
Role destination
```

The Handoff should not need logic such as:

```rust
if python {
    ...
} else if rust {
    ...
}
```

Environment-specific delivery belongs inside the appropriate Bridge.

---

# Bridge Meaning

A Bridge is a Core component responsible for knowing how to communicate with a particular kind of Role destination.

For example:

```text
Python Bridge
    → knows how to communicate with Python Role destinations

Rust Bridge
    → knows how to communicate with Rust Role destinations
```

Both Bridges themselves are implemented in Rust.

Conceptually:

```text
RoleForge Core — Rust
│
├── pipeline/
│
├── bridges/
│   ├── Python Bridge — Rust
│   └── Rust Bridge   — Rust
│
├── registry.rs
└── storage/
```

Do not create Python implementation files for the Python Bridge.

The Python Bridge is Rust code that will eventually know how to deliver Role input to Python.

---

# Bridge Contract

Create a small common internal contract for Core Bridges.

The purpose is that future Core code can interact with a Bridge through one consistent abstraction instead of understanding each Bridge individually.

Conceptually, a Bridge will eventually receive something equivalent to:

```text
target
+
Role data
```

and be responsible for delivering the Role data to that target.

The Role data originates from the Core and includes the existing discovered Role information, such as:

```text
name
global index
Role-local index
body
source metadata
```

Do NOT create a second indexing or metadata system.

Do NOT unnecessarily duplicate `CleanRole`.

Prefer reusing existing neutral Core structures where appropriate.

The exact Stage 08 entry structure is NOT part of this stage.

---

# Important: Do Not Invent the Stage 08 Entry Format Yet

We have a planned direction for the future Registry entry:

```json
{
  "Directory": {
    "entry": {
      "via": "python",
      "target": "directory/main.py"
    }
  }
}
```

The intended meaning is:

```text
via
    → identifies which Bridge should be used

target
    → identifies the destination that Bridge should receive
```

However, this Registry format belongs to Stage 08.

Do NOT migrate the current Registry format in Stage 07.

Do NOT modify existing Role Registry JSON structures merely to prepare for it.

Do NOT integrate `via` or `target` into the Dispatcher yet.

Stage 07 builds the Bridge system independently first.

---

# Bridge Names

The future Bridge identifier is NOT fundamentally a programming-language type.

For example:

```text
python
rust
```

are convenient names.

Conceptually a Bridge could instead be registered as:

```text
hii_im_boby
```

and the Core should not derive semantic behavior from that string.

The identifier means only:

> Select the Bridge registered under this identifier.

Therefore, do NOT create architecture such as:

```rust
enum Language {
    Python,
    Rust,
}
```

Do NOT make Core routing depend on programming-language detection.

Do NOT inspect file extensions to determine a Bridge.

Do NOT infer a Bridge from the Role body.

Bridge identity and programming language are separate concepts.

The built-in Bridges may conveniently be named:

```text
python
rust
```

but these are identifiers, not language semantics understood by generic routing code.

---

# Bridge Collection / Resolution

Create the smallest reasonable Core mechanism that allows a Bridge to be resolved by identifier.

Conceptually:

```text
"python"
    ↓
Bridge Resolver
    ↓
Python Bridge
```

and:

```text
"rust"
    ↓
Bridge Resolver
    ↓
Rust Bridge
```

The generic resolver must not contain Role-specific behavior.

It must also avoid architecture such as:

```rust
if bridge_name == "python" {
    ...
} else if bridge_name == "rust" {
    ...
}
```

if a small registration-based design can cleanly avoid it.

The purpose is to make future Bridges easy to add without redesigning the Handoff.

For example, future additions could conceptually include:

```text
javascript
lua
external_process
custom_runtime
```

Do NOT implement those Bridges now.

Only Python and Rust are required in Stage 07.

Keep the design small.

Do not build a plugin framework merely because more Bridges may exist later.

---

# Python Bridge

Add the first built-in Python Bridge as part of the Rust Core.

The Python Bridge must conform to the common Bridge contract.

Remember:

```text
Python Bridge ≠ Python code

Python Bridge = Rust Core code that knows how to communicate with Python destinations
```

The project already uses PyO3 for the Python-facing RoleForge API.

Inspect the existing implementation before deciding whether any existing PyO3 infrastructure is relevant.

Do not duplicate existing Python integration unnecessarily.

However, do NOT force actual Role loading/execution in this stage if doing so requires unresolved Stage 08 information such as the final target/entry contract.

The purpose of Stage 07 is to establish the Bridge and its common interface correctly.

Do not invent a fake Role execution mechanism merely to make the Bridge appear complete.

---

# Rust Bridge

Add the first built-in Rust Bridge as part of the Rust Core.

The Rust Bridge must conform to exactly the same generic Bridge contract exposed to the rest of the Core.

Generic Core code should not need a special path such as:

```text
Handoff → special Rust handling
```

Instead:

```text
Handoff
   ↓
Bridge abstraction
   ↓
Rust Bridge
```

Do not introduce dynamic-library ABI assumptions unless they are genuinely required by the current stage and already defined by the project.

Do not invent:

- `.dll` conventions
- `.so` conventions
- exported symbol names
- C ABI contracts
- dynamic loading rules

Those decisions have not been made yet.

If actual Rust Role execution requires such a decision, leave execution unresolved and clearly report it.

---

# What Must Be Real in Stage 07

Stage 07 must create a real Core Bridge architecture, not merely comments describing one.

At minimum, after this stage the Core should have:

1. A dedicated Bridge area/module inside the Rust Core.
2. A common Bridge contract/abstraction.
3. A Python Bridge implementing that contract.
4. A Rust Bridge implementing that contract.
5. A generic mechanism for resolving/selecting a Bridge by identifier.
6. Tests proving that the generic Bridge mechanism recognizes and distinguishes the two built-in Bridges without Role-specific logic.

The exact names and Rust types should follow the existing project's naming style.

Inspect the repository before choosing them.

Do not create unnecessary layers solely to satisfy this list.

---

# What Does NOT Need to Be Real Yet

Stage 07 does NOT need to successfully execute:

```text
Python Role source
```

or:

```text
Rust Role source
```

if doing so requires Stage 08's unresolved target contract.

A valid Stage 07 endpoint can conceptually be:

```text
Bridge resolved successfully
        ↓
Bridge is ready to receive future target + Role data
```

rather than:

```text
Role executed successfully
```

Do not pretend delivery occurred when it did not.

Tests and debug output must distinguish Bridge resolution from actual Role delivery.

---

# Runtime

The Bridge layer is part of the Core architecture and will become a Runtime stage.

Conceptually the future Runtime will contain:

```text
Loader
   ↓
Tokenizer
   ↓
Registry
   ↓
Dispatcher
   ↓
Handoff preflight
   ↓
Bridge resolution
   ↓
Bridge
   ↓
Role target
```

However, Stage 07 must not force incomplete Bridge execution into the existing Runtime merely to show this diagram.

If the Runtime cannot yet select a Bridge because `via` does not exist until Stage 08, keep the Bridge system independently testable in Stage 07.

Stage 08 will perform the real integration.

Do NOT hardcode a default Bridge into Runtime.

Do NOT assume every current Role is Python.

Do NOT assume every current Role is Rust.

---

# Existing Handoff Behavior

Preserve the Stage 06 behavior exactly unless a minimal internal change is required for compilation:

```text
Unknown
    → report temporarily
    → skip

Conflict
    → report temporarily
    → abort all Handoff

Resolved
    → eligible for future delivery
```

Do not weaken the Conflict preflight guarantee.

No Role may begin delivery before the complete DispatchResult collection has been checked for conflicts.

---

# Errors

Do not implement the final Error Manager or Console Manager in Stage 07.

If Bridge resolution needs an internal result/error for cases such as an unknown Bridge identifier, use the smallest structured internal representation appropriate for the current architecture.

Do not turn the Bridge resolver into an Error Manager.

Do not create permanent console presentation rules here.

The final structured error/event system remains a future stage.

---

# Testing

Add focused tests for the Bridge layer.

At minimum verify:

### Built-in Python Bridge

```text
resolve("python")
```

selects the Python Bridge.

### Built-in Rust Bridge

```text
resolve("rust")
```

selects the Rust Bridge.

### Unknown Bridge

An arbitrary unregistered identifier does not silently fall back to Python, Rust, or any other Bridge.

For example:

```text
resolve("hii_im_boby")
```

must be unknown unless that identifier was explicitly registered.

### Generic identifiers

The generic resolver must treat identifiers as identifiers, not programming-language declarations.

### Same contract

Python Bridge and Rust Bridge must satisfy the same common Bridge abstraction.

### No Role-specific knowledge

Tests must not require hardcoded Role names such as only:

```text
Directory
Config
Database
```

The Bridge layer operates independently of Role identity.

---

# File Structure

Use the existing repository structure and naming conventions.

A conceptual structure is:

```text
roleforge/
└── core/
    ├── bridges/
    │   ├── mod.rs
    │   ├── python.rs
    │   └── rust.rs
    │
    ├── pipeline/
    ├── registry.rs
    └── storage/
```

This is conceptual, not a mandatory exact file layout.

If a slightly different Rust module structure fits the current repository better, use it.

The important architectural rule is:

> Bridges belong to the Rust Core.

Do not create a top-level user-facing `bridges/` package outside Core.

---

# Do Not Implement

Do NOT implement in Stage 07:

- Stage 08 Registry entry migration
- `entry.via`
- `entry.target`
- final Handoff → Bridge integration
- actual Role `start()`
- Role-specific execution
- Role-specific parsing
- programming-language detection
- file-extension-based Bridge detection
- Python-vs-Rust conditionals in generic Handoff code
- hardcoded Role names
- dynamic-library ABI design
- exported Rust Role symbol conventions
- executable/process protocol
- Role aliases
- named Role instances
- `interaction_mode`
- final Error Manager
- Console Manager
- logging framework
- caching
- concurrency
- async execution
- speculative optimization
- unrelated Core refactoring

Do not modify historical Stage 01–06 prompt files.

---

# Architecture After Stage 07

The architecture should conceptually contain:

```text
RoleForge Core
│
├── Existing pipeline
│   │
│   ├── Loader
│   ├── Tokenizer
│   ├── Registry
│   ├── Dispatcher
│   └── Runtime / Handoff preflight
│
└── Bridge System
    │
    ├── common Bridge contract
    │
    ├── Bridge resolver
    │
    ├── "python"
    │   └── Python Bridge implemented in Rust
    │
    └── "rust"
        └── Rust Bridge implemented in Rust
```

The Bridge system exists and is testable.

The existing Role Registry does not yet select Bridges.

That connection belongs to Stage 08.

---

# Iron Rules

Preserve all existing RoleForge Iron Rules.

Add the following architectural principles where appropriate in the live AI architecture documentation:

> Bridges are Core components and are implemented in Rust.

> A Bridge identifier selects a registered Bridge; it is not a programming-language type.

> Environment-specific delivery behavior belongs to the Bridge, not to generic Handoff logic.

> The Handoff must not infer implementation language from Role names, source, targets, or file extensions.

> Adding a new Bridge should not require teaching generic Handoff logic about a new programming language.

Do not modify historical prompts to reflect these new decisions.

Historical prompts remain development history.

---

# Before Finishing

1. Inspect the current repository and Stage 06 implementation before changing code.
2. Read and preserve the current Iron Rules.
3. Implement the Bridge layer inside the Rust Core.
4. Implement the Python Bridge in Rust.
5. Implement the Rust Bridge in Rust.
6. Give both Bridges the same common Core contract.
7. Provide generic Bridge resolution by identifier.
8. Do not integrate the future `{ via, target }` Registry entry yet.
9. Do not fake actual Role execution.
10. Preserve Stage 06 Conflict and Unknown behavior.
11. Add focused tests for Bridge resolution and the common contract.
12. Run all existing Rust and Python tests to ensure there are no regressions.
13. Report the test results.
14. Report every file created, modified, removed, or renamed.
15. Explain briefly how the Bridge abstraction works.
16. Explicitly list anything that remains unresolved for Stage 08 instead of inventing it.
17. Save this prompt according to the existing project convention as:

```text
07_core_bridges.md
```

Respond to me in Hebrew.