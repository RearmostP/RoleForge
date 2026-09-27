# Stage 12 — Diagnostic & Error Inventory

## Goal

Inspect the current RoleForge codebase and create a complete inventory of all places where the system currently:

- detects an error;
- catches or converts an error;
- returns an error;
- raises a Python exception;
- decides that execution cannot continue;
- detects a non-fatal abnormal condition;
- detects a conflict;
- detects an unknown or unresolved item;
- emits a warning-like condition;
- emits informational runtime output;
- emits debug output;
- intentionally ignores or suppresses a failure;
- performs fallback behavior after something unexpected happens.

This stage is **analysis only**.

Do NOT implement an Error Manager, Event Manager, Console Manager, diagnostic system, or new error types.

Do NOT change existing runtime behavior.

The purpose of this stage is to understand what RoleForge already needs before designing the centralized diagnostic/event system.

---

## Core Idea

Do not search only for traditional errors.

RoleForge can encounter reportable conditions that are not necessarily fatal errors.

For example:

```text
File does not exist
→ likely a fatal error

Unknown Role
→ reportable condition
→ may or may not be fatal

Role registered in both registries
→ conflict
→ reportable condition
→ currently affects dispatch/handoff behavior

Debug information
→ diagnostic event
→ not an error

Useful runtime information
→ informational event
```

Do NOT assign final severity classifications during this stage unless the current implementation already explicitly defines one.

In particular, do not decide yet that:

```text
UnknownRole = Warning
RegistryConflict = Error
```

Those decisions belong to the future diagnostic system design.

For now, describe what the condition means and what the current code actually does.

---

# 1. Inspect the Entire Active Codebase

Inspect the current active RoleForge implementation, including at minimum:

```text
src/
python/roleforge/
external_test_project/
```

Also inspect current architecture documentation when necessary to understand intended behavior.

Historical prompts and old implementation reports may provide historical context, but they are not the current implementation.

The current code and current architecture documentation are the source of truth.

---

# 2. Find Traditional Error Handling

Find every meaningful place that creates, propagates, converts, catches, or exposes an error.

Search for mechanisms such as, but not limited to:

```rust
Result
Err(...)
?
map_err
ok_or
ok_or_else
unwrap
expect
panic!
PyErr
PyResult
```

Also inspect relevant error enums, structs, conversion implementations, and boundary conversions.

For Python, inspect mechanisms such as:

```python
raise
try
except
assert
```

Do not rely only on keyword search.

Follow the execution paths and identify where failures originate and where they are handled.

---

# 3. Find Reportable Non-Fatal Conditions

Look for conditions that RoleForge detects which are important enough to potentially report, even if they are not represented as traditional errors.

Examples include:

- unknown Roles;
- unresolved Roles;
- Registry conflicts;
- duplicate/conflicting registrations;
- skipped Roles;
- rejected dispatches;
- preflight failures;
- invalid Bridge targets;
- unsupported or missing receivers;
- ambiguous public API access;
- missing Role instances;
- invalid indexes;
- fallback behavior;
- ignored receiver return values where relevant;
- conditions that currently cause a Role to be skipped while other processing could theoretically continue.

Do not assume these are warnings or errors.

Record their current behavior.

---

# 4. Registry Conflict Logic

Pay particular attention to the existing Registry conflict behavior.

The architectural rule is:

> A Role name registered in exactly one registry can be resolved.  
> A Role name registered in neither registry is unknown.  
> A Role name registered in both registries is a conflict and must never be resolved by precedence.

Locate exactly where this condition is detected.

Document:

- which component detects it;
- what data is available when it is detected;
- what value/type represents the conflict;
- where that result goes next;
- whether anything is printed;
- whether execution stops;
- whether only that Role stops;
- whether the entire Handoff/load operation stops;
- whether the condition is converted into another error later.

Do not change the behavior.

---

# 5. Unknown Role Logic

Locate exactly where an unknown/unregistered Role is detected.

