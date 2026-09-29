# Stage 08 — Python Role Handoff

Implement the first real end-to-end Role handoff in RoleForge.

Stage 07 established the Core Bridge abstraction and proved that Bridges can be registered and resolved by opaque identifiers.

Stage 08 must now connect the existing Core pipeline to that Bridge system and implement the first real delivery mechanism:

> Core → Handoff → Python Bridge → Python Role → `roleforge_receive(...)`

This stage also introduces the stable Core-to-Role handoff data contract and replaces the temporary Rust Bridge placeholder with a Python-only real implementation.

The goal is not to create a general cross-language execution system.

The goal is to make one complete, clean, real handoff path work first: Python.

---

# Stage 08 Goals

After this stage, RoleForge should support the following real flow:

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
  ↓
entry.via
  ↓
Bridge Resolver
  ↓
Python Bridge
  ↓
entry.target
  ↓
Python Role module
  ↓
roleforge_receive(role)
```

A resolved Python Role must actually receive its RoleForge input.

Do not report successful delivery unless `roleforge_receive(...)` was actually called successfully.

---

# Preserve Existing Architecture

Preserve the established RoleForge principles:

> Core knows the protocol, never the roles.

> The source determines what the Role is. The Role determines what API and behavior it exposes.

> Role discovery must happen before Role-specific behavior is exposed.

> The Core resolves destinations, not implementations.

> Components process. Runtime orchestrates.

> No major pipeline component directly drives the next major component.

> The Core resolves and delivers. The Role decides what happens next.

> An undecided detail is not a decision.

> Optimize obvious waste immediately. Postpone architectural optimization.

Do not redesign unrelated completed stages.

---

# Important New Decision: One Stable Role Handoff Contract

Stage 08 introduces one stable Core handoff object for Roles.

Use a dedicated neutral Core structure for the data delivered to a Role.

Use an appropriate project-consistent name such as:

```text
RoleInput
```

unless repository inspection reveals a clearly better existing naming convention.

The conceptual contract is:

```text
RoleInput
├── name
├── index
├── role_index
├── body
└── source
    └── declaration_line
```

The fields correspond to the existing discovered Role metadata.

Do NOT create new indexes.

Do NOT recalculate indexes inside Bridges.

Do NOT remove source metadata.

Do NOT make the Role input depend on Python.

Do NOT create a different Core input model for every Bridge.

The important architectural rule is:

> Every Role receives the same logical RoleForge handoff contract regardless of implementation environment.

Different Bridges may need to represent that contract differently inside their destination environment, but the logical data contract originates from one stable Core model.

---

# CleanRole vs RoleInput

Currently, Stage 07 passes `CleanRole` directly to the Bridge contract.

Stage 08 should introduce a deliberate boundary between tokenizer-internal output and the official Role handoff contract.

Conceptually:

```text
Tokenizer
   ↓
CleanRole
   ↓
Core pipeline
   ↓
Handoff boundary
   ↓
RoleInput
   ↓
Bridge
   ↓
Role
```

`CleanRole` remains the neutral result of Core discovery/tokenization.

`RoleInput` becomes the stable data contract delivered across the Core-to-Role boundary.

Do not duplicate data unnecessarily throughout the entire pipeline.

Create `RoleInput` at the appropriate handoff boundary.

Keep the transformation simple and explicit.

The purpose of this separation is architectural stability:

> Internal tokenizer representation should not automatically become the permanent public contract of every Role implementation.

---

# RoleInput Must Be Environment-Neutral

`RoleInput` belongs to the Core.

It must not contain Python-specific objects or PyO3 types.

For example, do NOT design the Core structure as:

```text
PyObject
PyDict
Python module
Python callable
```

The Core-side structure must remain ordinary neutral Rust data.

The Python Bridge is responsible for exposing an equivalent representation to Python.

Future Bridges must be able to represent the same logical RoleInput contract without changing the Core pipeline.

---

# Mandatory Role Receiving Entry Point

Every Role implementation must expose a receiving entry point compatible with its Bridge.

For Python Roles, the mandatory receiving function is:

```python
def roleforge_receive(role):
    ...
