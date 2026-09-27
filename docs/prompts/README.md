# RoleForge development prompt history

RoleForge was built with AI-assisted coding. This archive preserves the prompts
used with Codex so contributors can follow the construction of the Core, the
decisions that changed along the way, and the later Error System work.

All archived prompts live directly in this one folder. Their original filenames,
stage numbers, wording, and historical code paths are retained. The original
numbered series and the D001 follow-up series are listed separately below;
matching numbers do not mean they belong to the same stage. Later conversation
requests use descriptive filenames and do not create new numbered stages.

These are historical requests, not instructions to rerun and not proof that every
requested detail was implemented. In particular, superseded error names,
internal-stage metadata, and old paths remain visible as part of the history.
For current behavior, use the code, tests, [Core Iron Rules](../ai/CORE_IRON_RULES.md),
[Error System Iron Rules](../ai/ERROR_SYSTEM_IRON_RULES.md), and implementation reports.
The adjacent [Error System rules reference](ERROR_SYSTEM_IRON_RULES.md) is a
current reference copy retained for the original prompts; it is not another
prompt or an independent authority.

## Core construction

| Original stage | Scope | Original prompt |
|---|---|---|
| 01 | File Loader | [01_file_loader.md](01_file_loader.md) |
| 02 | Main Tokenizer | [02_main_tokenizer.md](02_main_tokenizer.md) |
| 03 | Role Registry and Dispatcher | [03_role_registry_and_dispatcher.md](03_role_registry_and_dispatcher.md) |
| 04 | Core Runtime | [04_core_runtime.md](04_core_runtime.md) |
| 05 | First Python API | [05_first_python_api.md](05_first_python_api.md) |
| 06 | Conflict preflight and handoff | [06_role_handoff.md](06_role_handoff.md) |
| 07 | Core Bridge abstraction | [07_core_bridges.md](07_core_bridges.md) |
| 08 | Real Python receipt and neutral RoleInput | [08_python_role_handoff.md](08_python_role_handoff.md) |
| After 08, unnumbered | First real Test Role | [first_test.md](first_test.md) |
| 09 | Live Role objects and Project access | [09_live_role_objects.md](09_live_role_objects.md) |
| 10 | Separate tests and consumer example | [10_pre_publish_cleanup.md](10_pre_publish_cleanup.md) |
| 11 | Source/runtime layout and packaging | [11_runtime_source_layout_separation.md](11_runtime_source_layout_separation.md) |
| 12 | Diagnostic and error inventory | [12_diagnostic_event_inventory.md](12_diagnostic_event_inventory.md) |

## Error System follow-ups

The D001 sequence below is a separate prompt series. Prompt 03 supersedes the
earlier name/presentation/stage decisions in prompts 01 and 02. The later
comprehensive request expands the implementation across the Core inventory.

| Sequence | Request |
|---|---|
| D001 / 01 | [Missing package file](01_d001_missing_package_file.md) |
| D001 / 02 | [Earlier name and presentation proposal](02_package_path_error_presentation.md) |
| D001 / 03 | [Meaningful failure case and load-request context](03_d001_failure_case_and_load_context.md) |
| Later user request | [Comprehensive Core Error System implementation](core_error_system_implementation.md) |

## External validation and archive organization

These files preserve user requests from the current Codex conversation, with
their provenance stated above the original message. They were archived from the
conversation rather than reconstructed as new historical implementation prompts.

| Order | Request |
|---|---|
| After the complete Error System | [Simulation in the external pip-installed project](external_error_simulation.md) |
| After simulation and release discussion | [Consolidate this prompt archive](prompt_archive_consolidation.md) |

## Implementation evidence

Reports and current architecture stay under `docs/ai/`; user guides stay under
`docs/human/`. This folder is the single entry point for the prompt history.

- [Stage 08 report](../ai/STAGE_08_IMPLEMENTATION.md)
- [Stage 09 report](../ai/STAGE_09_IMPLEMENTATION.md)
- [Stage 10 report](../ai/STAGE_10_IMPLEMENTATION.md)
- [Stage 11 report](../ai/STAGE_11_IMPLEMENTATION.md)
- [Original diagnostic inventory](../ai/system_message_mapping/event_and_error_inventory.md)
- [D001 implementation checkpoint](../ai/D001_IMPLEMENTATION.md)
- [Complete Error System implementation and validation](../ai/ERROR_SYSTEM_IMPLEMENTATION.md)
- [Error metadata inventory](../ai/system_message_mapping/error_metadata_inventory.md)

The repository archive contains 16 pre-existing prompt files plus three user
requests preserved from the current conversation. It is not a transcript of
every conversation, tool call, or intermediate AI response.
