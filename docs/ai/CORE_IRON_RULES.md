# RoleForge Core Iron Rules

## 1. Purpose, authority, and interpretation

This document defines the architectural constraints and development principles of RoleForge Core. It is primarily a strict reference for AI coding agents and contributors modifying the project. It is not the final human-facing documentation.

The Iron Rules are not:

- A complete user manual.
- An inventory of every struct, function, or module.
- A requirement to preserve current filenames forever.
- A replacement for implementation documentation.
- A place to establish speculative future architecture.

Implementation details may change while the architectural contract remains intact. For example, the current neutral representation is named `CleanRole`; preserving neutral discovered Role information is the architectural requirement, while that exact Rust name and field layout are not permanent requirements.

Read the document using three distinct categories:

- **Architectural principles and constraints:** boundaries and invariants that implementation work must preserve unless an architectural change is explicitly approved.
- **Current established behavior or implementation details:** the current syntax contract and implementation context, explicitly identified where relevant. A current mechanism must not silently become a permanent architectural requirement.
- **Undecided areas:** questions for which no architecture has been approved. Examples or plausible implementations do not settle them.

> An undecided detail is not a decision.

Historical development prompts, including Stage 01 — File Loader, Stage 02 — Main Tokenizer, and Stage 03 — Role Registry and Dispatcher, record the development state when they were written. They are not the live architecture specification. They must not override newer Iron Rules or explicitly approved architectural decisions. Temporary notes and historical examples must likewise not be used to restore superseded assumptions.

The Iron Rules and current explicitly approved architectural decisions are authoritative. When a task appears to require violating an established rule, identify the conflict before changing the architecture. Do not silently reinterpret a rule to justify a convenient implementation.

## 2. Fundamental philosophy: independent mini-languages

RoleForge is not one large DSL. It is a framework for hosting many small, focused DSLs called **Roles**. Each Role is an independent domain or mini-language.

A Role may define its own:

- Syntax, tokenizer, and parser.
- Internal representation.
- Runtime and execution model.
- Actions and validation behavior.
- Python-facing API.

These are capabilities a Role may choose, not required stages or a mandatory common implementation template.

> Core knows the protocol, never the Roles.

The Core may know that a Role exists, its source identity, and the destination to which its information should be delivered. The Core must not understand the meaning of that Role's internal language.

Adding a Role must not require adding Role-specific parsing, semantics, or behavior to Core. A new mini-language must fit the common discovery and routing boundary without teaching Core that language.

## 3. Source determines Role identity

The source file is the source of truth for discovery. A Role declaration in the source establishes which Role is being declared.

> The source determines what the Role is. The Role determines what API and behavior it exposes.

> Role discovery must happen before Role-specific behavior is exposed.

Core must not determine Role identity from:

- The implementation language.
- A Python class or Rust type.
- Imported modules.
- An execution strategy.

Registration supplies a destination for an already discovered Role name. It does not replace the source declaration as the source of identity. Similarly, convenience API wrappers must not bypass discovery by assuming that the source belongs to a particular Role implementation.

## 4. Core responsibility boundary

Core owns the common infrastructure necessary to move source data to the correct Role boundary.

```text
Source file
    |
    v
Loader
    |
    v
Main Tokenizer
    |
    v
Neutral Role representation
    |
    v
Registry resolution
    |
    v
Dispatcher
    |
    v
Role handoff boundary
```

This is a conceptual data flow, not a requirement for each component to invoke the next component. Core Runtime owns orchestration between major components.

> The Core resolves and delivers. The Role decides what happens next.

Core's responsibility ends after it has successfully resolved and delivered the appropriate Role information to the Role destination. A resolution outcome alone must not be confused with an already implemented physical handoff mechanism.

After handoff, Core must not decide whether a Role:

- Performs additional tokenization.
- Uses a parser or constructs an AST.
- Executes immediately.
- Exposes API methods.
- Performs validation.
- Stores state.
- Continues processing, defers work, or intentionally does nothing.

Those decisions belong to the Role. Core delivery does not authorize Core to manage the Role's internal lifecycle.

## 5. Main Tokenizer boundary

The Main Tokenizer belongs to Core. It understands only RoleForge's outer syntax, discovers Role blocks, and produces neutral Role data.