```

This name is part of the RoleForge Python Role contract.

Do NOT search for alternative names.

Do NOT fall back to:

```text
receive
main
start
run
load
init
```

If `roleforge_receive` is missing, the Python Role does not satisfy the RoleForge receiving contract and delivery must fail explicitly.

The reason for the RoleForge-specific name is intentional:

A Role developer remains free to use names such as:

```python
receive()
start()
run()
```

for internal Role behavior.

`roleforge_receive` is reserved as the explicit RoleForge handoff entry point.

---

# roleforge_receive Is NOT start()

Keep these concepts strictly separate.

```text
roleforge_receive(role)
```

means:

> RoleForge is delivering this discovered Role instance and its data to the Role implementation.

It is the Core-to-Role handoff boundary.

It does NOT mean:

> Execute the Role's normal user-facing behavior.

`start()` remains a separate Role-level operation.

Conceptually:

```text
Core
 ↓
Python Bridge
 ↓
roleforge_receive(role)
 ↓
Role now owns/understands the delivered instance
────────────────────────────────────────────
Core responsibility ends
```

A later user operation may conceptually be:

```python
project.directory.start()
```

but Stage 08 must NOT implement or automatically call `start()` merely because delivery occurred.

Do not make:

```text
roleforge_receive == start
```

and do not make `roleforge_receive` automatically invoke `start()` unless a future explicit design decision says so.

---

# Registry Entry

Replace the current single-string Role entry with the approved structured entry concept:

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

The semantic meaning is:

```text
via
    = Bridge identifier

target
    = opaque destination interpreted by the selected Bridge
```

`via` is NOT a programming-language field.

This distinction is mandatory.

Do NOT introduce:

```json
"language": "python"
```

Do NOT introduce a Core enum such as:

```rust
enum Language {
    Python,
    Rust,
}
```

The Core must treat:

```text
via = "python"
```

as:

> Resolve the Bridge registered under the exact identifier `python`.

Nothing more.

---

# Bridge Identifier Semantics

Bridge identifiers are opaque registration identifiers.

For example:

```text
python
```

is convenient because the currently registered Bridge happens to communicate with Python.

But generic Core routing must not attach programming-language semantics to that string.

Conceptually, this must remain possible:

```text
via = "hii_im_boby"
```

if a Bridge has explicitly been registered under that identifier.

Therefore:

> `via` selects a Bridge registration. It does not declare a programming language.

Do not infer Bridges from:

- Role name
- target extension
- `.py`
- `.rs`
- Role body
- source file
- implementation type

Only explicit Bridge registration and exact identifier resolution determine the Bridge.

---

# Registry Parsing and Resolution

Update Registry parsing to understand:

```text
entry.via
entry.target
```

instead of treating `entry` as only a path string.

Use a small structured representation appropriate for the current Rust architecture.

Conceptually:

```text
RoleEntry
├── via
└── target
```

The exact Rust names may follow repository conventions.

Preserve the existing registry rules:

```text
Role exists in exactly one registry
    → resolvable

Role exists in neither registry
    → Unknown

Role exists in both builtin and dynamic registries
    → Conflict
```

There must still be NO builtin/dynamic precedence.

A Conflict must never silently resolve to one registration.

---

# Target Resolution

Be careful with the existing RoleForge path rules.

Currently:

```text
dynamic relative entries
    → relative to roleforge/roles/

builtin relative entries
    → relative to roleforge/builtin_roles/

absolute entries
    → remain absolute
```

Preserve the intended behavior where applicable to `entry.target`.

Do not make target resolution depend on the Python process current working directory.

Do not silently redefine target semantics beyond what is required for the Python file target in this stage.

The selected Bridge receives the resolved target appropriate for its delivery work.

If repository inspection reveals that preserving the existing path-resolution boundary requires a small adjustment to where target resolution occurs, keep the responsibility explicit and document it.

Do not let generic Bridge selection infer anything from the target path.

---

# Dispatcher

Update Dispatcher output so a resolved Role carries enough information for Handoff to perform:

```text
Role data
+
Bridge identifier
+
target
```

Preserve:

```text
global index
Role-local index
body
source metadata
source order
```

Do not make Dispatcher execute the Bridge.

Dispatcher resolves routing information.

It does not perform Role delivery.

Preserve:

> Components process. Runtime orchestrates.

---

# Handoff

Stage 08 completes the actual Handoff path for supported Bridges.

The Handoff receives the resolved Role routing result and uses:

```text
via
```

to resolve the Bridge.

It then provides the selected Bridge with:

```text
target
+
RoleInput
```

Conceptually:

```text
Resolved Role
   │
   ├── via
   ├── target
   └── Role data
        ↓
