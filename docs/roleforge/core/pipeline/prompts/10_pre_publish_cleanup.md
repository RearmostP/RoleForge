# Stage 10 — Pre-Publishing Cleanup

## Goal

Prepare the RoleForge repository for the upcoming packaging and publishing stage by cleanly separating production code from test code and clarifying the purpose of the external Python test project.

This stage is strictly a repository cleanup/refactor stage.

Do NOT implement PyPI publishing, packaging changes, release automation, new RoleForge features, or new Role behavior in this stage.

The existing behavior of RoleForge must remain unchanged.

---

## Context

RoleForge has now reached its first functional library milestone.

The complete flow currently works:

```text
.rfg source
→ Loader
→ Main Tokenizer
→ Registry
→ Dispatcher
→ Runtime / Handoff
→ Bridge
→ RoleInput
→ live Role object
→ Project API
```

For example:

```python
from roleforge import load

project = load("test.rfg")

project.test.hello()
project.test[0].hello()
project.test[1].hello()
```

Stage 09 established persistent live Role objects exposed through Project.

Before beginning packaging and publishing work, the repository should now be cleaned up so that production code and test code have clear physical boundaries.

---

## First: Inspect the Current Repository

Before modifying anything:

1. Inspect the current repository structure.
2. Read the current AI documentation under `../../../../ai`.
3. Read the current implementation, especially:
    - Core pipeline
    - Registry
    - Dispatcher
    - Runtime
    - Handoff
    - Bridges
    - Python API
    - live Role implementation
4. Locate all existing Rust test code.
5. Locate all Python-facing tests.
6. Identify test helpers/fixtures that exist only for testing.
7. Preserve all currently valid tests unless there is a strong technical reason not to.

Do not assume the repository structure from historical prompts.

The current implementation and current documentation are the source of truth.

---

# 1. Separate Rust Tests from Production Source Files

Production Rust source files should contain production implementation, not large inline test modules.

If a production module currently contains code such as:

```rust
#[cfg(test)]
mod tests {
    // many tests...
}
```

move the actual test implementation into a dedicated test file associated with that subsystem.

Prefer structures such as:

```text
subsystem/
├── mod.rs
└── tests.rs
```

or the equivalent structure appropriate for the existing module layout.

The production module should contain only the minimal test-module declaration:

```rust
#[cfg(test)]
mod tests;
```

The actual tests belong in `tests.rs`.

Do not move internal Core unit tests into Cargo's root-level `tests/` directory merely for organizational consistency.

Many RoleForge Core components intentionally use internal/private or `pub(crate)` APIs.

Do not make internal Core APIs public just so external integration tests can access them.

The desired principle is:

> Test code may live beside the subsystem it tests, but it must be physically separated from production implementation code.

---

# 2. Preserve Internal Test Access

When moving Rust tests, preserve their ability to test internal implementation details where appropriate.

Do not weaken encapsulation.

Specifically:

- Do not change `pub(crate)` to `pub` only for tests.
- Do not expose new public APIs only for tests.
- Do not move Core implementation details into the Python API.
- Do not alter architectural boundaries to make test organization easier.

Test organization must adapt to the architecture, not the other way around.

---

# 3. External Python Test Project

The current directory:

```text
anyone_py_project/
```

acts as a simulated consumer of RoleForge.

Rename it to:

```text
external_test_project/
```

The name should explicitly communicate that this is not part of the RoleForge library itself.

Its purpose is:

> Simulate a completely separate Python project that consumes RoleForge as an external library.

Update current references throughout the repository to use the new directory name where appropriate.

This includes:

- current documentation
- test commands
- examples
- active paths
- comments
- development instructions

Do not leave stale active references to `anyone_py_project`.

---

# 4. Responsibilities of the External Test Project

`../../../../../external_test_project` is for Python-facing / consumer-facing testing.

It may contain:

- `.rfg` test/example files
- Python API tests
- example usage
- `main.py`
- tests of `load()`
- tests of `Project`
- tests of live Role objects
- tests of user-visible behavior
- fixtures needed to simulate real RoleForge usage

It should represent the perspective of a Python developer using RoleForge.