Role body content is opaque to Core except for explicitly established global Core syntax rules, such as full-line Core comment handling. The tokenizer must not parse Role-specific syntax or infer Role-specific meaning.

For example:

```text
@role Directory
create src/
create tests/
```

Core may discover:

```text
Role name = Directory
Role body = "create src/\ncreate tests/\n"
```

Core must not understand what `create` means, whether the paths are valid Directory commands, or what actions those lines request. Those questions belong to the Directory Role.

## 6. Current established outer syntax

This section records the currently established Core syntax. Do not extend it with inferred directives, escaping rules, or naming restrictions.

### 6.1 Role declarations and column position

The declaration form is:

```text
@role RoleName
```

`@role` is a Core directive only when it starts at column 0. Leading spaces or tabs mean the line is not a Core Role declaration. This is intentional: indentation may belong to the Role's internal language.

Declaration-line comments are currently supported:

```text
@role Directory # comment
```

The discovered Role name is `Directory`.

A declaration without a Role name is invalid. Both of these are invalid:

```text
@role
@role # comment
```

Do not turn a nameless Core declaration into a valid empty-name Role. The valid-boundary rule below does not make malformed Core declarations valid.

No additional strict Role-name validation has been established as a Core requirement. PascalCase may be a convention, but must not become a mandatory naming rule merely because examples use it. This document does not invent a broader naming grammar.

### 6.2 Full-line Core comments and inline hashes

A full-line Core comment is a line whose first non-whitespace character is `#`.

Full-line Core comments are removed from Role content **while preserving newline structure**. Removing the comment must not collapse the surrounding source lines together.

Core does not interpret inline `#` characters inside Role bodies. These body lines must remain untouched:

```text
value = 10 # inline comment
color = #FF0000
```

Declaration-line comment support is a rule about the Core declaration line; it is not permission to strip inline comments from Role content.

### 6.3 Content before the first Role

Nonblank, noncomment content before the first valid Role declaration is invalid. Blank lines and full-line Core comments may appear before the first Role.

An indented `@role` before the first Role is ordinary content, not a declaration, and is therefore invalid in that position:

```text
    @role Directory
```

Inside an existing Role body, an indented `@role` remains opaque Role content:

```text
@role Directory
    @role SomethingInsideTheRoleLanguage
```

This source contains one Core Role declaration. Core does not interpret the indented line's Role-specific meaning.

### 6.4 Block boundaries and empty sources

A Role block continues until the next valid column-0 `@role` declaration or EOF. There is no `@end` Core directive.

Duplicate Role names are allowed at tokenizer level. Adjacent Role declarations and empty Role bodies are valid:

```text
@role Directory
@role Directory
@role Config
```

This discovers three instances, including two independent Directory instances. Empty or comment-only source may produce zero Roles.

Inside a Role body, the following strings do not create Role boundaries:

```text
@roles Other
text @role Other
@end
```

`@roles` is not the `@role` directive, a mid-line `@role` is not a column-0 declaration, and `@end` has no Core directive meaning.

Core does not parse string literals or escaping inside Role bodies. Do not add quote-aware Role parsing, an escaping syntax, or extra directives to alter these rules. Role-specific string syntax cannot be used by Core to infer exceptions to established outer syntax.

## 7. No Main Parser in the current Core architecture

There is intentionally no Main Parser after the Main Tokenizer. This is an architectural decision, not an unfinished development stage.

The Main Tokenizer already produces the neutral information Core currently needs. There is no Core-level semantic AST to construct afterward.

A Role may choose a pipeline such as:

```text
Clean Role data
    |
    v
Role Tokenizer
    |
    v
Role Parser
    |
    v
Role-specific representation
    |
    v
Role Runtime
```

Another Role may choose:

```text
Clean Role data
    |
    v
Role Runtime
```

Neither pipeline is mandatory. A Role is not required to have a tokenizer or parser.

Do not introduce a Main Parser because traditional compiler pipelines commonly contain one, or because an old prompt expected one. If a genuine future Core-level parsing requirement appears, the architecture may be reconsidered explicitly based on that requirement. Until then, a Main Parser does not belong in the Core pipeline.

## 8. Neutral Role representation

Core must produce a neutral representation of discovered Role instances. This representation must not depend on any particular Role implementation.

The current implementation uses `CleanRole`. The exact Rust struct name and exact field layout may evolve. The architectural contract is to preserve enough information to maintain:

- Role identity.
- Role body/content.
- Source ordering.
- Instance identity, including both indexes described below.
- Useful source metadata.

```text
Core syntax
    |
    v
Neutral Role representation
    |
    v
Role-specific world
```

Do not allow Role-specific AST nodes, parsed commands, runtime state models, or other Role-specific structures to leak backward into the Main Tokenizer. Neutral identity and source metadata are Core concerns; the interpretation of a Role body is not.

## 9. Dual Role indexing

Every discovered Role instance must have two distinct indexes: a **global index** and a **Role-local index**. These are Core identity/source metadata, not Role semantics.

### 9.1 Global index

The global index is the instance's position among all discovered Role instances in source order, starting at zero. It preserves declaration order across the entire source file, regardless of Role name.

### 9.2 Role-local index

Each Role name has its own independent local sequence, starting at zero. The Role-local index identifies which instance of that name was discovered.

For this source:

```text
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
```

The identities are:

| Role name | Global index | Role-local index |
| --- | --- | --- |
| Directory | 0 | 0 |
| Config | 1 | 0 |
| Directory | 2 | 1 |
| Database | 3 | 0 |
| Directory | 4 | 2 |
| Config | 5 | 1 |

The two indexes answer different questions:

- Global: “Where was this instance among all discovered Roles?”
- Role-local: “Which instance of this Role name is this?”

A local index alone is not a unique identifier across different Role names. Directory local 0 and Config local 0 are different instances, with different global identities.

### 9.3 Generic indexing requirement

Role-local indexing must work generically for discovered Role names, including dynamically registered Roles. Do not add Role-specific branches such as:

```text
if role == "Directory":
    increment_directory_counter()
else if role == "Config":
    increment_config_counter()
```

A generic name-to-counter structure or an equivalent implementation may be used. No particular container or exact field layout is required by this document; the two identity sequences are required.

Indexing belongs to discovery identity. It must not depend on whether a later registry result is Resolved, Unknown, or Conflict. Repeated source declarations do not themselves constitute a registration conflict.

### 9.4 Retain both identities

Both indexes must remain available after discovery. They may support public API instance selection, diagnostics, structured errors, warnings, logging, debugging, source identification, and internal tracing.

Do not discard global identity when grouping instances by Role name. Do not reconstruct global identity from grouped collections after discovery has already established it. Retain the identity explicitly.

Source order and per-name selection must coexist: grouping Directory instances for an API must not lose their original global positions of 0, 2, and 4 in the example above.

## 10. Source metadata

> Every discovered Role retains source information sufficient for meaningful diagnostics.

The current implementation records the source line containing the `@role` declaration, using 1-based line numbering. This is distinct from the zero-based instance indexes.

The broader architectural rule is useful diagnostic source information, not a permanently fixed metadata struct. The tokenizer should record source information when it discovers a Role instead of unnecessarily reconstructing it later.

Preserving newline structure when removing full-line Core comments allows later Role-local diagnostics to use stable source mapping. This does not establish a complex source-map system now. Retain simple useful metadata; introduce advanced mapping only when a demonstrated need justifies its design.

## 11. Role isolation

Roles are independent domains. Core must not contain direct dependencies on individual Role implementations.

This requirement applies throughout Core, including:

- Main Tokenizer.
- Registry.
- Dispatcher.
- Core Runtime/Orchestrator.
- Error/event infrastructure.
- Other common Core systems.

Role-specific tokenization, parsing, validation, execution, and API behavior belong outside Core. A new Role must not require modifying Core routing logic simply because its language differs from existing Roles.

Knowing a registered Role name and destination is compatible with isolation. Importing or branching on a particular Role's semantic implementation inside common Core processing is not.

## 12. Implementation language is not part of the contract

> Implementation language is not part of the Role contract.

Roles may eventually be implemented in different programming languages. Language must not become part of Core Role identity or the routing contract.

Do not require metadata such as the following merely to resolve a Role:

```text
language = "python"
language = "rust"
runtime_type = "native"
```

> The Core resolves destinations, not implementations.

Core resolves a Role name to its registered destination. It does not choose a Role's implementation strategy or infer identity from the language used at that destination.

This principle does not define a cross-language execution system. The physical handoff, Python/Rust bridge, and any other cross-language mechanism remain undecided. Do not invent them to fill in the conceptual routing diagram.