Handoff
        ↓
Bridge Resolver
        ↓
Bridge
        ↓
deliver(target, RoleInput)
```

Do not put Python-specific delivery logic in Handoff.

Handoff must not import Python modules itself.

Handoff must not inspect `.py`.

Handoff must not call `roleforge_receive` directly.

Those are Python Bridge responsibilities.

---

# Preserve Stage 06 Preflight Semantics

Before any actual delivery begins, preserve the complete preflight behavior introduced in Stage 06.

The complete dispatch result collection must be inspected first.

Temporary current policy:

```text
Unknown
    → temporary report
    → skip that Role
    → continue

Conflict
    → temporary report
    → abort ALL delivery for this load operation

Resolved
    → eligible for delivery
```

This is especially important now that delivery becomes real.

For example:

```text
Resolved A
Resolved B
Conflict C
Resolved D
```

must NOT result in:

```text
deliver A
deliver B
discover conflict
stop
```

Instead:

```text
preflight all results
        ↓
Conflict found
        ↓
deliver NOTHING
```

No Role may receive data before conflict preflight succeeds for the entire operation.

This remains a temporary policy until the final structured error system is designed.

---

# Bridge Resolution Failure

A Registry entry may reference a Bridge identifier that is not registered.

For example:

```json
{
  "Directory": {
    "entry": {
      "via": "hii_im_boby",
      "target": "directory/main.py"
    }
  }
}
```

when no Bridge named `hii_im_boby` exists.

This must NOT:

- fall back to Python
- infer Python from `.py`
- silently skip as success
- panic
- pretend delivery occurred

Represent the failure explicitly using the smallest structured error appropriate to the current architecture.

Do not build the final Error Manager in this stage.

---

# Remove the Temporary Rust Bridge

Stage 07 introduced both:

```text
PythonBridge
RustBridge
```

as placeholders proving the common Bridge abstraction.

We have now explicitly decided NOT to define Rust Role delivery yet.

Remove the current temporary Rust Bridge implementation from the active Bridge system.

Do not keep a fake Rust Bridge that always returns:

```text
DeliveryUnavailable
```

merely to reserve the name.

Do not register:

```text
"rust"
```

as a built-in Bridge in the current system.

If the dedicated Rust Bridge source file now has no valid purpose, remove it cleanly.

Update affected tests.

Do not replace it with TODO architecture or speculative ABI code.

Rust Role delivery is a future feature and remains intentionally undefined.

Historical Stage 07 documentation/prompt must remain historical and must NOT be rewritten to pretend RustBridge never existed.

Live architecture documentation, however, must describe the current state accurately:

> Python is currently the first implemented Role delivery Bridge.

Future Rust delivery remains undecided.

---

# Python Bridge — Real Delivery

Replace the Stage 07 placeholder Python Bridge behavior with real Python Role delivery.

The Python Bridge receives:

```text
resolved target
+
RoleInput
```

It must:

1. Load the Python module represented by the target.
2. Find the mandatory callable:

```text
roleforge_receive
```

3. Expose the logical RoleInput contract to Python.
4. Call:

```python
roleforge_receive(role)
```

5. Treat successful return as successful handoff.
6. Convert loading/call failures into explicit structured Bridge/Handoff errors appropriate for the current architecture.

The Python Bridge is implemented in Rust.

Do not create a Python implementation of the Bridge itself.

The existing project already uses PyO3 for its Python-facing API.

Inspect the current PyO3 integration and reuse appropriate infrastructure rather than creating a second unrelated Python embedding architecture.

Do not duplicate conversions unnecessarily.

---

# Python Representation of RoleInput

Python must receive one Role object, not a long positional parameter list.

Required conceptual API:

```python
def roleforge_receive(role):
    print(role.name)
    print(role.index)
    print(role.role_index)
    print(role.body)
    print(role.source.declaration_line)
```

Do NOT design:

```python
def roleforge_receive(
    name,
    index,
    role_index,
    body,
    declaration_line,
    ...
):
    ...
