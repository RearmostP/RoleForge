# Stage 09 — Live Role Objects and Project Role API

You are continuing development of RoleForge.

Before making any changes, inspect the current repository and read the current architecture documentation, especially:

- `docs/ai/CORE_IRON_RULES.md`
- `docs/ai/STAGE_08_IMPLEMENTATION.md`
- `docs/human/en/README.md`
- `docs/human/en/CREATING_ROLES.md`
- `docs/human/en/IRON_RULES.md`
- `docs/human/he/README.md`
- `docs/human/he/CREATING_ROLES.md`
- `docs/human/he/IRON_RULES.md`
- `roleforge/core/pipeline/prompts/README.md`
- the current Runtime, Handoff, Bridge, Python Bridge, Project, RoleInfo, RoleInput, Registry, and tests.

Treat the current implementation and current architecture documentation as the source of truth.

Historical prompts are development history and must not override newer architectural decisions.

---

# Goal

Stage 08 completed real Core → Python Role delivery.

Stage 09 must add the missing opposite side of that relationship:

A successfully delivered Role instance must become a live Python object accessible through the returned `Project`.

The intended user-facing direction is:

```python
project = load("project.rfg")

project.test.hello()
project.test[0].hello()
project.test[1].hello()
```

For a Role named `Test`:

- `project.test` refers to Role-local instance `0`.
- `project.test[0]` explicitly refers to the same instance.
- `project.test[1]` refers to the second `Test` instance.
- Each discovered Role occurrence must have its own live object.
- Multiple instances of the same Role share the Role's API but must remain separate instances with separate identity/state.

Do not implement `start()` merely because this API now makes it possible. `start()` lifecycle remains a separate concern unless an existing current document explicitly requires otherwise.

---

# Core architectural principle

Preserve the existing architecture:

> Core knows the protocol, never the roles.

> Core defines the data. The Bridge adapts it to the target environment. The Role defines the behavior.

The Core must not learn what `Test`, `Directory`, or any other Role means.

The Core must not contain Role-name-specific branches.

The implementation language of a Role must not become part of the Core routing contract.

`entry.via` remains an opaque Bridge identifier.

---

# 1. RoleInput remains the neutral Core contract

Do not replace `RoleInput` with a Python-specific object model.

The logical Core → Role handoff remains:

```text
RoleInput
├── name
├── index
├── role_index
├── body
└── source
    └── declaration_line
```

`CleanRole` remains an internal discovery/tokenizer representation.

`RoleInput` remains the explicit neutral handoff boundary.

Do not put PyO3/Python objects into Core-neutral data structures.

The Python Bridge is responsible for adapting this neutral RoleInput into the normal Python representation used by Python Roles.

There may also be an advanced way for a Role implementation to work with the clean/raw RoleInput representation when it deliberately wants to manage its own object model. Keep this capability small and optional; do not make it the main Stage 09 API or add repetitive boolean configuration to ordinary Roles.

---

# 2. The Python Bridge owns Python Role-object creation/adaptation

The Python Bridge is the language/environment boundary.

It already converts the neutral Core RoleInput into Python.

Extend this responsibility so that every successfully delivered Python Role occurrence has a live Python-side Role object/representation that can later be exposed by `Project`.

Conceptually:

```text
Rust Core
    │
    │ RoleInput
    ▼
Python Bridge
    │
    ├── Python Role object, role_index 0
    ├── Python Role object, role_index 1
    └── ...
```

Do NOT hardcode classes such as `TestRole`, `DirectoryRole`, etc. in the Bridge.

The Bridge knows how to create/adapt a generic Python Role instance.

The Role implementation defines the behavior/API exposed by that instance.

Keep these responsibilities separate:

```text
Core        → defines/discovers the data
Bridge      → creates/adapts the native Python representation
Role        → defines Role-specific behavior
Project     → exposes the resulting live instances to the user
```

Do not move Role-specific knowledge into the Bridge.

---

# 3. Role instance identity

Do not invent a filename-derived object identity.

The source filename identifies the loaded source/Project context; it is not the identity of each Role occurrence.

Within a Project:

- Role name identifies the Role group.
- `role_index` identifies an occurrence within that Role name.
- `index` remains the global source-order index.

For example:

```text
@role Test
@role Config
@role Test
```

has:

```text
Test    → index=0, role_index=0
Config  → index=1, role_index=0
Test    → index=2, role_index=1
```

The intended live mapping is therefore conceptually:

```text
project.test[0]   → (Test, role_index=0)
project.config[0] → (Config, role_index=0)
project.test[1]   → (Test, role_index=1)
```

`role_index` must not be recomputed by the Python API or Bridge.

Use the identity already assigned by the Core.

---

# 4. Project Role access

Extend the Python-facing `Project` API so discovered and successfully delivered live Roles can be accessed naturally.

Required behavior:

```python
project.test
```

returns/exposes the first `Test` instance.

These must refer to the same underlying live Role instance:

```python
project.test
project.test[0]
```

And:

```python
project.test[1]
```

selects Role-local instance 1.

The Role group/access layer may internally provide indexing behavior, but do not duplicate Role instances just to support the convenience syntax.

The same live instance must back both implicit instance 0 and explicit `[0]`.

Preserve the existing neutral:

```python
project.roles
```

metadata API unless a change is genuinely required.

`RoleInfo` and the live Role object are different concepts:

- `RoleInfo` = discovery/routing metadata.
- live Role object = callable/usable Role API instance.

Do not silently turn `RoleInfo` into the live Role object.

---

# 5. Dynamic attribute naming

The public direction is lower-case Python access for a Role such as:

```text
@role Test
```

through:

```python
project.test
```

However, do not invent a broad CamelCase/PascalCase/snake_case conversion system beyond what Stage 09 actually needs.

Inspect the current naming rules and implement the smallest deterministic mapping necessary for the existing Role naming convention and tests.

If a naming case is ambiguous or would require a new public naming policy, do not silently invent one. Document it as an open decision.

Also ensure normal `Project` attributes/methods cannot be accidentally shadowed without an explicit policy.

---

# 6. Role-specific functions

Extend the existing development `Test` Role so Stage 09 proves that a Role can expose callable behavior through the live object.

Use a minimal function such as:

```python
hello()
```

or equivalent.

The important test is not the text printed by the function.

The important behavior is:

```python
project.test.hello()
project.test[0].hello()
project.test[1].hello()
```

and that calls are routed to the correct live Role occurrence.

The two `Test` occurrences must remain distinct objects.

The implementation should make it possible for Role-specific behavior to observe/use the data belonging to its own occurrence.

Do not require every Python Role developer to write boilerplate whose only purpose is copying:

```python
role.name
role.body
role.index
role.role_index
role.source
```

into another class.

That adaptation is exactly the kind of environment-specific work the Python Bridge should handle.

At the same time, do not make the Bridge understand the meaning of Role-specific functions.

---

# 7. Handoff result semantics

Stage 08 deliberately treated the return value of `roleforge_receive(...)` as having no protocol meaning.

Stage 09 introduces live Role objects, so inspect this boundary carefully.

Do not casually redefine `roleforge_receive` return values as the public Role object unless that is actually necessary for the architecture chosen here.

The preferred architecture is that the Python Bridge owns the normal native Role representation and can retain/expose that representation after successful delivery.

`roleforge_receive` remains the Role-specific receiving hook.

If the implementation requires changing its return semantics, document that as an explicit Stage 09 protocol change and explain why.

Do not accidentally create two competing object-ownership models.

---

# 8. Lifetime and state

A live Role object returned through `Project` must remain valid after `load()` returns.

For example:

```python
project = load("test.rfg")
project.test[1].hello()
```

must work after the Core loading pipeline has completed.

Therefore inspect the current Python Bridge behavior carefully.

Stage 08 intentionally reloads Python Role source for each handoff and does not promise persistent module-global state.

Do not accidentally rely on temporary module bindings or borrowed Python objects whose lifetime ends when handoff returns.

Use proper PyO3 ownership/reference semantics for any Python objects retained by `Project`.

Two occurrences of the same Role must remain distinct even if they originate from the same Python target file.

---

# 9. Preserve Runtime and Handoff boundaries

Do not make Runtime execute Role behavior.

The pipeline remains conceptually:

```text
.rfg
→ Loader
→ Main Tokenizer
→ Registry
→ Dispatcher
→ conflict preflight
→ RoleInput
→ Handoff
→ Bridge
→ Role receiver / native Role representation
→ Project retains successful live Role instances
```

Runtime orchestrates the Core pipeline.

The Role decides what its functions do.

Calling:

```python
project.test.hello()
```

happens after loading and is user-driven Role behavior.

It is not another Runtime pipeline stage.

---

# 10. Error behavior

Preserve current Stage 08 behavior unless Stage 09 requires a narrowly scoped change:

- Registry conflicts are preflighted before delivery.
- A conflict must never be resolved by precedence.
- Unknown Roles are currently reported/skipped.
- Delivery failures stop later deliveries.
- Previous successful deliveries are not rolled back.
- Current console/debug reporting remains temporary.

Add clear Python-facing errors for invalid live Role access where necessary.

Examples worth covering:

```python
project.nonexistent
project.test[999]
```

Do not build the final Error Manager or Console Manager in this stage.

---

# 11. Tests

Add strong Python-facing tests for the new public behavior.

At minimum test:

1. One `Test` Role becomes accessible through `project.test`.
2. `project.test` and `project.test[0]` refer to the same underlying live instance.
3. Two `Test` declarations create two distinct live instances.
4. `project.test[1]` accesses the second instance.
5. Role-specific callable behavior works on both instances.
6. Each instance observes its own RoleInput-derived body/index/role_index.
7. Global `index` and Role-local `role_index` remain the values assigned by Core.
8. Access to a missing Role is handled clearly.
9. Out-of-range Role-local indexing is handled clearly.
10. Existing `project.roles` metadata behavior still works.
11. Existing conflict preflight behavior still prevents delivery/live-object creation.
12. Unknown Roles do not fabricate live objects.
13. A Role object remains usable after `load()` returns.
14. Two instances do not accidentally share per-instance state.
15. Existing Stage 08 delivery/error tests continue to pass.

Prefer Python-facing tests for the public API.

Keep Rust unit/regression tests where they protect internal invariants.

Run at minimum:

```sh
cargo test
python -m unittest discover -s anyone_py_project -v
```

---

# 12. Documentation update is part of Stage 09

After implementation, update all relevant current documentation.

Do not only add a Stage report while leaving existing human documentation describing Stage 08 behavior.

Review and update at least:

```text
README.md

docs/human/en/README.md
docs/human/en/CREATING_ROLES.md
docs/human/en/IRON_RULES.md

docs/human/he/README.md
docs/human/he/CREATING_ROLES.md
docs/human/he/IRON_RULES.md

docs/ai/CORE_IRON_RULES.md
```

Create an appropriate Stage 09 implementation report under `docs/ai/` if that matches the existing documentation convention.

Update:

```text
roleforge/core/pipeline/prompts/README.md
```

to include Stage 09 in the development history.

Documentation must clearly distinguish:

- RoleInput
- RoleInfo
- live Role object
- Role name
- global index
- role-local index
- Project/source identity
- Bridge responsibility
- Role responsibility

Update examples that currently state that `project.test`, indexing, or live Role APIs are not implemented.

Do not document future functionality as already implemented.

Keep English and Hebrew human documentation semantically aligned.

Mention the optional clean/raw RoleInput path only once where appropriate as an advanced escape hatch; do not make it a headline feature or repeat it throughout the documentation.

Historical prompts must remain historical. Do not rewrite old Stage prompts to pretend the new architecture existed earlier.

---

# 13. Do not implement unrelated future features

Stage 09 is NOT the stage for:

- final `start()` lifecycle
- Rust Role delivery
- JavaScript or other Bridges
- Role installation/removal API
- aliases / named Role instances
- final packaging architecture
- final Error Manager
- final Console Manager
- speculative caching
- concurrency
- broad naming-convention systems
- automatic Role discovery outside the Registry
- large refactors unrelated to live Role objects

Keep the change focused.

---

# Acceptance example

With a source such as:

```text
@role Test

hello = first

@role Test

hello = second
```

and a `Test` Role exposing `hello()`, this should work after `load()`:

```python
from roleforge import load

project = load("test.rfg")

project.test.hello()
project.test[0].hello()
project.test[1].hello()
```

The first two calls operate on the same Role occurrence.

The third operates on the second occurrence.

The objects for `[0]` and `[1]` must be distinct and must retain their own RoleInput-derived state.

The Core must not contain special knowledge of `Test` or `hello`.

---

# Final report

After implementation, report:

1. The architecture chosen for live Python Role objects.
2. How the Python Bridge creates/retains them.
3. How Role-specific behavior is attached/exposed without teaching the Bridge individual Roles.
4. How `project.<role>` and `[role_index]` are implemented.
5. How object lifetime is preserved after `load()`.
6. Any explicit protocol changes from Stage 08.
7. Files changed.
8. Tests added/updated.
9. Exact test results.
10. Documentation updated.
11. Any unresolved design questions that were intentionally left open.

Do not silently decide unrelated future architecture.

Respond to me in Hebrew.