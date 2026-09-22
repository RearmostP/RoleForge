# RoleForge

RoleForge is a framework for independent, Role-specific languages in one `.rfg` source file. A Rust Core discovers Role instances, resolves their registered destinations, and delivers their data. Each Role decides how to interpret its own body and what behavior or API to provide.

**Status:** under development, not production-ready. Real dynamic Python Role delivery works through the complete pipeline. Python is currently the only implemented delivery Bridge; Rust Role delivery is not implemented.

## The core idea

```text
@role Test

hello = world
number = 123
```

With the package built from this checkout and `Test` registered, loading that source invokes its Python receiving function:

```python
from roleforge import load

project = load("test.rfg")
project.test.hello()
project.test[0].hello()  # Same live instance.
```

The Role implements `roleforge_receive(role)` and receives a live Python object with read-only name, body, indexes, and source metadata. An optional `Role` subclass defines methods; the receiver initializes per-instance state. The Python Bridge creates the object and Project retains it after delivery. Core does not interpret `hello = world`; that belongs to the Role. Receipt does not automatically call `start()`.

The repository includes a working [Test Role](roleforge/python/roleforge/roles/Test/main.py), its [registration](roleforge/python/roleforge/core/storage/dynamic_roles.json), and a [Python example](external_test_project/main.py) that delivers two instances from [test.rfg](external_test_project/test.rfg). See the setup instructions before running it.

## Documentation

| Language | Introduction and setup | Practical guide | Architectural principles |
| --- | --- | --- | --- |
| English | [Start here](docs/human/en/README.md) | [Creating Roles](docs/human/en/CREATING_ROLES.md) | [Iron Rules](docs/human/en/IRON_RULES.md) |
| עברית | [מבוא והתקנה](docs/human/he/README.md) | [יצירת Roles](docs/human/he/CREATING_ROLES.md) | [כללי הברזל](docs/human/he/IRON_RULES.md) |

## Repository map

- `roleforge/src/`: Rust implementation, Python bindings, and subsystem-local Rust tests.
- `roleforge/python/roleforge/`: installed Python runtime package and Registry resources.
- `roleforge/python/roleforge/roles/`: dynamic Role implementations, including the development Test Role.
- `external_test_project/`: simulated external Python consumer, with user examples and Python-facing tests; see its [README](external_test_project/README.md).
- `docs/human/`: documentation for users and contributors.
- [docs/ai/](docs/ai/CORE_IRON_RULES.md): detailed architectural guardrails for AI-assisted development.
- [Prompt history](docs/roleforge/core/pipeline/prompts/README.md): historical development records, not current usage documentation.

Internal Rust unit tests live in subsystem-local `tests.rs` modules compiled only for testing. The external test project is separate from the library runtime.

Registry files and relative Role targets resolve from the installed Python package. Maturin builds the Wheel from `roleforge/python/`; Rust source and historical prompts stay outside the runtime package. Live access supports `project.test`, `project.test[1]`, and exact-name `project.get_role("Test", 1)`. `project.roles` remains discovery metadata. Automatic Role installation and a finalized execution lifecycle remain open.

Build a release Wheel with `maturin build --release`, install the generated file from `target/wheels/` with `python -m pip install --force-reinstall <wheel-path>`, and run the consumer example from outside the repository. No editable install or repository `PYTHONPATH` is needed. See the [Stage 11 report](docs/ai/STAGE_11_IMPLEMENTATION.md).
