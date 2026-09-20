# Task — Rebuild the RoleForge Core Iron Rules

Create a new complete `ROLEFORGE_CORE_IRON_RULES.md` document for the RoleForge project.

The previous Iron Rules document was accidentally lost/overwritten, so this is NOT a request to patch or incrementally edit the old document.

You must rebuild the document from the architectural decisions described below.

The resulting document will primarily be used as a strict architectural reference for AI coding agents such as Codex while they modify RoleForge.

It is NOT intended to be the final human-facing documentation.

Therefore:

- Do NOT compress the document merely to make it shorter.
- Do NOT optimize for minimum token count.
- Preserve the full architectural reasoning and boundaries.
- Be explicit about responsibilities and forbidden behavior.
- Repetition is acceptable when it prevents architectural misinterpretation.
- Include examples and small diagrams when they make a boundary clearer.
- Clearly distinguish permanent architectural principles from current implementation details.
- Clearly identify details that are still undecided.
- Do NOT invent decisions that are not explicitly established below.

The most important meta-rule is:

> An undecided detail is not a decision.

If something is not established by this specification, do not silently choose an architecture for it.

---

# 1. Purpose of the Iron Rules

The document must explain that the Iron Rules define architectural constraints and development principles for RoleForge Core.

The Iron Rules are NOT:

- a complete user manual,
- a record of every struct and function,
- a requirement to preserve current filenames forever,
- a replacement for implementation documentation,
- a place for speculative future architecture.

Implementation details may change while the architectural contract remains intact.

For example, the current implementation may use a struct named `CleanRole`, but the permanent architectural rule is about maintaining a neutral Core representation of discovered Roles, not necessarily preserving that exact Rust struct name forever.

Historical implementation prompts are also NOT the live architecture specification.

The repository currently contains historical stage prompts such as:

- Stage 01 — File Loader
- Stage 02 — Main Tokenizer
- Stage 03 — Role Registry and Dispatcher

Those prompts describe the development history at the time they were written.

They must not override newer Iron Rules or explicitly approved architectural decisions.

The Iron Rules and current approved decisions are authoritative.

---

# 2. Fundamental RoleForge Philosophy

RoleForge is NOT one large DSL.

RoleForge is a framework for hosting many small, focused DSLs called Roles.

Each Role is an independent domain / mini-language.

A Role may define its own:

- syntax,
- tokenizer,
- parser,
- internal representation,
- runtime,
- actions,
- Python-facing API,
- execution model,
- validation behavior.

The Core must remain isolated from Role-specific semantics.

The central rule is:

> Core knows the protocol, never the Roles.

The Core may know that a Role exists, what its source identity is, and where it should be delivered.

The Core must not understand the meaning of the Role's internal language.

Adding a new Role must not require adding Role-specific parsing or behavior to the Core.

---

# 3. Source Determines Role Identity

The source file is the source of truth for Role discovery.

The Core discovers Roles from the source before Role-specific behavior is exposed.

Preserve these principles explicitly:

> The source determines what the Role is. The Role determines what API and behavior it exposes.

> Role discovery must happen before Role-specific behavior is exposed.

The Core must not determine Role identity from:

- implementation language,
- Python classes,
- Rust types,
- imported modules,
- execution strategy.

The Role declaration in the source establishes the Role identity.

---

# 4. Core Responsibility Boundary

The Core owns the common infrastructure necessary to move source data to the correct Role boundary.

Conceptually:

    Source file
        ↓
    Loader
        ↓
    Main Tokenizer
        ↓
    Neutral Role representation
        ↓
    Registry resolution
        ↓
    Dispatcher
        ↓
    Role handoff boundary

The Core's responsibility ends after it has successfully resolved and delivered the appropriate Role information to the Role destination.

Preserve this rule prominently:

> The Core resolves and delivers. The Role decides what happens next.

After handoff, the Core must NOT decide:

