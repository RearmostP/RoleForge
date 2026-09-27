# RoleForge Core Error System — Iron Rules

## 1. Purpose and authority

This document defines the architectural constraints for the RoleForge Core Error System. It is a long-lived architecture reference, not an implementation report or a complete inventory of current failures.

Read it using three categories:

- **Rules:** boundaries and invariants future Error System work must preserve.
- **Current context:** examples drawn from the current implementation and inventory; these describe evidence, not permanent API requirements.
- **Undecided areas:** details that have not been approved and must remain open.

> An undecided detail is not a decision.

These rules complement [Core Iron Rules](CORE_IRON_RULES.md), especially its Core responsibility boundary and its rules for structured reporting and output presentation. If future work appears to require changing an established architectural rule, identify the conflict explicitly. Do not silently reinterpret either document.

## 2. The Error System belongs to Core

The Error System defined here owns failures and reportable conditions arising from RoleForge Core responsibilities, including:

- Loader
- Tokenizer
- Registry
- Dispatcher
- Handoff
- Bridges
- Core-owned API boundaries

Core owns failures related to preparing and delivering data or control to a Role.

Once handoff is complete and the Role owns execution, failures of Role-internal behavior belong to that Role. The Core Error System must not become a global error manager for Role implementations.

```text
RoleForge Core
├── Loader
├── Tokenizer
├── Registry
├── Dispatcher
├── Handoff
├── Bridges
└── Core Error System

Role
└── Role-internal behavior and failures
```

This boundary follows the Core rule: the Core resolves and delivers; the Role decides what happens next.

## 3. Known failures have explicit official names

A known Core failure must have a clear, stable, explicit name that describes what failed from RoleForge's perspective.

For example:

```text
PackagePathResolutionFailed
```

Do not represent a known operation failure only as an arbitrary free-form call such as:

```text
error("something went wrong")
```

The official name identifies the logical RoleForge failure. It is not a numeric error code.

The first implemented error is in the `RoleForge Error` reporting category and lives under `roleforge/src/core/errors/`. This records the initial implementation choice; it does not establish a severity level or a warning/event model.

## 4. Map the meaningful failure case only

The Error System maps what failed at the RoleForge level, not the internal cause, sub-stage, or low-level reason. Several internal operations may lead to the same named error without becoming separately modeled failure points.

For D001, the error is simply:

```text
PackagePathResolutionFailed
```

Do not model `ImportPathlib`, `ResolvePath`, `PathBufConversion`, or similar internal steps as error variants, fields, or a failure-point taxonomy.

## 5. Case-specific data must describe the error itself

Keep structured data when it is part of the meaningful error itself. For example, `FileNotFound` may carry the relevant path. Such data must not exist only inside an arbitrary message string.

Meaningful operation context may identify the user request affected by a failure. For D001, the Python load boundary retains `requested_file` from the supplied load path, without resolving or probing it. This is the requested source input, not the unresolved package directory. Python supplies the caller location through its normal traceback; do not collect caller metadata manually.

Do not collect metadata merely to explain why or where an operation failed internally. A `stage` field or equivalent internal-location detail is not part of the Error System design.

## 6. Each error owns focused message behavior

Each known error has its own focused definition or function responsible for its message behavior. It receives only information relevant to that error.

Conceptual examples:

```text
package_path_resolution_failed()
file_not_found(path)
missing_role_name(line)
unknown_bridge(identifier)
```

These examples do not settle Rust types, APIs, or signatures. Do not create one giant generic function that receives every possible field for every error.

## 7. Error-specific metadata

Different errors need different information. For example:

```text
FileNotFound
    path

MissingRoleName
    line

PackagePathResolutionFailed
    requested_file (load-request context owned by the Python boundary)
```

Do not force every error into a universal optional-field structure. Document any case-specific data contract beside the error definition, and add data only when the meaningful error itself requires it.

## 8. Grow from actual case requirements

The first implementation of an error need not predict future requirements. Start with a small, explicit error and extend it only when a concrete requirement for the meaningful failure case appears.

Do not build a generic metadata framework or collect internal diagnostic details for possible future use.

## 9. Construction and presentation are separate responsibilities

An error definition constructs or reports structured error information. It does not print itself to the console.

```text
Core component
      ↓
Error System
      ↓
Structured error
      ↓
Internal API / boundary
      ↓
Presentation layer
```

Do not couple individual errors directly to `println!`, `eprintln!`, stdout, or stderr. This follows the established Core rule that components report structured events/errors and do not present them. Console output belongs to the dedicated console/output responsibility; this document does not design that layer.

The Error System defines the error. It does not decide Role behavior or final rendering.

## 10. Do not preserve low-level causes in the Error System

A Core failure may originate in I/O, Python, a Bridge, the operating system, or another lower-level operation. Map it to the meaningful RoleForge-level failure case without storing the original exception, cause chain, or low-level diagnostic context as part of the error.

