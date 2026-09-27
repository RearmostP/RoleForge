# 02 — Rename PackagePathError and add shared category-based presentation

Repository: D:\David\Projects\Rust\RoleForge

## Goal
Rename the official error PackagePathResolutionFailed to PackagePathError and implement the agreed compact console presentation. Color selection belongs to a shared presentation layer and is driven by the reporting category, not repeated in each individual error definition.

Read the current code, tests, docs/ai/CORE_IRON_RULES.md, docs/ai/ERROR_SYSTEM_IRON_RULES.md, the adjacent reference copy, and docs/ai/system_message_mapping/error_metadata_inventory.md before editing. Preserve unrelated working-tree changes. Do not commit. Do not modify or install into the user's external test project D:\David\Projects\Python\test_my_libraty.

## 1. Official rename
- Rename the Rust definition to PackagePathError and its file to roleforge/src/core/errors/package_path_error.rs.
- Update module declarations, imports, tests, current architectural documentation and metadata mapping/statistics.
- This is the same error, still associated with D001. Do not count the rename as a second error type.
- Keep stage ReadPackageFile and the reporting category RoleForge Error.
- Preserve historical prompts, including 01_d001_missing_package_file.md, as records of earlier instructions. Do not globally rewrite history. Explain the rename in current documentation.
- Refresh the adjacent reference copy if canonical Iron Rules change, preserving its canonical-source notice and relative links.

## 2. Exact presentation
Plain-text layout, with no extra heading or blank line between its two lines:

[Error] PackagePathError (ReadPackageFile)
  Cannot locate the roleforge package directory: __file__ is missing.

Colored layout:
- [Error] PackagePathError: red (ANSI 31).
- A single space, followed by (ReadPackageFile): gray (ANSI 90).
- Reset styles at the end of the heading.
- Second line: two spaces of indentation followed by the explanation in the terminal's default color.
- No bold, extra frame, icons, duplicate category prefix or separate Stage line.
- Color must not leak into following output.

The category remains RoleForge Error internally; [Error] is its presentation label.

## 3. Shared presentation responsibility
Move category ownership out of the individual package-path error file if necessary so other future Core errors can use the same existing category and presentation rule.

The presentation layer maps the Error category to a red heading. It receives explicit structured category, official name, stage and explanation. Do not infer these by parsing formatted strings, matching the error name, or treating every Python AttributeError as a RoleForge error.

Individual error definitions own their explanation and error-specific data. They must not embed ANSI escapes, print, select colors or duplicate the shared layout. Introduce only the small concrete interface needed for this error and this presentation. A small presentation contract is not permission to add a universal optional-field metadata bag.

Keep neutral Rust error definitions independent of Python objects. Choose a clearly separate presentation module consistent with the repository. Document the chosen boundary briefly.

Do not implement Warning or Role-specific message categories yet, a generic logging framework, event bus, plugin system, manager singleton, automatic metadata collectors or a redesign of existing final_core_debug output.

## 4. Exceptions versus console display
Preserve the existing failure behavior: only AttributeError from reading the imported roleforge module's __file__ is classified here. Import failures, other lookup exceptions and a present None value retain their existing paths. D001 remains partially implemented.

Keep AttributeError compatibility and preserve the original Python exception as __cause__. The original cause is diagnostic information; do not discard it just to simplify normal output.

Keep exception text free of ANSI control sequences. Provide a small explicitly invoked presentation entry point accessible to a Python consumer so the agreed colored output can be demonstrated when catching this RoleForge error. Its exact name/signature is an implementation choice; document it and give a complete usage example. Provide an explicit plain-text/color-off option as well as a deterministic color-on option for the console preview. Preserve machine-readable diagnostic fields through this boundary rather than reconstructing them from str(error).

Normal presentation through this entry point shows only the agreed two lines, not the original Python cause or a traceback. This is not a request to globally suppress Python tracebacks: an uncaught exception may still use normal Python rendering. Do not install sys.excepthook, swallow exceptions, print automatically inside load(), or change global terminal settings.

The earlier test script explicitly printed Original cause; the new documented example should catch the error and use the presentation entry point instead. Unknown/non-RoleForge exceptions must not silently be relabeled or suppressed; document the entry point's handling of them.

## 5. Persistent documentation
Update docs/ai/system_message_mapping/error_metadata_inventory.md:
- Official name PackagePathError, inventory ID D001, category RoleForge Error, stage ReadPackageFile.
- Preserve partial implementation status; other D001 cases remain pending.
- Preserve the existing distinct-error counts: this rename does not increase them.
- Distinguish category/name presentation identity from error-specific stage metadata and retained cause context.
- Color and formatting are presentation policy, not newly collected diagnostic metadata. Do not invent collection requirements or statistics.

Update current usage documentation with a runnable example and the exact public presentation API. Mark prompt 02 in the central prompt index without claiming completion until verified.

## 6. Verification
Adapt existing tests and add focused coverage showing:
- The official name changed and category/stage remain correct and structured.
- The original cause is still retained and exceptions contain no ANSI sequences.
- Color-on formatting has the exact agreed layout and style resets.
- Color-off formatting has the same two-line content without ANSI escapes.
- Red is selected from the category in shared presentation code, not hardcoded per error.
- Ordinary Python errors are not misclassified as RoleForge errors.
- Present None and unrelated lookup failures preserve prior behavior.
- Normal loading and live Test Role access still work.

Use the project's test/build workflow and verify against the freshly built extension, not a stale installed wheel. Do not install into the user's external test environment. If verifying native stdout behavior, Python redirect_stdout alone does not capture Rust stdout; use appropriate capture or directly test the relevant boundary. Do not claim that such a capture proves no registry access or handoff unless it actually does.

Run relevant regression checks and report failures/blockers honestly. Avoid unrelated refactors or mass formatting.

## Completion report
Include files changed, chosen presentation API and module, tests run/results, a copyable Python example displaying the actual new error in color, and confirmation that only the existing missing-__file__ case is handled. Note that the user must rebuild/reinstall the wheel before using the new API in the external project. Do not execute later stages or implement other D001 cases.