- whether the Role performs additional tokenization,
- whether the Role uses a parser,
- whether the Role creates an AST,
- whether the Role executes immediately,
- whether the Role exposes API methods,
- whether the Role performs validation,
- whether the Role stores state,
- whether the Role continues processing,
- whether the Role intentionally does nothing.

Those decisions belong to the Role.

---

# 5. Main Tokenizer Boundary

The Main Tokenizer belongs to the Core.

It understands only RoleForge's outer syntax.

Its purpose is to discover Role blocks and produce neutral Role data.

It must NOT understand Role-specific syntax.

Role body content is opaque to the Core except for explicitly established global Core syntax rules.

For example:

    @role Directory
    create src/
    create tests/

The Main Tokenizer may understand:

    Role name = Directory
    Role body = "create src/\ncreate tests/\n"

It must NOT understand what `create` means.

That belongs to the Directory Role.

---

# 6. Current Core Syntax Rules

Document the currently established outer syntax rules accurately.

Role declaration:

    @role RoleName

Rules:

1. `@role` is a Core directive only when it starts at column 0.

2. Leading spaces or tabs mean the line is NOT a Core Role declaration.

3. This is intentional because indentation may belong to the Role's internal language.

4. Full-line Core comments are lines where the first non-whitespace character is `#`.

5. Full-line Core comments are removed from Role content while preserving newline structure.

6. Inline `#` characters inside Role bodies are not interpreted by the Core.

Examples that must remain untouched inside Role bodies:

    value = 10 # inline comment
    color = #FF0000

7. Declaration-line comments are currently supported:

   @role Directory # comment

The Role name is `Directory`.

8. A declaration without a Role name is invalid.

Examples:

    @role
    @role # comment

9. Nonblank, noncomment content before the first valid Role declaration is invalid.

10. An indented `@role` before the first Role is therefore ordinary content before a Role and is invalid.

11. An indented `@role` inside an existing Role body remains opaque Role content.

12. A Role block continues until the next valid column-0 `@role` declaration or EOF.

13. There is no `@end` Core directive.

14. Duplicate Role names are allowed at tokenizer level.

15. Empty or comment-only source may produce zero Roles.

16. Adjacent Role declarations and empty Role bodies are valid.

17. Strings such as:

    @roles Other
    text @role Other
    @end

inside a Role body do not create Role boundaries.

18. The Core does not parse string literals or escaping inside Role bodies.

Do not add new naming restrictions.

PascalCase may be a convention, but strict Role-name validation has not been established as a Core requirement.

Do not invent escaping syntax or additional directives.

---

# 7. No Main Parser in the Current Core Architecture

There is intentionally no Main Parser after the Main Tokenizer.

This is an architectural decision, not merely an unfinished stage.

The Main Tokenizer already produces the neutral information the Core currently needs.

There is no Core-level semantic AST that needs to be constructed afterward.

A Role may have its own pipeline:

    Clean Role data
        ↓
    Role Tokenizer
        ↓
    Role Parser
        ↓
    Role-specific representation
        ↓
    Role Runtime

Another Role may use:

    Clean Role data
        ↓
    Role Runtime

A Role is not required to have either a tokenizer or parser.

Do NOT introduce a Main Parser merely because traditional compiler pipelines commonly contain one.

If a genuine future Core-level parsing requirement appears, the architecture may be reconsidered based on that real requirement.

Until then, no Main Parser belongs in the Core pipeline.

---

# 8. Neutral Role Representation

The Core must produce a neutral representation of discovered Role instances.

The representation must not depend on a specific Role implementation.

The current implementation uses `CleanRole`.

The exact Rust struct name and exact field layout are implementation details and may evolve.

Architecturally, discovered Role data must preserve enough information to maintain:

- Role identity,
- Role body/content,
- source ordering,
- instance identity,
- useful source metadata.

The boundary is conceptually:

    Core syntax
        ↓
    Neutral Role representation
        ↓
    Role-specific world

Do not allow Role-specific data structures to leak backward into the Main Tokenizer.

---

# 9. Dual Role Indexing

Every discovered Role instance must have two distinct indexes.