Document:

- which component detects it;
- what information is available;
- how it is represented;
- what happens to the affected Role;
- whether other Roles continue;
- whether anything is printed;
- whether it becomes an exception;
- whether it reaches Python;
- whether it is currently treated as fatal or non-fatal.

Do not decide its future severity yet.

---

# 6. Loader and File Errors

Inspect Loader behavior for conditions such as:

- file not found;
- invalid path;
- unreadable file;
- filesystem errors;
- invalid input;
- encoding/read failures if applicable.

For each condition, document where it originates and how far it propagates.

---

# 7. Tokenizer / Source Errors

Inspect the Main Tokenizer and related source processing.

Find all conditions where source text is rejected or considered invalid.

Examples may include:

- text outside a Role;
- invalid Role declaration;
- indentation before `@role`;
- comments outside allowed contexts;
- malformed declaration;
- empty/missing Role name;
- any other syntax validation currently implemented.

Use the actual implementation as the source of truth.

Do not invent errors that do not currently exist.

---

# 8. Dispatcher / Runtime / Handoff

Inspect:

- Dispatcher;
- Core Runtime;
- Handoff;
- preflight logic.

Identify every place where a condition causes:

- dispatch rejection;
- Handoff rejection;
- operation abortion;
- Role skipping;
- conversion into another error;
- early return.

Document the exact current control-flow effect.

---

# 9. Bridge Errors

Inspect the Bridge abstraction and the current Python Bridge.

Find all failures related to:

- Bridge lookup/resolution;
- invalid Bridge identifiers;
- target/module loading;
- conversion from neutral Core data to Python;
- Python object creation;
- Role class validation;
- `roleforge_receive`;
- Python exceptions raised inside the receiver;
- missing receiver;
- invalid receiver;
- invalid Role class;
- Role class not inheriting from the required base;
- any PyO3 conversion failure.

For each one, identify whether the Bridge:

- creates the error;
- converts another error;
- propagates a Python error;
- maps it into a Core error;
- prints anything directly.

---

# 10. Python Public API Errors

Inspect:

```text
python/roleforge/__init__.py
python/roleforge/_live.py
```

and the Rust Python API boundary.

Map current public-facing failures such as:

- unknown Role requested through `get_role`;
- missing local Role index;
- invalid index type;
- ambiguous dynamic attribute;
- invalid attribute access;
- unsupported slice access;
- errors propagated from `load()`;
- any `KeyError`;
- `IndexError`;
- `AttributeError`;
- `TypeError`;
- other Python-visible exceptions.

Document what the Python user currently sees.

---

# 11. Direct Output

Find every place in active production code that directly outputs something.

Search for mechanisms such as:

```rust
println!
eprintln!
dbg!
```

and:

```python
print(...)
```

Classify the purpose of each output descriptively:

```text
debug information
temporary diagnostic
user-facing information
error presentation
warning-like presentation
Test Role output
```

Do NOT redesign or remove these outputs yet.

The future architecture follows:

> **Components report structured events/errors; they do not present them.**

and:

> **All console output goes through a dedicated console/output layer.**

This stage should identify the current places that violate or predate those future rules.

---

# 12. Panic / Unsafe Failure Paths

Find any production use of:

```rust
unwrap()
expect(...)
panic!(...)
unreachable!(...)
todo!(...)
unimplemented!(...)
```

For each occurrence, determine whether it can be reached through normal user/runtime input.

Do not automatically classify every `unwrap()` as a bug.

Explain its context.

Test-only occurrences should be listed separately or excluded from the main runtime inventory.

---

# 13. Ignored / Suppressed Conditions

Look for places where RoleForge intentionally or accidentally discards information.

Examples:

```text
Result ignored
return value ignored
error converted to None
fallback without reporting
failed lookup silently skipped
receiver return ignored
```

Important:

The current Python receiver return value being ignored is an intentional protocol behavior, not automatically an error.

