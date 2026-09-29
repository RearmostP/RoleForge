# RoleForge — Stage 03: Role Registry and Dispatcher

Implement Stage 03 of the RoleForge Core.

Before implementing Stage 03, make one small addition to the existing Stage 02 output: preserve the source declaration line of every discovered Role.

Keep the implementation small, isolated, and consistent with the existing RoleForge architecture and coding style.

Do not redesign Stage 01 or Stage 02.

---

# Core architectural rules

RoleForge is a framework for multiple independent Role-specific DSLs.

The Core understands only the outer RoleForge protocol.

It must never understand the grammar, behavior, actions, or implementation language of an individual Role.

The architectural principle is:

> Core knows the protocol, never the roles.

And:

> Components process. Runtime orchestrates.

Major components must not directly drive unrelated pipeline components.

The Registry does not dispatch.

The Dispatcher does not parse Role-specific syntax.

A Role is responsible for everything that happens after the Core hands the Role its clean content.

---

# Part 1 — Preserve Role source metadata

The current Main Tokenizer already discovers the exact source line on which every `@role` declaration occurs.

Currently that information is used for tokenizer errors but is discarded for successfully discovered Roles.

Preserve it.

Every `CleanRole` must contain source metadata identifying the line on which its `@role` declaration appeared.

For example:

    # config

    @role Directory
    folder src

    @role Database
    host localhost

The resulting Roles should conceptually contain:

    Directory:
        index = 1
        declaration_line = 3

    Database:
        index = 2
        declaration_line = 6

The line numbering remains 1-based, consistent with the existing tokenizer errors.

---

## Metadata design

Do not build a large source-mapping system.

We only need the declaration line now.

However, keep source information conceptually separated so it can be extended later without filling `CleanRole` with unrelated fields.

A small neutral metadata structure is preferred, conceptually similar to:

    SourceInfo {
        declaration_line: usize,
    }

and:

    CleanRole {
        index,
        name,
        body,
        source,
    }

The exact Rust naming may be adjusted if there is a clearly better name consistent with the existing project.

Do not add speculative metadata such as columns, byte spans, file IDs, or body line maps yet.

Only preserve information that is currently useful.

The declaration line means the line containing:

    @role RoleName

not the first line of the Role body.

---

# Part 2 — Internal Role storage layout

Stage 03 introduces the internal metadata required to locate Roles.

RoleForge has two conceptual categories of Roles:

1. Built-in Roles shipped with the library.
2. Dynamic Roles installed or registered by the user.

Keep their metadata separate.

The intended conceptual layout is:

    roleforge/
    ├── core/
    │   ├── ...
    │   └── storage/
    │       ├── builtin_roles.json
    │       └── dynamic_roles.json
    │
    ├── builtin_roles/
    │   └── ...
    │
    └── roles/
        └── ...

`builtin_roles/` is the location for Roles shipped with RoleForge.

`roles/` is the recommended/default location for user-installed dynamic Roles.

A dynamic Role is NOT required to physically exist inside `roles/`.

It may use an absolute external path.

Do not make the recommended directory mandatory.

---

# Part 3 — Registry JSON format

Keep the registry format intentionally minimal.

The Core only needs to know:

> Role name → Role entry location

Do not store the Role implementation language.

Do not add fields such as:

    language
    type
    runtime
    parser
    tokenizer
    actions

The implementation language is not part of the Core Role contract.

The principle is:

> The Core resolves destinations, not implementations.

A registry should conceptually be able to contain:

    {
      "MyRole": {
        "entry": "MyRole/entry"
      },

      "ExternalRole": {
        "entry": "D:/Development/ExternalRole/entry"
      }
    }

The Role name may be represented as the JSON object key as shown above.

Do not duplicate the name inside the object unless there is a real technical reason.

All entry locations are JSON strings.

---

# Part 4 — Relative and absolute paths

Do not require the JSON to explicitly say whether a path is relative or absolute.

Do NOT introduce structures such as:

    {
        "relative_path": "..."
    }

or:

    {
        "absolute_path": "..."
    }

Rust can determine this from the path itself.

Use Rust path facilities rather than manual string heuristics.

---

## Dynamic Roles

For `dynamic_roles.json`:

A relative entry path is resolved relative to the recommended dynamic Role directory:

    roleforge/roles/

Example:

    "entry": "MyRole/entry"

conceptually resolves to:

    roleforge/roles/MyRole/entry

An absolute path is used as-is.

Example:

    "entry": "D:/Development/MyRole/entry"

must remain that absolute destination.

---

## Built-in Roles

Apply the same concept to built-in Roles.