These indexes are Core identity/source metadata.

They are NOT Role semantics.

## 9.1 Global index

The global index represents the Role instance's position among ALL discovered Role instances in source order.

Example source:

    @role Directory
    ...

    @role Config
    ...

    @role Directory
    ...

    @role Database
    ...

    @role Directory
    ...

    @role Config
    ...

Global identities:

    Directory -> global 0
    Config    -> global 1
    Directory -> global 2
    Database  -> global 3
    Directory -> global 4
    Config    -> global 5

The global index preserves the original order of Role declarations across the entire source file.

## 9.2 Role-local index

Every Role name also has its own independent local sequence.

For the same source:

    Directory -> global 0, local 0
    Config    -> global 1, local 0
    Directory -> global 2, local 1
    Database  -> global 3, local 0
    Directory -> global 4, local 2
    Config    -> global 5, local 1

The local index answers:

"Which instance of this Role name is this?"

The global index answers:

"Where was this Role instance among all discovered Roles?"

## 9.3 Generic indexing requirement

Role-local indexing must be generic.

Do NOT implement Role-specific branching such as:

    if role == "Directory":
        ...
    else if role == "Config":
        ...

The Core must work identically for dynamically discovered/registered Role names.

A generic name-to-counter or equivalent data structure may be used by the implementation.

Do not unnecessarily prescribe one exact data structure in the Iron Rules if another implementation preserves the same contract.

## 9.4 Why both indexes are retained

Both identities should remain available after discovery because they may be useful for:

- public API instance selection,
- diagnostics,
- structured errors,
- warnings,
- logging,
- debugging,
- source identification,
- internal tracing.

Do not discard the global identity when Role-local grouping occurs.

Do not infer global identity later from grouped collections if it has already been discovered.

The Core should retain the identity explicitly.

---

# 10. Source Metadata

Every discovered Role must retain enough source information for meaningful diagnostics.

The current implementation records the source line containing the `@role` declaration.

The current line numbering is 1-based.

The architectural principle is broader than the exact struct layout:

> Every discovered Role retains source information sufficient for meaningful diagnostics.

The tokenizer should record source information when it discovers the Role rather than unnecessarily reconstructing it later.

Because removal of full-line Core comments preserves newline structure, Role-local diagnostics may later use stable source mapping.

Do NOT introduce a complex source-map system before there is a demonstrated need.

Simple useful metadata should be retained now; speculative diagnostic infrastructure should not be invented.

---

# 11. Role Isolation

Roles are independent domains.

The Core must not contain direct dependencies on individual Role implementations.

The Main Tokenizer, Registry, Dispatcher, Runtime, error infrastructure, and other Core systems must remain Role-agnostic.

A new Role should not require changing Core routing logic simply because its internal language differs.

Role-specific tokenization, parsing, validation, execution, and API behavior belong outside the Core.

---

# 12. Implementation Language Is Not Part of the Role Contract

Roles may eventually be implemented in different programming languages.

Implementation language must NOT become part of the Core's Role identity or routing contract.

Preserve explicitly:

> Implementation language is not part of the Role contract.

Do not require metadata such as:

    language = "python"
    language = "rust"
    runtime_type = "native"

merely so the Core can resolve a Role.

The Core resolves destinations, not implementation strategies.

Preserve:

> The Core resolves destinations, not implementations.

Do not invent a Python/Rust execution protocol yet.

The physical cross-language handoff mechanism remains undecided.

---

# 13. Registry Responsibility

The Registry maps Role names to registered Role destinations.

The Registry is responsible for registration metadata and resolution.

It is NOT responsible for:

- parsing Role content,
- understanding Role semantics,
- executing Roles,
- controlling Role runtimes,
- formatting user-facing errors,
- printing console output.

There are currently two registry sources:

- built-in Roles,
- dynamic Roles.

Built-in and dynamic registrations remain separate concepts.

The current minimal registration metadata contains the Role name and entry destination.

Do not add language/type/runtime metadata merely for routing.

---

