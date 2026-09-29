# First Core error implementation: D001, missing package __file__

Repository: D:\David\Projects\Rust\RoleForge

## Purpose and scope
Implement only the first concrete case of D001: reading __file__ from the imported roleforge package fails because the attribute is unavailable. D001 as a whole remains partially implemented. Do not implement import failures, __file__ = None, invalid paths, resolve failures, or other inventory items.

Read before editing:
- docs/ai/CORE_IRON_RULES.md
- docs/ai/ERROR_SYSTEM_IRON_RULES.md (canonical Error System rules)
- ERROR_SYSTEM_IRON_RULES.md beside this prompt (reference copy)
- docs/ai/system_message_mapping/event_and_error_inventory.md
- docs/ai/system_message_mapping/error_metadata_inventory.md
- roleforge/src/python_api.rs and relevant existing tests

Preserve unrelated working-tree changes. Do not commit. Do not modify or install into D:\David\Projects\Python\test_my_libraty or its virtual environment.

## Approved decisions
- Reporting category: RoleForge Error.
- Official error name: PackagePathResolutionFailed.
- First error-specific metadata: stage = ReadPackageFile.
- First supported condition: __file__ attribute unavailable on the imported roleforge module.
- Message meaning: Cannot locate the roleforge package directory because the __file__ attribute is missing.
- A missing attribute does NOT establish that a file is missing on disk. Do not invent a filesystem path or claim a file was not found.
- RoleForge Warning and RoleForge <RoleName> <Type> <Message> are future concepts only. Do not implement their infrastructure.

## Rust source layout
Create:
- roleforge/src/core/errors/mod.rs
- roleforge/src/core/errors/package_path_resolution_failed.rs

Register errors in roleforge/src/core/mod.rs.

mod.rs is the small internal access boundary. The dedicated file owns this error's structured definition, its stage and message construction. Keep detection and Python exception adaptation in python_api.rs. Do not move package resolution or runtime orchestration into errors.

Exact Rust signatures are an implementation choice, not an already approved universal API. Use a minimal concrete definition; do not build a generic report manager, metadata bag, registry, severity framework, macros or shared collection helpers. A function called report is not required. Do not predefine unused stages.

Category, error identity and stage must remain explicit structured information internally, not exist solely inside a formatted string. Fixed identifiers need not be caller-supplied metadata. Avoid redundant abstractions and keep neutral error definitions independent of Python objects.

## Detection and Python boundary
Inspect the existing expression:
    py.import("roleforge")?.getattr("__file__")?

Separate the import from attribute access so an import failure cannot accidentally be classified as ReadPackageFile.

Handle only the missing-attribute condition at the __file__ access. Preserve propagation of other exceptions. Python AttributeError from module attribute lookup is the relevant signal; do not claim to know why the attribute is unavailable, and do not add speculative introspection to investigate module internals.

A present __file__ with value None is outside this change and must continue through the existing path. Do not broaden this task into validating its value.

Use the dedicated error definition to construct the message. The Python boundary should propagate an exception with the new diagnostic while retaining the original Python exception as its cause. Keep the current AttributeError compatibility for this first case; no new public exception hierarchy is required. Document the exact resulting exception and message in the completion report. Do not discard the original cause by converting it only to text.

No printing or logging is added. Do not change existing temporary debug output or introduce a console subsystem. No recovery, fallback path or continued registry loading after this failure.

## Persistent mapping and statistics
Update docs/ai/system_message_mapping/error_metadata_inventory.md in Hebrew:
- Mark D001 as partially analyzed/implemented, with only this specific case completed after successful verification.
- Record category, official name, ReadPackageFile, the observable missing-attribute condition, and the diagnostic need for stage.
- Distinguish the observed condition from unknown underlying reasons. Do not invent causes.
- Record that stage comes from the reporting point and requires no additional collection.
- Record retained original exception context separately from the first error-specific metadata field; explain it already exists at the Python boundary and requires no new collection. If included in the cumulative table, label its role clearly as underlying cause context.
- Count PackagePathResolutionFailed once per applicable metadata item, regardless of stages or inventory rows.
- Leave all other D001 cases and other inventory items pending, not counted as zero.
- Keep statistics descriptive. Do not implement shared collection functions or derive an optimization plan yet.

Do not rewrite the historical inventory as though the entire D001 item has been migrated. If adding a current-status note, clearly separate it from historical observations.

## Documentation consistency
The prompt directory mirrors the implementation layout:
    docs/ai/roleforge/src/core/errors/

The canonical error rules remain docs/ai/ERROR_SYSTEM_IRON_RULES.md. The adjacent copy is for convenient reading and must not become a competing authority. If a minimal documentation update is necessary to record the now-established folder/category decisions, update the canonical document and refresh its reference copy, preserving the copy notice and relative link correction. Do not treat warnings or Role-message design as settled.

## Verification
Add focused regression coverage following existing test conventions:
1. Missing roleforge.__file__ yields the named error and ReadPackageFile diagnostic; original AttributeError is retained as cause; registry loading/handoff does not occur.
2. A different exception raised during __file__ lookup is propagated without being mislabeled as a missing attribute.
3. A present None value is not classified as this missing-attribute case.
4. Normal loading and live Test Role access still work.

Restore any manipulated module attributes/hooks in finally blocks or equivalent guards. Do not mutate the user's external test environment. Verify against a build of the changed code, not an old installed wheel. Use the repository's existing test/build workflow and report any environment blockers honestly. Run relevant existing regression tests; avoid unrelated formatting or broad refactors.

## Completion report
Report:
- Files changed and the small internal API chosen.
- Exact Python-facing diagnostic and how the original cause is retained.
- Tests executed and their results, including any skipped/blocked checks.
- Metadata inventory updates.
- Explicit confirmation that only the missing-__file__ case is implemented and the rest of D001 remains pending.