A relative entry from:

    builtin_roles.json

is resolved relative to:

    roleforge/builtin_roles/

An absolute path, if one exists, is used as-is.

---

## Important path rule

Registry resolution must NOT depend on the Python process current working directory.

Changing the application's current working directory must not silently change where registered Roles resolve.

Use stable RoleForge-owned base locations.

A future public API may allow changing the default relative Role directory, for example conceptually:

    RoleForge.set_relative_path(...)

but this is ONLY a future idea.

Do NOT implement that API in Stage 03.

---

# Part 5 — Registry responsibility

Implement a small Role Registry component.

Its responsibility is metadata lookup and path resolution.

Conceptually, it needs to answer questions such as:

    Does this Role exist?

and:

    What is the resolved entry location for this Role?

Possible conceptual operations are:

    exists(name)
    get_entry(name)

The exact internal Rust API should follow idiomatic Rust and the current project structure.

Do not mechanically implement these exact function names if a simpler or cleaner interface makes more sense.

The Registry may load/read the built-in and dynamic registry metadata required for these operations.

Avoid unnecessary repeated filesystem reads where a simple local design can avoid them.

Follow the project optimization principle:

> Optimize obvious waste immediately. Postpone architectural optimization.

Do not introduce complex caching systems.

---

# Part 6 — Built-in and dynamic lookup

Both built-in and dynamic Roles must be discoverable through the Registry.

Keep their persistent JSON files separate.

Do not merge the files just for convenience.

If lookup precedence becomes necessary because the same Role name exists in both registries, do NOT invent a major policy silently.

Prefer a small explicit implementation decision if technically required, and report the issue in the final summary.

Do not introduce a plugin dependency system.

Do not implement installation/removal APIs yet unless strictly required by Stage 03 internals.

---

# Part 7 — Dispatcher

Implement the Core Dispatcher.

Input:

    Vec<CleanRole>

Conceptually, for each `CleanRole` in source order:

    CleanRole
        ↓
    Dispatcher
        ↓
    Registry lookup
        ↓
    Role exists?
       /       \
     yes       no
      |         |
    resolve   structured
    entry     unknown-role result
      |
    handoff destination

The Dispatcher is responsible for routing.

The Registry is responsible only for resolving metadata.

Do not put dispatch behavior inside the Registry.

---

# Part 8 — Unknown Roles

An unknown Role must NOT crash or stop processing the remaining Roles.

Example:

    @role Directory
    ...

    @role NotInstalled
    ...

    @role Database
    ...

Conceptually:

    Directory     → resolved/dispatched
    NotInstalled  → reported as unknown and skipped
    Database      → still processed

Unknown Role information must remain structured.

Do NOT write:

    println!(...)

inside the Dispatcher.

Do not directly format user-facing console messages there.

Do not implement console presentation inside the Registry either.

The future architecture will have dedicated error/event handling and console/output management.

The principle is:

> Components report structured events/errors; they do not present them.

And:

> All console output goes through a dedicated console/output layer.

For Stage 03, return or expose enough structured information for a future Error Manager to handle an unknown Role.

The structured information should preserve useful context such as:

- Role name
- Role index
- source declaration line

Do not build the complete Error Manager in this stage.

Do not build the Console Manager in this stage.

Do not decide yet whether an unknown Role is ultimately displayed as an error, warning, or another presentation category.

That policy belongs to the later error/output architecture.

---

# Part 9 — Role handoff boundary

The Core must not inspect Role-specific syntax.

After a Role has been resolved, the Core's responsibility is routing the `CleanRole` to its registered destination.

The Core must not care whether the Role implementation is written in:

- Rust
- Python
- another supported implementation mechanism in the future

Do not add a `"language"` field to solve routing.

Implementation language is deliberately outside the Registry contract.

If actually invoking arbitrary language-independent Role entry files requires a runtime/plugin-loading mechanism that does not yet exist, do NOT invent one in this stage.

Instead, implement the routing/resolution boundary cleanly and stop at the point where a later execution/handoff mechanism is genuinely required.

Report that boundary clearly.

Do not fake execution.

---

# Part 10 — No Main Parser

Do NOT add a Main Parser.

The previous architecture considered:

    Loader
      ↓
    Main Tokenizer
      ↓
    Main Parser

That design has now changed.

The Main Tokenizer already produces the complete neutral Core representation:

    Vec<CleanRole>

containing:

    index
    name
    body
    source metadata

There is currently no additional Core-level semantic structure for a Main Parser to create.

Therefore:

> There is no Main Parser in the current Core architecture.

Do not add an empty/pass-through Parser merely to preserve an older architecture.