# 14. Registry Path Resolution

Relative built-in Role destinations are resolved relative to the built-in Role location.

Relative dynamic Role destinations are resolved relative to the dynamic Role location.

Absolute paths remain absolute.

Resolution must not accidentally depend on the process's Python/Rust current working directory.

The current physical directories include:

    roleforge/builtin_roles/
    roleforge/roles/

The current Rust implementation uses a source-tree-derived RoleForge root.

Treat the exact implementation mechanism as an implementation detail.

Do not establish the current `CARGO_MANIFEST_DIR` strategy as a permanent packaging architecture.

Packaging/runtime path behavior may need to evolve later.

Do not redesign it in this document.

---

# 15. Registry Resolution Rules

Role resolution has three conceptual states:

    Resolved
    Unknown
    Conflict

The rules are:

> A Role name registered in exactly one registry can be resolved.

> A Role name registered in neither registry is unknown.

> A Role name registered in both registries is a conflict and must never be resolved by precedence.

There is NO automatic precedence between built-in and dynamic registrations.

If the same Role name exists in both registries, the Core must not silently choose either one.

Both conflicting destinations should remain available in structured conflict information where useful for diagnostics.

A conflict for one Role must not automatically prevent later independent Role instances from being examined.

---

# 16. Dispatcher Responsibility

The Dispatcher receives neutral discovered Role data and uses Registry resolution to determine routing outcomes.

The Dispatcher must preserve source order.

Its conceptual outcomes are:

    Resolved
    Unknown
    Conflict

A resolved result contains the Role information and the resolved destination.

An unknown result preserves the Role information.

A conflict result preserves the Role information and the conflicting registration information.

The Dispatcher is NOT a Role Runtime.

The Dispatcher must not understand Role-specific content.

The Dispatcher must not execute Role-specific logic.

The Dispatcher must not decide how a Role internally continues after handoff.

---

# 17. Handoff Boundary

A successfully resolved Role eventually crosses a handoff boundary from Core infrastructure into the Role-specific world.

Conceptually:

    Core
      ↓
    resolved Role
      ↓
    handoff
    =========================
    Role-specific world

After this boundary, the Core does not orchestrate the Role's internal processing.

The Role may decide to:

- tokenize its content,
- parse its content,
- validate,
- execute,
- expose API state,
- defer work,
- do nothing.

The Core does not need to know.

The physical handoff mechanism is NOT yet decided.

Do NOT invent:

- an ABI,
- FFI protocol,
- Python bridge architecture,
- Rust plugin ABI,
- process model,
- dynamic library protocol,
- serialization protocol,
- callback system,

unless such a mechanism is explicitly designed in a later stage.

The architectural rule is the boundary itself, not its future physical implementation.

---

# 18. Core Runtime / Orchestration

Major Core components process their own inputs.

The Core Runtime/Orchestrator owns the execution order between major Core components.

Preserve:

> Components process. Runtime orchestrates.

Conceptually:

    Core Runtime
        │
        ├── Loader
        │      ↓
        ├── Main Tokenizer
        │      ↓
        ├── Registry / Dispatcher
        │      ↓
        └── Core handoff boundary

A major pipeline component must not absorb orchestration responsibility merely for convenience.

For example:

- Loader should not become the Main Tokenizer.
- Main Tokenizer should not drive Role execution.
- Dispatcher should not become a Role Runtime.

Internal helper calls within one component are allowed.

This rule applies to major architectural boundaries, not every function call.

Preserve:

> Core Runtime orchestrates the Core pipeline, never Role behavior.

The Core Runtime may coordinate the pipeline until Role delivery/handoff.

It must not manage the Role's internal lifecycle after that boundary.

---

# 19. Clear Component Boundaries

Every major component should have a clear input and output boundary.

Conceptually:

    clear input
        ↓
    component
        ↓
    clear output

Do not create abstractions merely to increase the number of layers.

For example, a dedicated `output.rs` file or output wrapper is useful only when it represents a meaningful contract or transformation.