## 13. Registry responsibility

The Registry maps Role names to registered Role destinations. It owns registration metadata and resolution.

There are currently two registry sources:

- Built-in Roles.
- Dynamic Roles.

Built-in and dynamic registrations remain separate concepts. Current minimal registration metadata consists of the Role name and entry destination. Do not add language, type, or runtime metadata merely for routing.

The Registry must not:

- Parse Role content or understand Role semantics.
- Execute Roles.
- Control Role runtimes.
- Format user-facing errors.
- Print console output.

Resolution information can support later delivery and diagnostics without giving the Registry execution or presentation responsibilities.

## 14. Registry path resolution

Relative built-in Role destinations are resolved relative to the built-in Role location. Relative dynamic Role destinations are resolved relative to the dynamic Role location. Absolute paths remain absolute.

Resolution must not accidentally depend on the process's Python or Rust current working directory. Changing the caller's working directory must not silently change which base a registry-relative destination uses.

Current physical directories include:

```text
roleforge/builtin_roles/
roleforge/roles/
```

The current Rust implementation uses a source-tree-derived RoleForge root. The exact mechanism is an implementation detail. In particular, the current `CARGO_MANIFEST_DIR` strategy must not be declared a permanent packaging architecture.

Final packaging and runtime path-resolution strategy may evolve later. This document preserves the established relative-base distinction and absolute-path behavior without redesigning packaging.

## 15. Registry resolution rules

Resolution has three conceptual states:

```text
Resolved
Unknown
Conflict
```

| Built-in registration for the name | Dynamic registration for the name | Outcome |
| --- | --- | --- |
| Present | Absent | Resolved to built-in destination |
| Absent | Present | Resolved to dynamic destination |
| Absent | Absent | Unknown |
| Present | Present | Conflict |

> A Role name registered in exactly one registry can be resolved.

> A Role name registered in neither registry is unknown.

> A Role name registered in both registries is a conflict and must never be resolved by precedence.

There is no automatic precedence between built-in and dynamic registrations. Do not silently choose the built-in registration, the dynamic registration, the first entry encountered, or a destination selected according to implementation language.

Both conflicting destinations should remain available in structured conflict information where useful for diagnostics. A conflict for one Role must not automatically prevent later independent Role instances from being examined.

Unknown and Conflict are resolution outcomes. They do not establish the final user-facing severity model, and neither authorizes a fallback that silently chooses a conflicting destination.

Multiple declarations of a Role in source are separate instances. They are not the same issue as that Role name being registered in both registry sources.

## 16. Dispatcher responsibility

The Dispatcher receives neutral discovered Role data and uses Registry resolution to determine routing outcomes. It must preserve source order.

Its conceptual outcomes are:

- **Resolved:** retains the Role information and resolved destination.
- **Unknown:** retains the Role information.
- **Conflict:** retains the Role information and conflicting registration information.

Role information includes the instance identity and useful source metadata established during discovery. Routing must not collapse repeated declarations into a single instance or discard their identity when an outcome is not Resolved.

The Dispatcher is not a Role Runtime. It must not:

- Understand Role-specific content.
- Execute Role-specific logic.
- Decide whether a Role tokenizes, parses, validates, or executes.
- Decide how a Role internally continues after handoff.

Producing a resolved routing result is conceptually distinct from choosing a physical handoff protocol. The latter remains undecided.

## 17. Handoff boundary

A successfully resolved Role eventually crosses the boundary from Core infrastructure into the Role-specific world.

```text
Core infrastructure
    |
    v
Resolved Role information + destination
    |
    v
Handoff
======================================
Role-specific world
```

After this boundary, Core does not orchestrate the Role's internal processing. The Role may tokenize, parse, validate, execute, expose API state, defer work, or do nothing. Core does not need to know which path it chooses.

The architectural decision is the responsibility boundary. The physical handoff mechanism is not yet decided.

Do not invent any of the following without a later explicit design decision:

- ABI or FFI protocol.
- Python bridge architecture.
- Rust plugin ABI.
- Process model.
- Dynamic library protocol.
- Serialization protocol.
- Callback system.

A conceptual handoff arrow must not be treated as approval for one of these mechanisms.

## 18. Core Runtime and orchestration

> Components process. Runtime orchestrates.