```

The single object is intentional.

It gives RoleForge one stable handoff contract that can evolve carefully without turning every field into part of a positional function signature.

The Python representation should correspond directly to the Core RoleInput structure.

Do not expose unrelated Core internals.

Do not give Python direct ownership of tokenizer-specific implementation details beyond the approved handoff contract.

Use immutable/read-only behavior where practical and consistent with the existing Python API.

---

# Source Metadata

Preserve source information.

At minimum the Python Role must be able to obtain the existing:

```text
declaration_line
```

through the delivered Role object.

Conceptually:

```python
role.source.declaration_line
```

Do not flatten or discard source metadata merely because only one source field currently exists.

The architecture already treats source information as a distinct concept, and future diagnostics may extend it.

Do not overbuild future source mapping in this stage.

---

# Python Module Loading

Implement only the minimum reliable loading behavior needed for Python file targets.

The target identifies the Python Role implementation file.

Do not invent:

- package discovery frameworks
- Python plugin marketplaces
- import scanning
- automatic module naming conventions
- environment-wide Role searches
- fallback imports
- `sys.path` heuristics
- file-extension-based Bridge selection

The Bridge was already selected by `via`.

The Python Bridge may naturally validate/use the target as required to load the selected Python module, but target interpretation belongs inside that Bridge.

Use a deterministic loading strategy.

Avoid accidental module-name collisions when multiple Role targets are loaded.

Do not depend on the user's current working directory for target resolution.

---

# Receiver Validation

The Python Bridge must explicitly validate the receiving contract.

At minimum distinguish:

```text
target cannot be loaded

roleforge_receive is missing

roleforge_receive exists but is not callable

roleforge_receive raised an exception

delivery succeeded
```

Do not silently reinterpret these as one another.

Do not search for alternative receiver names.

Do not automatically execute another function after successful receipt.

---

# Delivery Boundary

A successful call to:

```python
roleforge_receive(role)
```

means the Core successfully delivered that Role instance.

At that point:

> Core responsibility ends.

The Role may internally:

- tokenize its own body
- parse its own DSL
- build its own objects
- store the received instance
- prepare its own API
- perform Role-specific validation if its own design requires it

Those behaviors belong to the Role.

Do NOT move them into Core or Python Bridge.

---

# Return Value of roleforge_receive

Do not invent a Role execution result protocol in Stage 08.

The purpose of `roleforge_receive` is receipt/handoff.

Unless an existing project decision already defines otherwise, successful completion of the call is sufficient to represent successful delivery.

Do not assign semantic meaning to arbitrary Python return values.

Do not create a result serialization protocol.

A Python exception means the handoff call failed and should be represented as a delivery failure.

---

# Runtime

Runtime remains the Core orchestrator.

Conceptually, Stage 08 Runtime becomes:

```text
load file
   ↓
tokenize
   ↓
dispatch
   ↓
preflight all dispatch results
   ↓
if conflict:
    abort delivery
else:
    for eligible resolved Roles in source order:
        create RoleInput
        resolve via
        call selected Bridge with target + RoleInput
```

Do not make Registry drive Dispatcher.

Do not make Dispatcher drive Handoff.

Do not make Bridge drive Runtime.

Runtime owns execution order.

---

# Delivery Order

After successful preflight, resolved Roles should be handed off in existing source order.

Preserve the global Role order already represented by the pipeline.

Unknown Roles are skipped under the current temporary policy.

Do not reorder delivery by:

- Role name
- Bridge
- target
- registry source
- Role-local index

Source order remains authoritative for discovered Role ordering.

---

# Python API

Preserve the existing Stage 05 Python API unless a minimal change is required to expose the result of the now-real handoff.

Existing concepts include:

```python
from roleforge import load

project = load("project.rfg")
```

Do not redesign the public Project API in this stage.

Do not implement dynamic:

```python
project.directory
```

unless it already exists or is strictly required for this stage.

Do not implement `start()` in this stage.

The main goal is that `load()` can now perform the Core pipeline and successfully deliver resolved Python Roles through `roleforge_receive`.

Be explicit in tests about what `load()` currently does.

---

# Important Distinction: load() and start()

The existing architectural direction remains:

```text
load()
    → discover/load/prepare Roles

start()
    → Role-level operation
```

Stage 08 makes `load()` capable of completing the required receiving handoff.

That does NOT mean `load()` should automatically execute Role-specific `start()` behavior.

Do not merge these concepts.

---

# Error Handling

The final Error Manager and Console Manager still do not exist.

Do not build them in Stage 08.

Use small structured internal errors for the new failure cases.

Potential conceptual categories include:

```text
UnknownBridge
PythonTargetLoadFailure
MissingReceiver
ReceiverNotCallable
ReceiverRaised
```

These names are conceptual.

Follow current project naming conventions and avoid unnecessary error hierarchy.

Do not turn the Bridge layer into a God Object.

Temporary console/debug behavior may remain where required by existing Stage 06 architecture, but new delivery logic should not scatter arbitrary `println!` calls throughout components.

---

# Testing

Add real end-to-end Python handoff tests.

The tests should use a minimal Python Role target containing:

```python
def roleforge_receive(role):
    ...