Each individual Role may later have its own tokenizer/parser if its DSL requires one.

For example:

    Core Main Tokenizer
            ↓
        CleanRole
            ↓
        Directory Role
            ↓
    Directory Tokenizer
            ↓
    Directory Parser

That Role-specific parsing is outside Stage 03.

---

# Part 11 — Component boundaries

Keep dependencies clean.

Do not create a chain where major components automatically invoke the next major component.

For example, do NOT change the Loader so it invokes the Tokenizer.

Do NOT change the Tokenizer so it invokes the Dispatcher.

The eventual Runtime/Orchestrator owns execution order.

The rule remains:

> Components process. Runtime orchestrates.

Stage 03 may expose clean functions/types that a future Runtime can compose.

Do not implement the complete Runtime unless absolutely necessary for isolated Stage 03 tests.

---

# Part 12 — Input / Output headers

Continue the existing RoleForge source-file convention.

Every new Rust source file should begin with a concise Input/Output comment describing its responsibility.

For example:

    // Input: Clean Roles and Role registry metadata.
    // Output: Resolved Role destinations and structured unresolved Role information.

Keep these comments concise and accurate.

Do not write long documentation essays at the top of source files.

---

# Part 13 — Tests

Add focused tests for the new behavior.

At minimum test:

## Source metadata

Verify that discovered Roles retain the correct 1-based declaration line.

Include comments and blank lines before/between Roles.

Example concept:

    # heading

    @role First
    body


    # comment
    @role Second
    body

Verify the declaration lines exactly.

---

## Registry lookup

Verify that a registered Role can be found.

Verify that an unknown Role is not falsely resolved.

---

## Dynamic relative path

Verify that a relative dynamic entry is resolved from the default dynamic Role directory.

---

## Built-in relative path

Verify that a relative built-in entry is resolved from the built-in Role directory.

---

## Absolute path

Verify that an absolute entry path is not prefixed with the RoleForge Role directories.

Keep platform differences in mind when testing paths.

Avoid brittle Windows-only assumptions if the test can be written portably.

---

## Dispatcher preserves source order

Given multiple `CleanRole`s, process them in their original order.

Do not reorder by Role name or registry source.

---

## Unknown Role does not stop processing

Test a sequence conceptually equivalent to:

    Existing
    Missing
    ExistingAgain

The missing Role must produce structured unresolved information while the later Role is still processed/resolved.

---

## Unknown Role source information

Verify that the structured unknown-Role result preserves enough context to identify:

- its name
- its positional index
- its declaration line

---

# Part 14 — Do not overbuild Stage 03

Do NOT implement:

- Main Parser
- Role-specific tokenizers
- Role-specific parsers
- full Runtime/Orchestrator
- Python bindings
- public Python registration API
- `set_relative_path`
- Role installation UI
- dependency management
- Role versioning
- Role action discovery
- implementation-language detection
- full Error Manager
- Console Manager
- logging framework
- complex caching
- concurrency
- plugin sandboxing

Only implement what Stage 03 actually needs.

---

# Expected architecture after Stage 03

Conceptually, the Core should now have enough pieces for:

    .rfg file
        ↓
    Loader
        ↓
    LoadedFile
        ↓
    Main Tokenizer
        ↓
    Vec<CleanRole>
        ↓
    [future Runtime/Orchestrator]
        ↓
    Dispatcher
        ↓
    Registry
        ↓
    resolved Role destination
        OR
    structured unknown-Role information

And after the Core handoff:

    Role
      ↓
    Role-specific processing

The Role-specific processing is not part of this stage.

---

# Final review

Before finishing:

1. Run the relevant Rust tests.
2. Check that existing Stage 01 and Stage 02 tests still pass.
3. Make sure no Main Parser was introduced.
4. Make sure the Registry contains no implementation-language metadata.
5. Make sure the Dispatcher does not print directly to the console.
6. Make sure unknown Roles do not prevent later Roles from being processed.
7. Make sure Role declaration source lines survive the Main Tokenizer.
8. Make sure relative path resolution does not depend on the process current working directory.
9. Keep the implementation local, simple, and idiomatic.
10. Do not modify historical prompt files to retroactively rewrite earlier development history.

After implementation, summarize:

- files created
- files modified
- the final Registry JSON structure
- how built-in and dynamic paths are resolved
- how source declaration lines are preserved
- how Dispatcher results represent resolved and unresolved Roles
- what exact boundary remains before actual Role execution/handoff can occur
- test results
- any design question that genuinely could not be resolved without making a new architectural decision

Respond to me in Hebrew.