Case-specific data remains appropriate under sections 5 and 7; metadata whose only purpose is to explain the internal cause does not.

Current compatibility detail: file/output failures retain only `io::ErrorKind`
to preserve existing Python I/O exception classes (such as `PermissionError`
and `BrokenPipeError`). This value is not an additional failure case or message
detail. No original I/O error, OS text, parser diagnostic, or cause chain is kept.
Do not extend this compatibility detail into a taxonomy of internal causes.

## 11. Provide a small internal Core API

Core components report known failures through an internal Error System API. They should not need to know how the final message will be presented.

```text
Loader ──────────┐
Tokenizer ───────┤
Registry ────────┤
Dispatcher ──────┤
Handoff ─────────┤──> Error System API ──> Error definitions
Bridge ──────────┘
```

Keep this boundary small and explicit. Its existence does not require a large manager object, singleton, service architecture, or other speculative abstraction.

The current internal API consists of explicit error types exported from
`roleforge/src/core/errors/mod.rs`, focused `Display` implementations, and a
synchronous `CoreEvent` reporter supplied to Runtime. It has no manager object.
Python exception conversion belongs to the Python boundary; the neutral error
definitions contain no PyO3 values. See [current implementation](ERROR_SYSTEM_IMPLEMENTATION.md).

## 12. Design errors incrementally from the inventory

The current error/event inventory contains observations D001–D054. These IDs exist only to identify inventory entries. They are not RoleForge error codes and do not prescribe the future architecture.

For each relevant item considered for implementation:

1. Inspect the actual current code.
2. Determine the logical RoleForge error, if any, represented by the observation.
3. Give that error a clear official name.
4. Identify only case-specific data that is part of the meaningful error itself.
5. Define its message behavior.
6. Integrate it without redesigning unrelated errors.

D001, `PackagePathResolutionFailed`, was the first implemented case. The
subsequently authorized comprehensive implementation covers the meaningful
Core-owned cases across the inventory. Continue to inspect actual production
behavior per case; coverage does not require an error type for every observation.
D001 remains one case without an internal failure-point taxonomy.

## 13. Inventory observations are not error types

D001–D054 describe observations in the current implementation. An item may describe:

- A failure.
- A detection point.
- Propagation or conversion.
- A continuation-policy decision.
- Presentation.
- Fallback behavior.
- A nonfatal condition.

Therefore:

```text
one D-item does not necessarily equal one Error type
one Error type may represent the same meaningful failure case across multiple observations
```

The inventory is evidence for designing the Error System. It is not itself the architecture of that system.

## 14. Keep the Error System focused

The Error System must not gradually take ownership of unrelated responsibilities, including:

- Console presentation.
- Role behavior.
- Runtime orchestration.
- Arbitrary debugging utilities.
- Every subsystem's recovery policy.
- Unrelated logging behavior.

Do not create an `ErrorManager` god object to centralize everything associated with failures. Keep detection, continuation policy, error definition, and presentation within their approved responsibility boundaries.

## 15. No Error System disable mode

Do not design the Error System around being manually disabled or compiled out.

### Normal source processing

```text
.rfg
 ↓
Core
 ↓
Roles
```

The Core Error System is required while Core operates.

### Cached processing

```text
Cache
 ↓
Core / Runtime
 ↓
Roles
```

Cache is intended to reduce repeated work. The Core Error System remains required wherever Core operates.

### Future final compilation

```text
Final compiled instructions
 ↓
Minimal required runtime
 ↓
Roles
```

If a future final compiled artifact does not need the normal source-processing Core pipeline, that machinery need not be part of that runtime. This is an architectural separation question, not an Error System enable/disable requirement.

Do not introduce Error System feature flags, “errors off” modes, debug/release switches that remove error handling, or conditional compilation whose purpose is to remove the Error System. Cache and final compilation are separate concepts and are outside this document's design scope.

## 16. Prefer explicit definitions to premature generalization

Build the Error System from known, concrete Core failures. Prefer explicit definitions over speculative generic abstractions. A small amount of repetition is acceptable when it makes each error clear and independently understandable.

Do not prematurely introduce:

- Macro-heavy error generation.
- Generic metadata containers.
- Dynamic string-based error registration.
- Complex error hierarchies.
- Error plugin systems.
- Speculative severity systems.
- Large categorization frameworks.

An abstraction may be introduced later if repeated, real patterns demonstrate its value.

## 17. Deliberately undecided details

This document does not decide:

- Types and API extensions for future, not-yet-observed failure cases.
- A severity model.
- A warning/event model.
- A final console rendering policy beyond the preserved current output format.
- Numeric error codes.
- Cache format.
- Final compilation format.
- Final minimal-runtime architecture.

Do not silently turn any of these open questions into a decision.

> An undecided detail is not a decision.