Do not wrap a `Vec<T>` in another type solely because every subsystem is expected to have an "output object".

Create structures when they represent real boundaries, invariants, identity, or transformation.

---

# 20. Errors, Warnings, and Structured Events

Core components report structured information.

They do not present user-facing messages themselves.

Preserve:

> Components report structured events/errors; they do not present them.

For example, a component should not do:

    println!("Unknown role: Directory");

Instead, it should return or emit structured information representing the condition.

The future error/event layer may decide how the condition should be classified and presented.

Do NOT prematurely decide every severity.

In particular, whether an Unknown Role is ultimately an error, warning, or another event category is not yet permanently established.

Do not encode an undecided severity into architecture merely because a current enum or test needs a temporary representation.

---

# 21. Console / Output Layer

All user-facing console output from the Core must pass through a dedicated output/presentation layer.

Preserve:

> All console output goes through a dedicated console/output layer.

Normal Core components should not directly print user-facing messages.

Conceptually:

    Component
        ↓
    structured event/error
        ↓
    event/error handling
        ↓
    console/output presentation

The future Error Manager and Console Manager must not automatically be treated as one giant subsystem.

Do not create a God Object that performs:

- error detection,
- classification,
- aggregation,
- formatting,
- logging,
- console rendering,
- unrelated orchestration,

all in one place.

Exact error/output architecture will be designed when that stage is reached.

---

# 22. Dynamic Roles Are a Core Capability

RoleForge must support Roles beyond those shipped with the library.

Dynamic Role registration is a fundamental capability.

Adding a dynamic Role must not require hardcoding that Role into the Core.

Current project structure includes:

    roleforge/builtin_roles/
    roleforge/roles/

`roles/` is the current conventional/default location for dynamic Role implementations.

Do not interpret that convention as meaning every future Role must physically live there under all configurations.

The exact future API for:

- installing,
- registering,
- unregistering,
- removing,
- overriding Role locations,

has not yet been finalized.

Do not invent it in the Iron Rules.

---

# 23. Public API Philosophy

The public API should be simple for normal usage while allowing explicit control when needed.

Preserve:

> Simple by default, explicit when needed.

The conceptual model is:

    RoleForge = engine
    Project   = loaded result
    Role      = independent domain / mini-language

There should be one underlying Core operation with convenience interfaces around it.

Preserve:

> One core operation, multiple convenience interfaces.

The final Python API syntax is NOT fully finalized.

Examples in the Iron Rules are conceptual unless explicitly marked as contractual.

---

# 24. Multiple Instances of the Same Role

Multiple declarations of the same Role name are valid.

For example:

    @role Directory
    ...

    @role Config
    ...

    @role Directory
    ...

The Core must not assume there is only one Directory Role instance.

Therefore, a conceptual public API such as:

    project.directory.validate()

must NOT be documented as if it inherently identifies one unique instance.

For the current conceptual API direction, Role-local indexing can be used to select an instance:

    project.directory[0].validate()
    project.directory[1].validate()

Here:

    project.directory[0]

conceptually refers to the first Directory instance by Role-local index, regardless of its global source index.

The exact Python object model and syntax remain subject to later API design.

Do NOT yet decide what:

    project.directory.validate()

should mean when:

- there is one Directory instance,
- there are multiple Directory instances,
- there are zero Directory instances.

Possible shorthand behavior may be considered later.

Do not invent it now.

---

# 25. Future Named Role Instance Idea Is Undecided

There is a future idea of allowing an instance of a Role to receive a unique name/alias while still being based on the same underlying Role definition.

Conceptually, this is somewhat similar to:

    class Source(Directory):
        pass

    class Tests(Directory):
        pass

where the new identity exists without adding functionality.

However, this is NOT an approved feature yet.

Do NOT design or establish:

- alias syntax,
- named instance syntax,
- inheritance semantics,
- Registry behavior for aliases,
- uniqueness rules,
- API exposure,
- tokenizer representation.

This may be explored later.

Until then:

> An undecided detail is not a decision.