Major Core components process their own inputs. Core Runtime/Orchestrator owns execution order between those components.

```text
Core Runtime
    |
    +-- Loader
    |      |
    |      v
    +-- Main Tokenizer
    |      |
    |      v
    +-- Registry / Dispatcher
    |      |
    |      v
    +-- Core handoff boundary
```

A major component must not absorb orchestration responsibilities merely for convenience:

- Loader must not become the Main Tokenizer.
- Main Tokenizer must not drive Role execution.
- Dispatcher must not become a Role Runtime.

Internal helper calls within a component are allowed. This rule governs major architectural boundaries; it is not a prohibition on ordinary function composition.

> Core Runtime orchestrates the Core pipeline, never Role behavior.

Core Runtime may coordinate the pipeline until Role delivery/handoff. It must not control the Role's internal lifecycle afterward. This orchestration principle does not decide which future component physically implements an as-yet-undesigned handoff mechanism.

## 19. Clear component boundaries and meaningful structures

Each major component should have a clear input and output boundary:

```text
Clear input
    |
    v
Component
    |
    v
Clear output
```

Create structures when they represent real boundaries, invariants, identity, or transformations. Do not create abstractions merely to increase the number of layers.

A dedicated `output.rs` file or output wrapper is useful only when it represents a meaningful contract or transformation. Do not wrap a `Vec<T>` solely because every subsystem is expected to have an “output object.”

A processing component's output data contract is not the same thing as user-facing console presentation. Requiring a dedicated presentation layer does not require a redundant output wrapper for every component.

## 20. Errors, warnings, and structured events

> Components report structured events/errors; they do not present them.

Core components return or emit structured information about conditions they encounter. They do not themselves present user-facing messages.

For example, a processing component must not do:

```rust
println!("Unknown role: Directory");
```

It should instead report structured information identifying the condition and relevant Role information. This example establishes the reporting/presentation boundary, not a final event type, exact enum, or transport mechanism.

The future error/event layer may decide how a condition is classified and presented. Do not prematurely decide every severity. In particular, whether an Unknown Role is ultimately an error, warning, or another event category is not permanently established.

A temporary enum name or test representation does not settle the architecture's severity model. Preserve the routing meaning of Unknown and Conflict without inventing final presentation policy.

## 21. Console and output presentation

> All console output goes through a dedicated console/output layer.

All user-facing console output from Core must pass through dedicated output/presentation infrastructure. Normal processing components must not directly print user-facing messages.

```text
Component
    |
    v
Structured event/error
    |
    v
Event/error handling
    |
    v
Console/output presentation
```

This establishes a separation of responsibilities, not the final Error Manager or Console Manager design.

Do not automatically merge the future Error Manager and Console Manager into a single giant subsystem responsible for error detection, classification, aggregation, formatting, logging, console rendering, and unrelated orchestration.

Exact error/output architecture will be designed when that stage is reached. Presentation must not acquire responsibility for deciding Role behavior.

## 22. Dynamic Roles are a Core capability

RoleForge must support Roles beyond those shipped with the library. Dynamic Role registration is a fundamental capability, not an exception requiring Role-specific Core changes.

Current project structure includes:

```text
roleforge/builtin_roles/
roleforge/roles/
```

`roles/` is the current conventional/default location for dynamic Role implementations. It does not mean every future Role must physically live there under every configuration.

The exact future APIs for installing, registering, unregistering, removing, and overriding Role locations are not finalized. Do not invent them in implementation work that does not explicitly design those contracts.

The ability to support dynamic Roles does not authorize implicit precedence over built-in registrations. Cross-registry name conflicts remain Conflict.

## 23. Public API philosophy

> Simple by default, explicit when needed.

The conceptual model is:

```text
RoleForge = engine
Project   = loaded result
Role      = independent domain / mini-language
```

There should be one underlying Core operation, with convenience interfaces around it.

> One core operation, multiple convenience interfaces.

Convenience interfaces must preserve source-driven discovery and the same architectural boundaries. They must not become competing semantic pipelines that infer Role identity differently.

The final Python API syntax and final Project object model are not fully finalized. Public API examples in this document are conceptual unless explicitly identified as contractual. The philosophy does not fix method names, loading syntax, wrapper types, or how Role-specific APIs attach to the loaded result.

## 24. Multiple instances of the same Role