```

Verify at minimum:

### Successful delivery

A registered Python Role is discovered, routed through:

```text
via = python
```

and its `roleforge_receive(role)` function is actually called.

Verify the received object contains:

```text
name
index
role_index
body
source.declaration_line
```

with the correct values.

### Multiple instances

Multiple instances of the same Role preserve:

```text
global index
Role-local index
source order
```

through actual Python delivery.

### Missing receiver

A valid Python target without:

```python
roleforge_receive
```

fails explicitly.

### Receiver not callable

A target containing something such as:

```python
roleforge_receive = 123
```

fails explicitly.

### Receiver exception

A receiver that raises an exception is reported as delivery failure.

### Missing/invalid target

A target that cannot be loaded fails explicitly.

### Unknown Bridge

A Registry entry with an unregistered `via` fails explicitly and does not fall back based on the target.

### Conflict preflight

If any Registry conflict exists, verify that NO Python receiver is called, including receivers belonging to resolved Roles appearing before the conflict.

This is critical now that handoff has real side effects.

### Unknown Role behavior

Preserve the current temporary Stage 06 policy:

```text
Unknown
    → report/skip
    → later eligible resolved Roles may still be delivered
```

### No automatic start

A Python Role may define:

```python
def start():
    ...
```

Verify that Stage 08 handoff does NOT automatically call it.

Only:

```python
roleforge_receive(role)
```

is the mandatory receiving entry point.

---

# Test Philosophy

Continue to preserve the user's preference for meaningful Python-facing tests.

Internal Rust tests are appropriate for:

- Registry entry parsing
- Bridge resolution
- RoleInput construction
- Core invariants

But the real handoff must also be proven from the Python-facing side.

Do not claim Stage 08 works merely because isolated Rust Bridge tests pass.

There must be a real Python Role receiving real RoleInput through `roleforge_receive`.

Run all existing Rust and Python tests after implementation.

Do not delete useful existing regression tests merely because the stage changed.

Update tests whose Stage 07 expectations intentionally changed, especially the temporary Rust Bridge tests.

---

# Remove Obsolete Stage 07 Placeholder Behavior

After real Python delivery exists, remove obsolete behavior that always returned:

```text
DeliveryUnavailable { bridge: "python" }
```

for the Python Bridge.

Do not leave dead placeholder code next to the real implementation.

Likewise remove the active Rust Bridge placeholder as specified earlier.

Clean up dead imports/tests/modules resulting from these changes.

Do not remove historical documentation showing what Stage 07 did at that point in development.

---

# Documentation / Iron Rules

Update the live AI Iron Rules to reflect the decisions approved in Stage 08.

Document explicitly:

> Every Role receives one common logical RoleInput contract from the Core.

> RoleInput is environment-neutral and belongs to the Core-to-Role boundary.

> Bridges adapt the same logical RoleInput contract to their destination environments.

> Every Role implementation must expose a receiving entry point compatible with its Bridge.

> For Python Roles, the mandatory receiving entry point is `roleforge_receive(role)`.

> `roleforge_receive` is the RoleForge handoff entry point and is separate from `start()` and Role-specific APIs.

> Bridge identifiers are opaque registration identifiers, not programming-language declarations.

> Registry entries select a Bridge using `via` and provide its destination using `target`.

> Generic Handoff resolves Bridges; environment-specific delivery remains inside each Bridge.

> Python is currently the first real Role delivery Bridge.

> Rust Role delivery is not currently defined or registered as a working built-in Bridge.

Also update stale live-documentation sections where they directly conflict with already approved and implemented project state.

In particular, accurately distinguish:

```text
approved architectural contract
implemented current behavior
future unresolved behavior
```

Do NOT silently invent resolutions for still-open decisions.

Historical prompt files remain historical.

Do not rewrite Stage 07's prompt to remove the Rust Bridge from history.

---

# Rust Bridge Future Status

After Stage 08, live documentation should make the state clear:

```text
Python Bridge
    → implemented real delivery