It is NOT part of the RoleForge runtime implementation.

Internal Rust/Core unit tests do not belong here.

---

# 5. Production Code Boundary

The `../../..` directory should contain RoleForge implementation and required runtime resources.

Test implementation must not be mixed unnecessarily into production implementation files.

Internal Rust `tests.rs` files may remain physically beside the subsystem they test because they are compile-time test modules and are not part of the normal library build.

The desired structure is conceptually:

```text
RoleForge/
├── roleforge/                 # RoleForge implementation
│   └── core/
│       └── ...
│           ├── mod.rs         # production implementation
│           └── tests.rs       # tests for that subsystem
│
├── docs/
│
├── external_test_project/     # simulated external Python consumer
│   ├── main.py
│   ├── test.rfg
│   └── ...
│
├── Cargo.toml
├── Cargo.lock
└── pyproject.toml
```

The exact internal layout should follow the existing architecture rather than forcing unnecessary directory changes.

Do not remove useful tests merely to make the source tree visually smaller.

---

# 6. No Behavioral Changes

This stage must not change RoleForge behavior.

In particular, preserve:

- Loader behavior
- Main Tokenizer behavior
- global Role indexes
- Role-local indexes
- Registry resolution
- Unknown Role behavior
- conflict preflight behavior
- Dispatcher behavior
- Runtime orchestration
- Handoff behavior
- Bridge resolution
- Python Bridge behavior
- `RoleInput`
- `roleforge_receive`
- live Role object lifetime
- `project.roles`
- `project.<role>`
- `project.<role>[index]`
- `project.get_role(...)`
- existing error behavior
- current receiver-return semantics
- the rule that `load()` does not automatically call `start()`

This stage is organizational, not architectural.

---

# 7. Do Not Implement Publishing Yet

Stage 10 prepares the repository for publishing.

Do NOT yet:

- publish to PyPI
- create PyPI accounts or tokens
- implement GitHub Actions publishing
- redesign `../../../../../pyproject.toml`
- redesign `../../../../../Cargo.toml`
- solve wheel distribution
- implement cross-platform builds
- implement release automation
- change package versioning strategy
- implement `start()`
- add new Bridges
- add Role installation/removal
- add new RoleForge utilities
- redesign the Error/Console system
- add unrelated features

Those belong to later stages.

---

# 8. Keep Historical Prompts Historical

Files under the prompt history are development records.

Do not rewrite old stage prompts merely because paths or architecture have evolved.

If a historical prompt mentions `anyone_py_project`, leave it unchanged if changing it would rewrite history.

Current documentation, however, should use the new current name.

Follow the existing repository rule:

> Historical prompts do not override the current implementation or current documentation.

---

# 9. Verification

After the refactor, run the relevant existing test suites.

At minimum verify:

```bash
cargo test
cargo build
cargo fmt --check
```

Also run the Python-facing tests from the renamed external test project using the appropriate current development setup.

Verify that:

1. Existing Rust tests still pass.
2. Existing Python-facing tests still pass.
3. No production behavior changed.
4. No internal API was made public solely for testing.
5. `anyone_py_project/` no longer exists as the active external project directory.
6. Current documentation does not contain stale active references to the old directory name.
7. Historical references remain untouched where appropriate.
8. The example live Role flow still works.

---

# 10. Documentation

Update current documentation only where necessary to reflect:

- the renamed external test project
- its explicit purpose as a simulated external consumer
- the separation between internal Rust tests and external Python-facing tests

Add a short README inside:

```text
external_test_project/
```

explaining that this directory exists to simulate a separate Python project consuming RoleForge.

Keep it concise.

---

# 11. Final Report

After completing the stage, report:

1. Which Rust files contained inline tests.
2. Where each test module was moved.
3. Whether any tests were removed, and why.
4. The final external test project name.
5. Every current reference updated from `anyone_py_project`.
6. Any historical references intentionally left unchanged.
7. Test/build results.
8. Any problems discovered during the cleanup.
9. Confirmation that no RoleForge runtime behavior was intentionally changed.

Do not commit or push unless explicitly asked.

Respond to me in Hebrew.