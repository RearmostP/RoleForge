# 03 — D001: failure case and load-request context

Repository: D:\David\Projects\Rust\RoleForge
Status: implementation prompt, not executed.
Suggested location: docs/ai/roleforge/src/core/errors/03_d001_failure_case_and_load_context.md

## Decision and rationale

D001 represents one meaningful failure: RoleForge could not resolve its package directory. Keep the official name PackagePathResolutionFailed and category RoleForge Error.

The user decided to map the meaningful failure case only. Internal stages and low-level causes require unnecessary mapping and maintenance. Do not model ImportPathlib, ReadPackageFile, ResolvePath, PathBufConversion, or equivalent internal reasons.

A developer must still know which load request failed and where they invoked it. Preserve the requested file as meaningful operation context. Python's normal traceback supplies the caller's source file and line. This identifies the affected user operation without explaining the internal cause.

D001 happens before reading the requested source file. Do not imply that the requested file is missing or invalid. It is the input to the failed load request, not the unresolved package directory.

This decision supersedes conflicting instructions in historical prompts 01 and 02, including the PackagePathError rename, stage/cause retention, and missing-__file__-only scope. Preserve those prompts as history. Do not implement prompt 02's additional color or presentation API work in this task.

## Preparation and scope

Read applicable repository instructions, docs/ai/CORE_IRON_RULES.md, the canonical docs/ai/ERROR_SYSTEM_IRON_RULES.md, D001 in the event/error inventory, the metadata inventory, current Core error definitions, python_api.rs, and relevant tests. The canonical rules take precedence over any stale adjacent reference copy.

Preserve unrelated working-tree changes. Do not commit or modify/install into the external Python test project. Implement D001 only; do not proceed to D002 or design metadata for other errors.

## Required implementation

1. Keep one explicit Core error: PackagePathResolutionFailed. It owns the explanation `Failed to resolve the RoleForge package path.` Keep it independent of Python objects, ANSI codes, and console output.
2. Remove stage enums, stage-specific constructors, internal-location fields, and original low-level cause retention for this error. Do not replace them with a different internal taxonomy.
3. Treat package-path discovery as one bounded operation: package/pathlib imports, __file__ lookup, Path construction, resolve, parent, and PathBuf extraction. Map ordinary failures returned within this operation to the same error. Keep Registry::load and subsequent work outside this boundary. Do not swallow or relabel process-control exceptions such as KeyboardInterrupt and SystemExit.
4. Preserve the requested input path already available at load() as structured load-request context, suggested name `requested_file`. Retain the supplied path without extra filesystem resolution or probing. Keep this context distinct from internal package-resolution details. Use the smallest concrete representation; do not introduce a universal metadata bag or automatic collector.
5. Expose the error identity and requested_file through the Python boundary without requiring parsing of message text. Inspect the existing exception contract and choose a minimal compatible representation. Document the exception type and attributes. Do not identify RoleForge errors solely by a built-in exception class or introduce a broad hierarchy.
6. Remove explicit set_cause retention for D001 and avoid accidental exposure of the discarded internal exception through chaining. Preserve the translated exception's normal Python traceback and propagation so it includes the user's load call. Do not capture caller source metadata manually, install global hooks, swallow exceptions, or print automatically from load().
7. Keep construction separate from presentation. The readable exception should include the focused explanation and requested-file context through the appropriate boundary/presentation responsibility. Individual errors own their message logic. Do not create a giant generic message function, color system, or new console API.

Conceptual output; actual traceback formatting and exception-class prefix are controlled by Python:

```text
File "main.py", line 18
    roleforge.load("app.rfg")

RoleForge Error: PackagePathResolutionFailed
Failed to resolve the RoleForge package path.
Requested file: app.rfg
```

A consumer that catches the exception can choose whether to show its traceback. The short message alone does not promise the caller's source location.

## Documentation updates

- Update docs/ai/system_message_mapping/error_metadata_inventory.md: remove obsolete D001 stage/cause requirements and counts; record requested_file as load-request context, its source at load(), and that no additional collection is needed.
- Distinguish fixed identity/category from context. Count this as one error type. D001 is an inventory identifier, not an error code. Leave other inventory items pending.
- Clarify the canonical Error System Iron Rules if needed to permit meaningful load-request context while continuing to prohibit internal cause/stage metadata. Preserve all other established rules.
- Refresh the adjacent reference copy consistently, preserving its canonical-source notice and links. Do not rewrite historical prompts.
- Add this prompt to the current prompt index. Report actual verified status; a prompt's presence does not mean it was implemented.

## Verification

Use the repository's build/test workflow and a freshly built extension, not a stale installed wheel. Add focused behavior tests showing:

- Representative failures across package-path discovery produce the same official error and explanation, with requested_file and without stage or retained low-level cause.
- The propagated Python exception's traceback includes an actual caller's source location without synthesized metadata.
- Different requested files remain distinguishable, including a relative path supplied through a variable.
- Registry/later failures are not relabeled as D001; ordinary successful loading still works.
- Process-control exceptions propagate without being relabeled.
- No automatic printing or ANSI sequences are introduced.

Do not recreate a taxonomy of hypothetical failure reasons in the tests. Avoid dependence on exact Python traceback formatting or native exception wording. Report any verification blockers honestly.

## Completion report

List changed files, final error definition, Python exception type and structured context contract, where load-request context is owned, test results, and a copyable Python example demonstrating requested-file context and the normal traceback. State precisely what was verified and whether any D001 work remains incomplete. Stop before other inventory cases.