Multiple declarations of the same Role name are valid:

```text
@role Directory
...
@role Config
...
@role Directory
...
```

Core must not assume there is only one Directory instance. In this example, the Directory instances have global indexes 0 and 2, and Role-local indexes 0 and 1.

The current conceptual API direction can use Role-local indexing to select an instance:

```python
# Conceptual syntax, not a finalized Python object model.
project.directory[0].validate()
project.directory[1].validate()
```

`project.directory[0]` conceptually selects the first Directory instance by Role-local index, regardless of its global source index. The Role defines the meaning and availability of `validate()`; Core does not impose that method on all Roles.

Do not document the following expression as inherently identifying one unique instance:

```python
project.directory.validate()
```

Its behavior is undecided when there is one Directory instance, multiple Directory instances, or zero Directory instances. Do not invent automatic singleton selection, broadcasting, or any other shorthand behavior. Exact selection syntax and the Python object model remain subject to later API design.

## 25. Future named Role instances and aliases: undecided

A future idea is to give a Role instance a unique name or alias while retaining the same underlying Role definition. The analogy is somewhat like a new class identity without added functionality:

```python
# Analogy only: not approved RoleForge syntax or an API contract.
class Source(Directory):
    pass

class Tests(Directory):
    pass
```

This is not an approved feature. The analogy does not make Python classes the source of discovered Role identity and does not establish inheritance as a RoleForge mechanism.

Do not design or establish:

- Alias or named-instance syntax.
- Inheritance semantics.
- Registry behavior for aliases.
- Uniqueness rules.
- API exposure.
- Tokenizer representation for named instances.

> An undecided detail is not a decision.

## 26. Role interaction model

A Role may conceptually choose to expose behavior through DSL only, Python API only, or both. Core must not force every Role into one interaction or execution model.

The exact representation of this capability is not finalized. Do not automatically add metadata such as:

```text
interaction_mode = "dsl"
interaction_mode = "python_api"
interaction_mode = "hybrid"
```

Such metadata requires an actual designed contract before it is introduced. The conceptual range of possible interaction styles does not establish mandatory registration fields or Core branching on those styles.

This flexibility does not remove the source-discovery rule. How a Role exposes behavior and how its source identity is discovered are separate questions.

## 27. Avoid God Objects

No component should accumulate unrelated responsibilities merely because centralization is convenient.

| Component | Responsibility boundary |
| --- | --- |
| Core Runtime | Orchestrates Core; does not parse Role languages or control Role internals. |
| Registry | Resolves registrations; does not execute Roles. |
| Dispatcher | Determines routing outcomes; does not become a Role Runtime. |
| Main Tokenizer | Understands outer Core syntax; does not parse Role DSLs. |
| Error/event infrastructure | Handles structured information; does not automatically own all presentation. |
| Console/output infrastructure | Presents information; does not make Role decisions. |

Prefer explicit boundaries over convenient coupling. This does not require unnecessary layers or a separate abstraction for every helper operation.

## 28. Development methodology: local optimization

Performance is part of normal implementation quality. Avoid obvious unnecessary waste during implementation and review.

Prefer simple local improvements that clearly reduce unnecessary:

- I/O operations.
- Allocations and copies.
- Repeated parsing or computation.
- Intermediate data.

Make these improvements during normal development when they are simple, obvious, safe, local, easy to understand, and compatible with established architecture.

“Avoid premature optimization” is not permission to knowingly retain obvious waste. At the same time, do not introduce speculative architectural optimization such as:

- Complex caching.
- Concurrency solely for hypothetical performance.
- Unsafe code solely for hypothetical performance.
- Custom allocation strategies without demonstrated need.
- Performance-driven architectural redesign without evidence.

> Optimize obvious waste immediately. Postpone architectural optimization.

Performance improvements must not compromise clarity, isolation, maintainability, correctness, or established architectural boundaries. A local optimization is not justification for moving Role semantics into Core or discarding discovered identity and source metadata.

## 29. AI and contributor rules

An AI or contributor modifying RoleForge must:

1. Read and follow these Iron Rules before making architectural changes.
2. Preserve established component boundaries.
3. Avoid adding Role-specific knowledge to Core.
4. Avoid inventing future systems merely because they seem useful.
5. Distinguish current implementation from architectural contract.
6. Avoid converting temporary implementation details into permanent rules.
7. Avoid speculative abstractions.
8. Avoid speculative metadata.
9. Avoid speculative execution systems.
10. Preserve structured outputs instead of introducing direct presentation inside processing components.
11. Perform simple, safe local optimizations when obvious.
12. Postpone architectural optimization until there is demonstrated need.
13. Preserve source identity and metadata instead of unnecessarily reconstructing them later.
14. Preserve both global and Role-local instance identity.
15. Never resolve cross-registry conflicts using implicit precedence.
16. Never assume one Role declaration per Role name.
17. Never assume every Role has a parser.
18. Never assume every Role uses the same implementation language.
19. Never assume every Role uses the same execution model.
20. Never silently decide an unresolved architectural question.

When an implementation task appears to require changing an Iron Rule, stop and identify the conflict rather than silently redesigning the architecture. Distinguish a routine implementation choice that preserves the contract from a decision that changes the contract or settles an explicitly unresolved question.

> An undecided detail is not a decision.

## 30. Historical development stages

The currently completed Core development stages are recorded here for context:

| Stage | Development scope |
| --- | --- |
| Stage 01 | File Loader |
| Stage 02 | Main Tokenizer |
| Stage 03 | Role Registry and Dispatcher |

The Main Parser was intentionally removed from the Core architecture. It must not be restored by interpreting an old stage plan as an unfulfilled architectural requirement.

Historical prompt files are development records, not the architectural source of truth. Do not rewrite their contents merely because later decisions refined the architecture.

Small corrections to a completed stage do not automatically create a new numbered stage. Future stage numbering must follow explicitly agreed development planning, not assumptions inferred from historical prompts.

## 31. Explicitly undecided areas

**The following areas are intentionally not decided. Their presence here protects against accidental architecture invention; it is not a request to design them now.**

| Undecided area | Boundary that must be preserved meanwhile |
| --- | --- |
| Physical Role handoff mechanism | Core resolves and delivers; Role controls subsequent behavior. No ABI, callback, serialization, or process protocol is implied. |
| Cross-language execution mechanism | Implementation language is not part of Role identity or resolution. |
| Python/Rust bridge architecture for Roles | Do not infer a bridge design from the current implementation languages. |
| Final Project/Public API object model | Keep the conceptual engine/loaded-result/Role distinction and source-driven discovery. |
| Single-instance shorthand behavior | Do not assign meaning to unindexed access for one, multiple, or zero instances. |
| Named Role instance / alias system | The idea is not an approved feature. |
| Alias and named-instance syntax | Do not extend tokenizer syntax or define inheritance, uniqueness, registry behavior, or API exposure for it. |
| Final Role registration/install/remove API | Dynamic registration remains a capability; install, register, unregister, remove, and location-override APIs are not finalized. |
| Exact error severity model | Resolution outcomes do not settle severity; Unknown is not permanently classified. |
| Final Error Manager architecture | Preserve structured reporting without inventing the final manager design. |
| Final Console Manager architecture | Preserve dedicated presentation without merging every concern into one subsystem. |
| `interaction_mode` metadata | Conceptual interaction styles do not establish a metadata contract. |
| Advanced source mapping | Retain useful source metadata and newline structure without speculative mapping infrastructure. |
| Final packaging/runtime path-resolution strategy | Preserve registry-relative bases and absolute paths; do not freeze `CARGO_MANIFEST_DIR` as permanent packaging architecture. |

Current names, field layouts, and file arrangements may evolve while preserving the established contracts. Where this document does not establish a decision, examples and implementation convenience must not silently supply one.

> An undecided detail is not a decision.

## Short Version — Iron Rules Checklist

This is a quick pre-modification checklist for AI coding agents. It does not replace the full document or its explicit boundaries and undecided areas.

1. RoleForge is a framework for independent mini-languages, not one giant DSL.
2. Core knows the protocol, never the Roles.
3. The source determines what the Role is; the Role determines its API and behavior.
4. Role discovery happens before Role-specific behavior is exposed.
5. Main Tokenizer understands only Core outer syntax.
6. Role body content remains opaque to Core, except for explicitly established global Core syntax rules.
7. There is no Main Parser in the current Core architecture.
8. Every Role instance retains useful source metadata.
9. Every Role instance retains both a global index and a Role-local index.
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