Rust Bridge
    → not currently implemented
    → receiving/loading/ABI mechanism intentionally undecided
```

Do not create a fake placeholder implementation merely to make the architecture diagram symmetrical.

A future Rust Bridge should use the same logical RoleInput contract, but its physical receiving mechanism will be designed only when explicitly decided.

---

# Do Not Implement

Do NOT implement in Stage 08:

- Rust Role loading
- Rust plugin ABI
- dynamic library conventions
- `.dll` / `.so` Role protocol
- Rust exported symbol conventions
- JavaScript/Lua/other Bridges
- automatic Bridge inference
- programming-language metadata
- automatic `start()`
- final dynamic `project.directory` API unless strictly required
- Role aliases
- named Role instances
- `interaction_mode`
- final Error Manager
- final Console Manager
- logging framework
- caching
- concurrency
- async Role execution
- process isolation
- serialization protocol
- network Role delivery
- package/plugin marketplace
- speculative optimization
- unrelated architectural refactoring

Keep Stage 08 focused.

---

# Architecture After Stage 08

The intended architecture after this stage is:

```text
Python user
    │
    │ load("project.rfg")
    ▼
Python API
    ▼
Rust Core Runtime
    │
    ├── Loader
    │
    ├── Main Tokenizer
    │
    ├── Registry
    │      └── entry { via, target }
    │
    ├── Dispatcher
    │
    ├── Handoff preflight
    │
    └── RoleInput creation
            │
            ▼
       Bridge Resolver
            │
            │ via = "python"
            ▼
       Python Bridge — Rust
            │
            │ target
            │ RoleInput
            ▼
       Python Role module
            │
            ▼
 roleforge_receive(role)
            │
            ▼
     Core responsibility ends
```

The Core still does not understand the Role's DSL.

The Core still does not execute Role-specific `start()` behavior.

The Python Bridge understands only how to perform the Python receiving handoff.

The Python Role owns everything after successful receipt.

---

# Completion Criteria

Stage 08 is complete only when all of the following are true:

1. The Registry supports structured:

```text
entry.via
entry.target
```

2. `via` is treated strictly as an opaque Bridge identifier.

3. The current Rust Bridge placeholder is removed from the active implementation.

4. Python is the only currently implemented real built-in Role Bridge.

5. A neutral Core `RoleInput` handoff contract exists.

6. The RoleInput contract preserves:

```text
name
index
role_index
body
source.declaration_line
```

7. Handoff converts existing Role data into RoleInput at the appropriate boundary.

8. Handoff resolves the selected Bridge without environment-specific conditionals.

9. Python Bridge loads the selected Python target.

10. Python Bridge requires:

```python
roleforge_receive(role)
```

11. Python receives one object representing RoleInput.

12. Missing/non-callable receivers fail explicitly.

13. Python receiver exceptions fail explicitly.

14. Successful receiver completion counts as successful delivery.

15. `start()` is not automatically invoked.

16. Conflict preflight still prevents ALL delivery before any side effects occur.

17. Unknown Role behavior remains consistent with Stage 06.

18. Unknown Bridge identifiers never fall back or infer a Bridge.

19. Actual Python handoff is covered by Python-facing tests.

20. Existing Rust and Python regression tests pass after intentional updates.

21. Live Iron Rules reflect the current architecture.

22. Historical prompts remain historical.

---

# Before Finishing

Inspect the current repository before implementation.

Do not rely only on this prompt's examples if the repository has evolved in naming or module placement.

After implementation:

1. Run the complete Rust test suite.
2. Run the complete Python test suite.
3. Report the exact test results.
4. Report any compiler warnings.
5. Report every file created, modified, removed, or renamed.
6. Explicitly confirm that the Rust Bridge placeholder was removed.
7. Explicitly confirm that Python Role delivery is real and no longer returns the Stage 07 placeholder `DeliveryUnavailable`.
8. Show the final Registry entry shape.
9. Show the final Python `roleforge_receive(role)` contract.
10. Explain where `CleanRole → RoleInput` occurs.
11. Explain where Core responsibility ends.
12. List any remaining unresolved decisions without inventing solutions.
13. Compare the live Iron Rules against the resulting project state and report any remaining stale or incomplete sections.
14. Do not commit or push unless explicitly asked.

Save this Stage 08 prompt according to the project's prompt-history convention as:

```text
08_python_role_handoff.md
```

Respond to me in Hebrew.