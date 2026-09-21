# RoleForge — First Test

Create the first intentionally simple real Role used to manually test the RoleForge pipeline that has been built so far.

This is NOT a new numbered development stage.

Do not treat this as Stage 09.

The purpose is simple:

> Prove that the system we built can take a real `.rfg` file, discover a Role, resolve it through the Registry, pass through Handoff and the Python Bridge, and successfully call the Role's `roleforge_receive(role)` function.

Keep this test intentionally simple.

Do not add new architecture merely to make the test more impressive.

---

# Goal

We want to manually test this complete path:

```text
Python user
    ↓
load("test.rfg")
    ↓
Rust Core Runtime
    ↓
Loader
    ↓
Main Tokenizer
    ↓
Registry
    ↓
Dispatcher
    ↓
Handoff preflight
    ↓
RoleInput
    ↓
Bridge Resolver
    ↓
Python Bridge
    ↓
Test Role
    ↓
roleforge_receive(role)
```

The important result is:

> A real Python Role receives the real RoleInput produced from a real `.rfg` file.

This should be visible when running the example manually.

---

# Use a Dynamic Test Role

Create a very small dynamic Role for this test.

Use a clear name such as:

```text
Test
```

Place it in the normal dynamic Role area according to the current project structure.

Conceptually:

```text
roleforge/
└── roles/
    └── Test/
        └── main.py
```

Use the actual repository conventions if capitalization or exact placement differs.

Do not create a built-in Role for this test.

The purpose is to test a realistic dynamic Role path.

---

# Test Role Implementation

The Role should contain only the mandatory RoleForge receiving entry point:

```python
def roleforge_receive(role):
    ...
```

Inside it, print the received information clearly.

At minimum print:

```text
name
index
role_index
body
source.declaration_line
```

For example, the output may conceptually look like:

```text
=== ROLEFORGE TEST ROLE ===

Role received successfully!

name: Test
global index: 0
role index: 0
body: '...'
declaration line: 1
```

The exact formatting is not architecturally important.

Keep it readable for a human running the test manually.

---

# Important: Do Not Add Role Logic

This Role is NOT intended to demonstrate a real DSL.

Do not add:

- Role tokenizer
- Role parser
- validation system
- actions
- `start()`
- classes
- configuration system
- persistent state
- dependencies
- advanced API
- new Core behavior

The Role exists only to prove receipt.

Its entire meaningful behavior should be:

```text
receive RoleInput
    ↓
print what was received
```

---

# Test .rfg File

Create a small `.rfg` file for the manual test.

Conceptually:

```text
@role Test

hello = world
number = 123
```

The body is intentionally opaque to the Core.

The Test Role does not need to parse it.

It only needs to print the body exactly as it was received after the existing Core processing rules.

---

# Dynamic Registry Entry

Register the Test Role in the dynamic Role Registry using the current Stage 08 format:

```json
{
  "Test": {
    "entry": {
      "via": "python",
      "target": "Test/main.py"
    }
  }
}
```

Use the exact path required by the repository's current dynamic Role path-resolution rules.

Do not add language metadata.

Do not infer Python from `.py`.

The Bridge is selected explicitly through:

```text
via = "python"
```

---

# Manual Python Test

Create or update a small user-side example that allows us to run the test manually.

Conceptually:

```python
from roleforge import load

project = load("test.rfg")
```

Keep it simple.

The point is that running this file should cause the real Test Role's:

```python
roleforge_receive(role)
```

to execute.

Do not manually import the Test Role.

Do not manually call `roleforge_receive`.

Do not bypass RoleForge.

The Test Role must be reached through the actual RoleForge pipeline.

---

# What We Are Proving

This test should prove that all of these existing pieces work together:

```text
Loader
✓

Main Tokenizer
✓

Role discovery
✓

global index
✓

Role-local index
✓

source metadata
✓

Dynamic Registry
✓

entry.via
✓

entry.target
✓

Dispatcher
✓

Handoff preflight
✓

RoleInput creation
✓

Bridge resolution
✓

Python Bridge
✓

Python target loading
✓

roleforge_receive(role)
✓
```

Do not fake any of these steps.

---

# Verify the Received Data

When the Test Role receives the RoleInput, verify manually and/or with a small assertion that:

```text
role.name == "Test"
role.index == 0
role.role_index == 0
role.source.declaration_line == 1
```

Also verify that:

```text
role.body
```

contains the expected body from the `.rfg` file.

Do not recalculate these values inside the Test Role.

They must be the values delivered by RoleForge.

---

# Test Multiple Instances Too

After the basic single Role test works, extend the same `.rfg` test slightly to verify multiple instances of the same Role.

For example:

```text
@role Test

hello = first

@role Test

hello = second
```

The Test Role should be called twice.

Expected identity:

```text
first Test:
    global index = 0
    role_index = 0

second Test:
    global index = 1
    role_index = 1
```

Both calls should use the same registered Test Role implementation.

This verifies that multiple Role instances are delivered independently through the same Bridge and target.

Keep this as part of the same simple first test.

Do not build a separate framework for it.

---

# Do Not Call start()

Do not add or call:

```python
start()
```

This test is specifically testing the receiving handoff:

```python
roleforge_receive(role)
```

The existing architectural separation remains:

```text
roleforge_receive
    = Core → Role delivery

start
    = future Role-level operation
```

Do not mix them.

---

# Do Not Change the Core Unless There Is a Real Bug

This is primarily a usage/integration test.

The Core should already support this flow after Stage 08.

Do not refactor or redesign the Core merely while creating this test.

If the test exposes an actual bug in the current implementation:

1. Stop and identify the bug clearly.
2. Explain what existing architectural contract is being violated.
3. Make only the smallest justified fix if it is clearly within the already-approved architecture.
4. Report the fix explicitly.

Do not invent new architecture to solve a test setup problem.

---

# Existing Tests

Do not replace the existing automated Stage 08 tests.

This manual Role is additional proof that the system works from a developer/user perspective.

After adding the first test:

1. Run the manual example.
2. Show the actual output.
3. Run the complete Rust test suite.
4. Run the complete Python test suite.
5. Report the exact results.

---

# Documentation

This test should not require a major Iron Rules update.

It should exercise architecture that is already documented.

Only update live architectural documentation if the test reveals that the current documentation is factually inconsistent with the already-approved implementation.

Do not add the Test Role itself as an architectural concept.

It is only a development/testing Role.

Do not modify historical Stage prompts.

---

# Keep It Fun and Small

This is the first real RoleForge Role created primarily to watch the full system work.

Do not overengineer it.

We want to be able to run one simple Python file and visibly see:

```text
RoleForge found the Role
        ↓
RoleForge routed it
        ↓
Python Bridge loaded it
        ↓
roleforge_receive(role) was called
        ↓
the Role printed the exact data it received
```

That is the success condition.

---

# Before Finishing

Report:

1. Where the Test Role was created.
2. The dynamic Registry entry used for it.
3. The `.rfg` test file contents.
4. The Python file used to call `load()`.
5. The Test Role's `roleforge_receive(role)` implementation.
6. The actual output from the single-instance test.
7. The actual output from the multiple-instance test.
8. Whether the received indexes and source metadata were correct.
9. The complete Rust test result.
10. The complete Python test result.
11. Any warnings or errors encountered.
12. Any Core changes that were necessary and exactly why.
13. Every file created, modified, removed, or renamed.

Do not commit or push unless explicitly asked.

Save this prompt in the existing prompt-history location as:

```text
first_test.md
```

This is intentionally not numbered.

Respond to me in Hebrew.