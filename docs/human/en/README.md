# Understanding RoleForge

[עברית](../he/README.md) · [Project home](../../../README.md) · [Creating Roles](CREATING_ROLES.md) · [Iron Rules](IRON_RULES.md)

RoleForge hosts multiple independent Roles in a common source format. It is not one large DSL: a Role may define its own syntax, tokenizer, parser, validation, behavior, and API when it needs them. A Role that only prints its input is equally valid.

The Rust Core understands Role boundaries and routing, while the Role understands its own language. For example, Core can discover a `Test` block without assigning meaning to `hello = first` inside it. The project is under development and is not presented as production-ready.

## How data reaches a Role

```text
.rfg source
    → Core: Loader and Main Tokenizer discover Roles
    → Registry lookup and Dispatcher produce routing results
    → Handoff preflight checks the whole result collection
    → Handoff creates RoleInput and resolves entry.via
    → selected Bridge delivers to entry.target
    → Role implementation: roleforge_receive(role)
```

Runtime orchestrates these operations. Components do not directly drive the next major component. There is no Main Parser: any parsing of a Role's language belongs to that Role.

Python delivery is implemented in Rust using the same PyO3 integration as the public Python API. Core models remain ordinary Rust data; the Python Bridge exposes a corresponding read-only object. Core's delivery responsibility ends when the receiver returns successfully. Its return value has no result-protocol meaning.

## Run the existing example

Use a source checkout, Python 3.10 or newer, Rust/Cargo supporting edition 2024, and the native compiler/linker required by your Rust toolchain. The build uses Maturin through `pyproject.toml`; pip may download build dependencies.

From the repository root, create a virtual environment:

```sh
python -m venv .venv
```

Activate it in PowerShell:

```powershell
.\.venv\Scripts\Activate.ps1
```

Or in a POSIX shell:

```sh
. .venv/bin/activate
```

Then build/install this checkout and run the example:

```sh
python -m pip install -e .
python -u anyone_py_project/main.py
```

The [example](../../../anyone_py_project/main.py) locates its `.rfg` file beside the script. It does not import the Test Role manually. The [dynamic Registry](../../../roleforge/core/storage/dynamic_roles.json) already registers `Test` with `via: "python"` and `target: "Test/main.py"`.

The receiver prints two instances: indexes `0/0` and `1/1`, declaration lines `1` and `5`, and bodies containing `hello = first` and `hello = second`. Temporary Core debug output is also printed. This is a receipt demonstration, not a parser or `start()` demonstration.

If `import roleforge` fails, use the same Python environment in which you installed the checkout. An unbuilt checkout alone does not provide the native extension. Rebuild after Rust changes; rebuild if the checkout moves, because Registry paths currently depend on its build-time location.

## What load() returns

```python
from roleforge import load

project = load("anyone_py_project/test.rfg")  # From the repository root.
for info in project.roles:
    print(info.name, info.index, info.role_index, info.status)
```

`Project` has read-only `path` and `roles` attributes. `roles` is a tuple of read-only `RoleInfo` records in source order. Each record exposes `name`, `index`, `role_index`, `body`, `declaration_line`, `status`, `entry`, `builtin_entry`, and `dynamic_entry`.

These records describe discovery and routing. They are not executable Role instances and are distinct from the `RoleInput` object passed to a receiver: source information there is nested under `role.source`. `entry` is the resolved target path for a resolved record; conflicting records expose both paths instead. Inapplicable paths are `None`.

## Routing and failures today

| Situation | Current load behavior |
| --- | --- |
| Name occurs in exactly one Registry | Eligible for delivery in source order. |
| Name is in neither Registry | Report Unknown and skip it; other resolved instances can continue. |
| Name is in both Registries | Report all conflicts and deliver nothing in this load; return Project metadata. No Registry has precedence. |
| Unknown Bridge, target load failure, missing/non-callable receiver, or receiver exception | Raise `RuntimeError` with context; stop further delivery. Earlier successful calls are not rolled back. |
| Invalid outer syntax | Raise `ValueError` with source-line context. |
| Source/Registry read or decode failure | Raise an appropriate Python I/O error. |

`status == "resolved"` describes routing, not a delivery receipt: a conflict elsewhere can prevent every delivery. Unknown/Conflict reporting and debug output are temporary policy, not the final Error or Console Manager.

## Current implementation and open work

Implemented: Rust Loader, Main Tokenizer, Registry, Dispatcher, Runtime, preflight, Handoff, Bridge resolver, neutral RoleInput, Python Bridge, Python `load()`, source metadata, and global/per-name indexes. The dynamic Test Role exercises real Python delivery.

Not implemented or not finalized: Rust Role delivery, other Bridges, a Core-managed `start()` lifecycle, dynamic `project.directory` or `project.test[1]` APIs, aliases/named instances, Role install/remove APIs, and final Error/Console Managers. Additional Bridges are possible architecturally; their mechanisms are not promises or settled designs.

The built-in Role Registry is currently empty. `roleforge/builtin_roles/` reserves the built-in target base; `Test` is a dynamic development Role. A built-in Bridge and a built-in Role are different concepts.

Registry storage and target bases are currently tied to the source tree via Rust's build-time `CARGO_MANIFEST_DIR`. Portable distribution/runtime path rules remain open; this guide describes checkout-based development.

## Contributor checks and documentation

After installing the checkout, run these sequentially from the root:

```sh
cargo test
python -m unittest discover -s anyone_py_project -v
```

The Python handoff tests temporarily replace source-tree Registry fixtures and restore their original bytes. Do not run them concurrently with other loads or tests using those files.

Read the [human Iron Rules](IRON_RULES.md) before changing architecture. The [AI Iron Rules](../../ai/CORE_IRON_RULES.md) provide deeper constraints. The [Stage 08 report](../../ai/STAGE_08_IMPLEMENTATION.md) records that implementation checkpoint, while the [prompt index](../../../roleforge/core/pipeline/prompts/README.md) explains the historical sequence. Historical prompts do not override current documentation or approved architectural rules.