Still mention it if useful for understanding the boundary, but clearly distinguish intentional protocol behavior from suppressed failures.

---

# 14. Produce an Inventory

Create a concise inventory table.

Suggested format:

| ID | Component | Location | Condition | Current Representation | Current Effect | Current Output | Reaches User? |
|---|---|---|---|---|---|---|---|
| D001 | Loader | ... | File missing | ... | load stops | ... | Yes |
| D002 | Registry | ... | Unknown Role | ... | ... | ... | ... |
| D003 | Registry/Handoff | ... | Registry conflict | ... | ... | ... | ... |

Use stable temporary IDs such as:

```text
D001
D002
D003
...
```

These IDs are only for discussing the inventory.

They are NOT proposed permanent error codes.

---

# 15. Group the Findings

After the raw inventory, group the findings by their **nature**, without assigning final severity.

Suggested groups:

```text
I/O / File conditions
Source / Syntax conditions
Registry / Resolution conditions
Runtime / Pipeline conditions
Bridge / Interop conditions
Role Contract conditions
Python API conditions
Debug / Informational output
Internal invariant failures
```

If the actual code suggests better groups, explain them.

Do not force every item into an inappropriate category.

---

# 16. Identify Detection vs Presentation

For every important finding, distinguish between:

```text
Detection
    Where RoleForge discovers that something happened.

Propagation
    How that information moves through the system.

Policy
    Where the system decides whether processing continues.

Presentation
    Where/how the user is told about it.
```

These responsibilities may currently be mixed together.

Identify where that happens.

Do NOT separate them in code during this stage.

---

# 17. Identify Duplicate Patterns

Look for repeated error/event handling patterns.

For example:

- several components manually create similar strings;
- several components directly print diagnostic information;
- multiple layers independently convert failures;
- repeated Python exception construction;
- repeated early-return patterns;
- repeated source-location information.

Document these patterns because they may indicate what the future centralized system should provide.

Do not refactor them yet.

---

# 18. Do Not Design the Final System Yet

Do NOT introduce:

- Error Manager;
- Event Manager;
- Diagnostic Manager;
- Console Manager;
- severity enum;
- error codes;
- warning codes;
- event bus;
- logging framework;
- global error state;
- new exception hierarchy;
- new public API.

Do not rename existing errors merely to make the inventory cleaner.

This stage exists specifically so the next design stage can be based on real requirements rather than assumptions.

---

# 19. Final Analysis

At the end, provide:

1. Total number of meaningful diagnostic/error/event detection points found.
2. The complete inventory table.
3. A short explanation of each important item.
4. Which conditions currently stop the entire operation.
5. Which conditions currently stop only one Role or one stage.
6. Which conditions currently allow processing to continue.
7. Which conditions are currently directly printed.
8. Which conditions become Python exceptions.
9. Which conditions are currently silent.
10. Which places mix detection with presentation.
11. Which places mix detection with continuation policy.
12. Repeated patterns that a future centralized system could unify.
13. Any places where current behavior is unclear or inconsistent.

Finally, provide a small conceptual map like:

```text
Current system

Component
    ↓ detects
Condition
    ↓
Current representation
    ↓
Current propagation
    ↓
Current continuation behavior
    ↓
Current presentation/user result
```

Do NOT propose the final Error/Event architecture yet.

The purpose of the report is to give us enough information to design that architecture ourselves in the next stage.

---

## Important Architectural Context

Keep these existing RoleForge principles in mind while analyzing:

> **Components process. Runtime orchestrates.**

> **Core Runtime orchestrates the Core pipeline, never Role behavior.**

> **The Core resolves and delivers. The Role decides what happens next.**

> **Core knows the protocol, never the roles.**

> **Components report structured events/errors; they do not present them.**

> **All console output goes through a dedicated console/output layer.**

The last two rules describe the intended future direction.

This stage should reveal what needs to change later to satisfy them cleanly.

Do not implement those changes now.

Respond to me in Hebrew.