---

# 26. Role Interaction Model

A Role may conceptually choose to expose behavior through:

- DSL only,
- Python API only,
- both.

The Core should not force every Role into the same interaction model.

However, the exact representation of this capability is NOT finalized.

Do not automatically add metadata such as:

    interaction_mode = "dsl"
    interaction_mode = "python_api"
    interaction_mode = "hybrid"

until an actual contract requiring it has been designed.

---

# 27. Avoid God Objects

No component should accumulate unrelated responsibilities merely because centralization is convenient.

Examples:

- Runtime orchestrates; it does not parse Role languages.
- Registry resolves registrations; it does not execute Roles.
- Dispatcher routes; it does not become the Role Runtime.
- Main Tokenizer understands outer Core syntax; it does not parse Role DSLs.
- Error infrastructure handles structured error/event information; it does not automatically own all presentation.
- Console/output infrastructure presents information; it does not make Role decisions.

Prefer explicit boundaries over convenient coupling.

---

# 28. Development Methodology — Local Optimization

Performance quality matters during normal development.

RoleForge should avoid obvious unnecessary performance waste.

When implementing or reviewing a component, prefer simple local optimizations when they clearly reduce unnecessary:

- I/O operations,
- allocations,
- copies,
- repeated parsing,
- repeated computation,
- unnecessary intermediate data.

These improvements should be made during normal development when they are:

- simple,
- obvious,
- safe,
- local,
- easy to understand,
- compatible with the established architecture.

Do NOT interpret "avoid premature optimization" as permission to knowingly keep obvious waste.

At the same time, do not introduce speculative architectural optimization such as:

- complex caching,
- concurrency solely for hypothetical performance,
- unsafe code solely for hypothetical performance,
- custom allocation strategies without demonstrated need,
- performance-driven architectural redesign without evidence.

Preserve:

> Optimize obvious waste immediately. Postpone architectural optimization.

Performance is part of implementation quality.

It must not compromise:

- clarity,
- isolation,
- maintainability,
- correctness,
- established architectural boundaries.

---

# 29. AI / Contributor Rules

This section is especially important because the document is primarily intended for AI-assisted development.

An AI modifying RoleForge must:

1. Read and follow the Iron Rules before making architectural changes.

2. Preserve established component boundaries.

3. Avoid adding Role-specific knowledge to the Core.

4. Avoid inventing future systems merely because they seem useful.

5. Distinguish current implementation from architectural contract.

6. Avoid converting temporary implementation details into permanent rules.

7. Avoid speculative abstractions.

8. Avoid speculative metadata.

9. Avoid speculative execution systems.

10. Preserve structured outputs instead of introducing direct presentation inside processing components.

11. Perform simple safe local optimizations when obvious.

12. Postpone architectural optimization until there is demonstrated need.

13. Preserve source identity and metadata rather than reconstructing information unnecessarily later.

14. Preserve both global and Role-local instance identity.

15. Never resolve cross-registry conflicts using implicit precedence.

16. Never assume one Role declaration per Role name.

17. Never assume every Role has a parser.

18. Never assume every Role uses the same implementation language.

19. Never assume every Role uses the same execution model.

20. Never silently decide an unresolved architectural question.

When an implementation task appears to require changing an Iron Rule, stop and identify the conflict instead of silently redesigning the architecture.

The guiding meta-rule is:

> An undecided detail is not a decision.

---

# 30. Historical Development Stages

The document may briefly record the currently completed Core stages for context, but they must not become the architectural source of truth.

Current development history:

Stage 01:
File Loader

Stage 02:
Main Tokenizer

Stage 03:
Role Registry and Dispatcher

The Main Parser was intentionally removed from the Core architecture.

Historical prompt files are development records.

Do not rewrite historical prompts merely because a later decision refined the architecture.

Small corrections to a completed stage do not automatically create a new numbered development stage.

Future stage numbering must follow explicitly agreed development planning rather than being inferred from old assumptions.

---

# 31. Explicitly Undecided Areas

