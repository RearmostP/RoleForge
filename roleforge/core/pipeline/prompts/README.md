# RoleForge development prompt history

[Project home](../../../../README.md) · [English documentation](../../../../docs/human/en/README.md) · [תיעוד בעברית](../../../../docs/human/he/README.md) · [Live AI Iron Rules](../../../../docs/ai/CORE_IRON_RULES.md)

This folder preserves the prompts used to develop RoleForge. They describe requests and decisions at particular points in time, not a current feature specification or user guide. Older prompts can describe architecture that later changed or ideas that were never implemented.

Use the live human documentation for current usage and the live Iron Rules for architectural constraints. Those and explicitly approved current decisions take precedence over historical prompts; the implementation determines what actually works today. Historical contents are retained without rewriting them to match newer decisions.

## Numbered stages, in order

| Stage | Prompt | Intended scope |
| --- | --- | --- |
| 01 | [File Loader](01_file_loader.md) | Read UTF-8 source into LoadedFile and preserve its path. |
| 02 | [Main Tokenizer](02_main_tokenizer.md) | Discover Role blocks and apply Core comment rules while keeping Role bodies opaque. |
| 03 | [Registry and Dispatcher](03_role_registry_and_dispatcher.md) | Preserve declaration lines, resolve built-in/dynamic registrations, and retain Unknown/Conflict results. |
| 04 | [Core Runtime](04_core_runtime.md) | Orchestrate the existing pipeline and provide temporary final-Core inspection output. |
| 05 | [First Python API](05_first_python_api.md) | Expose load and Project metadata through the Rust/Python boundary. Some conceptual API ideas in the prompt were not implemented. |
| 06 | [Role Handoff](06_role_handoff.md) | Establish complete conflict preflight and temporary Unknown/Conflict reporting before delivery. |
| 07 | [Core Bridges](07_core_bridges.md) | Introduce common Bridge registration and resolution, with temporary Python and Rust placeholders. |
| 08 | [Python Role Handoff](08_python_role_handoff.md) | Introduce RoleInput and entry { via, target }, connect real Python receipt, and remove the Rust placeholder. |

For example, the Rust Bridge in Stage 07 is historical: Stage 08 removed it. The old string Registry entries likewise do not describe today's structured entries. Read the [Creating Roles guide](../../../../docs/human/en/CREATING_ROLES.md) for current examples.

## Non-numbered work

[First real test](first_test.md) follows Stage 08. It requests a small dynamic Test Role, a user-side load example, and single/multiple-instance verification. It is intentionally **not Stage 09**.

Human documentation and repository organization are also non-numbered work. This index does not assign them a new stage or rename existing prompts.
