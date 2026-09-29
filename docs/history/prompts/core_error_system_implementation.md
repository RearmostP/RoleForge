# Comprehensive Core Error System — user request

Source: the user's implementation request in the Codex conversation, archived on
2026-09-27. The request body below is preserved from that message. This filename
is an archive label, not a newly assigned development stage. Actual implementation
and verification are recorded in [the implementation report](../../ai/ERROR_SYSTEM_IMPLEMENTATION.md).

---

Continuing from [פילוסופיית ספריית פייתון](chatgpt-conversation://6a9969b0-1394-83eb-b6ab-a181c7bd6ea5): Take full ownership of designing and implementing the RoleForge Core Error System in the local repository at `D:\David\Projects\Rust\RoleForge`.

The user does NOT want to manually review D001–D054 one by one. You should do the full engineering work yourself, using the current repository as the source of truth, while strictly respecting the established architecture and Error System Iron Rules.

Before changing anything:
1. Inspect the current git status and repository structure.
2. Read the active architecture docs, especially `docs/ai/CORE_IRON_RULES.md` and `docs/ai/ERROR_SYSTEM_IRON_RULES.md`.
3. Read the existing error/event inventory containing D001–D054 and inspect the corresponding current production code.
4. Preserve unrelated user changes. Do not overwrite or revert work that is not yours.

Then design and implement the Error System comprehensively.

Authoritative direction:
- Core knows the protocol, never the Roles.
- Components process; Runtime orchestrates.
- Components report structured errors/events; they do not present them.
- Core owns Core failures through delivery/handoff. Failures inside a Role after successful handoff belong to that Role, not to the Core Error System.
- Known meaningful Core failure cases get explicit, stable RoleForge-level names.
- Map the meaningful RoleForge-level failure CASE, not the low-level internal reason, sub-stage, or taxonomy of why it failed.
  Example: D001 should be `PackagePathResolutionFailed`; do NOT create ImportPathlib/ResolvePath/PathBufConversion stages merely to describe internal failure points.
- Do not spend effort preserving/mapping low-level causes solely for diagnostic completeness. Keep case-specific data only when it is meaningfully part of the error itself, e.g. a missing file path or relevant source line.
- Each error should own its appropriate message logic.
- Do not force all errors into one giant universal metadata structure with many optional fields.
- Keep error construction/reporting separate from console/output presentation.
- Provide a small explicit internal Core API rather than a God Object.
- D001–D054 are inventory observations, not public error codes and not necessarily one-to-one with final error types.
- Prefer explicit, maintainable definitions over speculative generic frameworks/macros/hierarchies.
- Do not add an 'errors off' mode or feature flags for removing the Error System.
- Cache and future final compilation are outside this task.
- Do not implement Role `start()`, aliases, install/remove, cache, compiler/final artifact, or other unrelated future features.

Very important decision boundary:
You are empowered to make normal implementation decisions yourself: module/file layout, Rust types, naming details consistent with existing conventions, conversions, internal helpers, tests, deduplication, and other engineering details that follow naturally from the established rules.

However, DO NOT silently make a new product/architecture POLICY decision when the existing behavior/docs do not already determine it. Examples include changing whether a condition is fatal vs nonfatal, whether one Role failure aborts all delivery vs only that Role, whether UnknownRole becomes a warning/error, or changing conflict continuation semantics. Preserve current behavior unless an existing authoritative decision clearly says otherwise. If a meaningful unresolved policy choice is genuinely required to continue correctly, stop at that point and ask the user one concise question in Hebrew instead of guessing.

Implementation expectations:
- Cover the meaningful Core-owned failures/conditions found by the inventory, consolidating low-level observations into sensible RoleForge-level cases where appropriate.
- Replace scattered/ad-hoc Core error construction with the new system where appropriate, without changing established runtime semantics.
- Remove direct Core presentation coupling where the Error System architecture requires separation, but do not invent a large Console Manager unless necessary and already justified. Keep presentation changes minimal and scoped.
- Preserve Python-facing exception behavior as much as possible unless the new Error System explicitly requires consistent formatting/conversion; do not casually break the public API.
- Keep Python Bridge boundaries clean: neutral Core data/errors should not become PyO3-specific merely for convenience.
- Update/add focused tests for the Error System and affected paths. Test behavior, messages where contractually meaningful, and preservation of existing continuation/stop policies.
- Update active AI architecture/implementation documentation as needed so it matches the final implementation. Do not rewrite historical prompts/reports merely to erase history.
- Follow the established test-code separation rule: test code may live beside the subsystem but must be physically separated from production implementation code.

Validation:
Run the relevant complete validation suite available in the repository, including at minimum Rust formatting/check/build/tests and Python tests/live flow where applicable. If packaging boundaries are affected, also verify the wheel remains clean and the external consumer flow still works. Do not claim a test passed unless you actually ran it.

External consumer project:
The user's external pip-installed experimentation project is at approximately `D:\David\Projects\Python\test_my_libraty`. Use it for validation if useful, but do not modify it unnecessarily. The RoleForge repository itself remains the implementation target.

At the end, provide a concise Hebrew report containing:
1. The architecture you implemented.
2. The final set/grouping of meaningful RoleForge error cases you introduced (do not dump irrelevant low-level inventory detail).
3. Important files changed/created.
4. Any existing behaviors intentionally preserved, especially Unknown/Conflict/Handoff continuation policies.
5. Tests/commands actually run and their results.
6. Any remaining unresolved issue or policy decision that genuinely requires the user's input.

Do not stop after analysis or produce only a plan. Implement the complete Error System unless you hit a genuine unresolved policy decision as defined above.

Respond to me in Hebrew.