Include a clearly visible section listing major areas that are intentionally NOT decided yet.

At minimum include:

- physical Role handoff mechanism,
- cross-language execution mechanism,
- Python/Rust bridge architecture for Roles,
- final Project/Public API object model,
- shorthand behavior for a single Role instance,
- named Role instance / alias system,
- alias syntax,
- final Role registration/install/remove API,
- exact error severity model,
- final Error Manager architecture,
- final Console Manager architecture,
- `interaction_mode` metadata,
- advanced source mapping,
- final packaging/runtime path-resolution strategy.

Do not attempt to solve these while writing the document.

Their presence in the undecided section is intentional protection against accidental AI architecture invention.

---

# 32. Required Short Version

End the document with a section named:

    Short Version — Iron Rules Checklist

This is NOT a replacement for the full document.

It is a quick pre-modification checklist for an AI coding agent.

It must include, at minimum, these principles:

1. RoleForge is a framework for independent mini-languages, not one giant DSL.

2. Core knows the protocol, never the Roles.

3. The source determines what the Role is; the Role determines its API and behavior.

4. Role discovery happens before Role-specific behavior is exposed.

5. Main Tokenizer understands only Core outer syntax.

6. Role body content remains opaque to the Core.

7. There is no Main Parser in the current Core architecture.

8. Every Role instance retains useful source metadata.

9. Every Role instance retains both:
    - global index,
    - Role-local index.

10. Duplicate Role declarations are valid.

11. Implementation language is not part of the Role contract.

12. The Core resolves destinations, not implementations.

13. Exactly one registry match = Resolved.

14. No registry match = Unknown.

15. Built-in + dynamic match = Conflict.

16. Conflicts are never resolved through precedence.

17. Components process. Runtime orchestrates.

18. Core Runtime orchestrates the Core pipeline, never Role behavior.

19. The Core resolves and delivers. The Role decides what happens next.

20. Components report structured events/errors; they do not present them.

21. All console output goes through a dedicated console/output layer.

22. Dynamic Roles must not require Role-specific Core changes.

23. Avoid God Objects.

24. Simple by default, explicit when needed.

25. One core operation, multiple convenience interfaces.

26. Optimize obvious waste immediately. Postpone architectural optimization.

27. An undecided detail is not a decision.

---

# 33. Writing Requirements

Write the actual Iron Rules document in English.

Use clear Markdown.

The document should be detailed and explicit.

Do NOT intentionally shorten it for human readability.

This is primarily an architectural reference for AI coding agents.

Use:

- headings,
- subheadings,
- bullet lists,
- blockquotes,
- code examples,
- small ASCII pipeline diagrams,

where useful.

Do not add decorative content that does not improve architectural precision.

Do not introduce unrelated architectural decisions.

Do not modify the RoleForge implementation as part of this task unless explicitly required only to place/save the generated document.

Do not modify the historical Stage 01, Stage 02, or Stage 03 prompt contents.

If an existing Iron Rules file contains the accidentally overwritten optimization-update prompt rather than the real Iron Rules, replace that content with the newly rebuilt complete document.

Before finalizing, verify that no section contradicts another section.

In particular verify:

- duplicate Role declarations are compatible with the dual-index model,
- Runtime ownership ends at the Core/Role boundary,
- Dispatcher is not described as a Role Runtime,
- Registry does not gain execution responsibility,
- Role implementation language does not affect resolution,
- Unknown and Conflict do not introduce implicit precedence,
- the Main Parser is not accidentally reintroduced,
- conceptual Public API examples do not assume one instance per Role,
- undecided features remain explicitly undecided.

After writing the file, report in Hebrew:

1. where the final Iron Rules file was created or updated;
2. the major architectural sections it contains;
3. how the dual-index rule was represented;
4. which areas were explicitly left undecided;
5. whether you found any contradiction or ambiguity while rebuilding the document;
6. whether you made any change outside the Iron Rules document.

Do not claim an undecided issue has been solved.

Respond to me in Hebrew.