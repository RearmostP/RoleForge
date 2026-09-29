# Creating a Role — instructions for AI implementation agents

## 1. Scope and authority

Use this guide when implementing a new Role or extending an existing Role. It describes the implemented Python delivery contract and separates facilities supplied by RoleForge from work owned by the Role developer.

Read [Core Iron Rules](CORE_IRON_RULES.md) and [Error System Iron Rules](ERROR_SYSTEM_IRON_RULES.md) before implementation. They remain the architectural authority. Consult [Error System implementation](ERROR_SYSTEM_IMPLEMENTATION.md) for current failure behavior. Historical prompts are development records, not missing requirements to implement.

Python is currently the only implemented Role delivery environment. A normal new Python Role requires a target file, a Registry entry, source examples, and tests. It does not require adding Role-specific logic to Rust Core.

## 2. Establish the Role's contract before writing it

Derive these decisions from the user's request and existing Role conventions. Ask only about missing decisions that materially affect behavior; record any reasonable assumptions.

| Decision | What the agent must establish |
| --- | --- |
| Purpose | The domain responsibility and expected results of the Role. |
| Source name | The exact name used in `@role Name` and in the Registry. |
| Body language | Whether the body is raw text, structured data, or a custom language; its syntax and invalid-input behavior. |
| Public API | Methods/properties callers will use, arguments, return values, and mutations. |
| Timing | Work performed during receipt versus work explicitly requested through methods later. |
| State | Per-instance data, initialization, and any deliberately shared external resources. |
| Effects | Files, network operations, output, or other effects required by the Role's purpose. |
| Dependencies | Python packages/resources required and how they become available in the consumer environment. |
| Failures | Domain errors, when they occur, and what callers can catch or inspect. |

A parser, AST, `start()` method, execution engine, or validation phase is optional. Add one only when the requested Role needs it. Do not infer them from the word “Role.”

## 3. Responsibility matrix

| Area | RoleForge already provides | Developer must implement or decide |
| --- | --- | --- |
| Loading | UTF-8 source loading through `roleforge.load(path)`. | Source files/examples and any Role-owned external input. |
| Discovery | Outer `@role` boundaries, Core comment processing, discovered names and source order. | Meaning of the body after Core processing. |
| Identity | Global `index`, per-name `role_index`, declaration line. | Domain identifiers if required, without replacing Core identity. |
| Resolution | Built-in/dynamic Registry lookup, conflict detection, relative target bases. | One appropriate Registry entry pointing to the implementation. |
| Delivery | Python target loading, input adaptation, object construction, synchronous receiver invocation. | A callable `roleforge_receive(role)` and optional `Role` subclass. |
| Input | Base-class properties exposing read-only input and source metadata. | Domain state and derived representations; no duplicate input-copying layer. |
| Live API | Project retains delivered objects and supports name/index access. | The actual methods and properties on those objects. |
| Multiple occurrences | Independent live instances delivered in source order. | Per-instance state and behavior that works for repeated declarations. |
| Errors | Named Core failures, Python adaptation, structured runtime events and current output policy. | Role-domain validation and exceptions after successful receipt. |
| Packaging | Existing Maturin packaging of the Python runtime and native extension. | Include Role files/resources, provide dependencies, and verify an installed build. |
| Lifecycle | Receipt and retention of the live object. | Explicit domain operations and resource cleanup when needed; Core supplies no automatic start/shutdown protocol. |

Core does not parse a Role's language, infer its API from module functions, execute its user-facing methods, or manage its internal lifecycle.

## 4. Files and registration

For a new dynamic Role called `Echo`, use this layout in the checkout:

```text
roleforge/python/roleforge/
├── roles/
│   └── Echo/
│       └── main.py
└── core/storage/
    └── dynamic_roles.json
```

Add this entry to the existing JSON object in `dynamic_roles.json`:

```json
{
  "Echo": {
    "entry": {
      "via": "python",
      "target": "Echo/main.py"
    }
  }
}
```

This JSON demonstrates one entry, not permission to replace the whole Registry. Preserve `Test` and all unrelated registrations. Check `builtin_roles.json` as well: a name present in both registries is a conflict, with no implicit precedence.

`via` is the exact Bridge identifier; currently use `python`. The `.py` extension does not select a Bridge. Relative dynamic targets resolve under the imported package's `roles/`; relative built-in targets resolve under `builtin_roles/`. Absolute targets remain absolute. Targets are not resolved beside the `.rfg` file or relative to the caller's working directory.

At runtime, Registry storage and relative target bases belong to the imported installed `roleforge` package. Editing the checkout alone does not update an already installed wheel. Manual installed-package edits can be overwritten by reinstalling. There is no public Role installation command, registration API, automatic discovery, or dependency installer.

## 5. Implement the Python entry point

Place this complete minimal example in `roles/Echo/main.py`:

```python
from roleforge import Role as BaseRole


class Role(BaseRole):
    def text(self):
        self.calls += 1
        return self.body


def roleforge_receive(role):
    role.calls = 0
```

The target module's optional class must be named `Role` and subclass `roleforge.Role`. Import the base under an alias so the subclass declaration is unambiguous. Prefer the inherited constructor; it accepts the adapted input and establishes the base object. If a custom constructor is necessary, preserve that one-input contract and call `super().__init__(role_input)`.

The required entry point is an ordinary synchronous `roleforge_receive(role)` callable. Core does not try alternative names. Do not use `async def`: the Bridge does not await it. Returning normally means receipt succeeded; the return value is ignored. Attach state to the supplied object rather than returning a replacement object.

The Bridge constructs the live object before invoking the receiver. Initialize per-instance state in the receiver. If there is no target-module `Role` class, the receiver gets the generic base Role; this supports simple receipt without custom methods. Defining unrelated module functions does not expose them as methods on `project.echo`.

`Echo` deliberately keeps its body as text. For a structured Role, implement its parser/validation inside the Role or its own dependencies. Decide whether parsing happens during receipt or in an explicit method, taking the different failure boundaries below into account.

## 6. Input contract and source boundaries

| Live-object field | Meaning |
| --- | --- |
| `role.name` | Exact name discovered from the declaration. |
| `role.index` | Zero-based position among all discovered Roles in that source. |
| `role.role_index` | Zero-based occurrence among Roles with the same exact name. |
| `role.body` | Text after Core-level comment processing, without interpretation of the Role's language. |
| `role.source.declaration_line` | One-based declaration line in the original source. |
| `role.role_input` | The frozen adapted input underlying the base properties. |

Input-derived properties and nested source metadata are read-only in the supplied representation. Do not overwrite them, replace `_input`/`_instances`, or shadow the base identity properties. Put mutable domain state under separate attributes. Custom representations can be stored there too.

The current RoleInput has no source filename, Project reference, body AST, complete source map, or automatic cross-Role dependency container. `project.path` belongs to the public Project. If the Role needs additional context, establish an explicit Role API or separately approved contract; do not assume hidden fields exist or extend Core speculatively.

Preserve these outer-language rules:

- A declaration starts with `@role Name` at column zero. A following declaration or EOF ends the body; there is no `@end` directive.
- Repeated names and empty bodies are valid. Indexes come from discovery, including unknown or conflicting declarations; never renumber them in the Role.
- Full-line `#` comments, including indented comments, are removed while preserving newline structure. Inline `#` in bodies remains. Declaration-line comments are processed by Core.
- Body whitespace and LF/CRLF structure matter. Do not silently strip or normalize the body unless the Role's own language requires it.
- Core is not quote-aware inside bodies. A column-zero `@role` declaration remains a boundary even inside text the Role would consider a string. An indented declaration inside a body is ordinary body text.
- Nonblank, noncomment content before the first declaration and declarations without a name are Core syntax errors.

## 7. Load and use multiple instances

Create UTF-8 `example.rfg` beside the consumer script:

```text
@role Echo
first
@role Echo
second
```

Use this consumer after installing the package containing the new Role and its registration:

```python
from pathlib import Path
from roleforge import load

project = load(Path(__file__).with_name("example.rfg"))
first = project.get_role("Echo", 0)
second = project.get_role("Echo", 1)

assert first is project.echo
assert first is project.echo[0]
assert second is project.echo[1]
assert first is not second
assert (first.index, first.role_index, first.source.declaration_line) == (0, 0, 1)
assert (second.index, second.role_index, second.source.declaration_line) == (1, 1, 3)
assert first.calls == second.calls == 0
assert first.text().strip() == "first"
assert (first.calls, second.calls) == (1, 0)
assert second.text().strip() == "second"
assert (first.calls, second.calls) == (1, 1)
```

The example strips only for assertions; `text()` returns the original body. Core may print its current diagnostic output during loading. No `text()` or `start()` call happens automatically.

Dynamic Project attributes use `name.lower()` only, with no snake_case conversion. The result must be a valid non-keyword Python identifier. Existing Project members take precedence. Ambiguous lowercase names require exact-name `project.get_role(name, role_index)` access. Exact-name access also handles reserved or unusual names. PascalCase in this guide is an example convention, not a new name-validation rule.

Indexing selects the Core-assigned occurrence, not a Python list offset: negative indexes do not select from the end, and slices are unsupported. `project.roles` is a tuple of discovery/routing `RoleInfo` records, not the live objects. Its `status == "resolved"` means routing succeeded, not proof that receipt happened. Source metadata there uses `info.declaration_line`, while live objects use `role.source.declaration_line`.

## 8. Imports, lifetime, and effects

The Python Bridge loads an explicit target file afresh for each delivery in the current interpreter. It uses a path-derived module name and restores the temporary `sys.modules` binding afterward. It does not add target directories to `sys.path` or establish a package-relative import context for arbitrary sibling modules.

Use dependencies that are importable in the consumer's environment. A sibling helper file is not automatically importable merely because it is next to `main.py`; package it deliberately and verify the installed import path. Do not rely on persistent target-module globals across deliveries or use them as the default store for instance state.

Live objects retain their input, class, and method globals after `load()` returns, including when a caller retains an object after releasing Project. This is object lifetime support, not an automatic cleanup protocol. If domain operations acquire resources, provide explicit Role-owned cleanup/context-management behavior as needed. Core supplies no rollback of earlier receiver effects when a later delivery fails.

## 9. Failure ownership and diagnosis

| Situation | Existing behavior and implementation guidance |
| --- | --- |
| Name absent from both registries | Unknown is reported and skipped; other resolved Roles may proceed. Verify the registration when the expected live object is missing. |
| Name present in both registries | All conflicts are reported and all delivery for the load is aborted before any target is loaded. Do not implement an override rule. |
| Invalid target/import | Core reports `PythonTargetLoadFailure`; check target and dependencies. |
| Missing/non-callable receiver | Core reports `MissingReceiver` or `ReceiverNotCallable`. |
| Invalid Role class or failed construction | Core reports `InputConversion`; preserve the subclass/construction contract. |
| Receiver raises | Core reports `ReceiverRaised` and stops later delivery. Original exception text/cause is discarded at this boundary. |
| Delivery cleanup fails | Core reports `DeliveryCleanupFailed` if there was no primary delivery failure. Do not modify the Bridge's temporary module binding from the Role. |
| Public Role method raises after receipt | The Role's actual exception propagates with normal Python behavior. Define domain exceptions here when callers need domain details. |

Delivery failures are exposed as `RuntimeError` with a named `case` and relevant Role context. Earlier successful effects are not undone. Core output failure can also abort loading. Do not catch every exception and return success from the receiver: that would falsely declare successful receipt.

A Role may validate during receipt, but then validation failures follow the receiver failure contract above. If callers need detailed domain errors after receipt, expose explicit validation/parsing methods. Do not modify the Core Error System to manage Role-internal execution, and do not invent public numeric codes from inventory identifiers such as D001.

## 10. Verification and completion requirements

Use tests appropriate to the Role's behavior. A normal implementation should demonstrate:

1. A registered source declaration loads through the public `load()` API and exposes the intended live methods.
2. Two occurrences have correct identities, independent mutable state, and expected bodies.
3. Empty and invalid domain input follow the Role's documented policy; test at the chosen receipt/method boundary.
4. Domain operations return the intended results and perform only intended effects; failures after receipt retain the documented domain exception.
5. Input metadata remains intact and the Role does not depend on a particular unrelated Role name or source order.
6. The built wheel contains the Role and resources, and a consumer outside the checkout can import dependencies and load it.

For the Windows checkout, build using its existing Python environment:

```powershell
.\.venv\Scripts\python.exe -m maturin build --release
```

Install the generated wheel into the chosen test environment using that environment's Python and the actual generated wheel path:

```text
python -m pip install --no-index --no-deps --force-reinstall <wheel-path>
python <consumer-script-path>
```

Provision required dependencies separately; `--no-deps` does not install them. Use the same test interpreter for installation and execution. Run the consumer from outside the repository without a source-tree `PYTHONPATH`; an installed build is the packaging evidence. Rebuild/reinstall after changing packaged Role files or Registry entries.

Existing Python regression suites use `python -m unittest discover -s tests` and `python -m unittest discover -s external_test_project` from the repository root. These suites temporarily modify the imported package's Registry fixtures: run serially and restore original bytes in test cleanup. Run relevant suites when changing shared integration; ordinary Role-only work should not require Rust changes. If Core changes are justified, run its relevant checks, including `cargo test`.

Keep tests outside the installed runtime. Add Role examples and document its own syntax/API, dependencies, effects, and failure behavior. In the delivery report, identify the files and Registry entry, demonstrate usage, report actual validation and limits, and distinguish implemented behavior from any future ideas.

## 11. Agent checklist and unsupported assumptions

- Preserve existing Registry entries, source identity, component boundaries, and unrelated work.
- Supply the receiver; add a subclass only when its custom API/behavior is useful.
- Keep Role-specific parsing, validation, execution, and state in the Role.
- Use the supplied input contract and public `load()` flow instead of manually constructing a parallel Project/discovery system.
- Do not add Rust Role loading, async scheduling, alias syntax, automatic installation, shared lifecycle management, or a generic parser framework as part of an ordinary Role task. These are not supplied features.
- Do not infer an automatic `main()`, `run()`, `start()`, or shutdown call. Only receipt is currently automatic.
- Do not convert example choices into new global requirements. If the request genuinely needs an architectural change, identify the contract change explicitly before implementation.

## 12. Implementation references

- [Human Role creation guide](../human/en/CREATING_ROLES.md)
- [Existing Test Role](../../roleforge/python/roleforge/roles/Test/main.py)
- [Base Role and live access](../../roleforge/python/roleforge/_live.py)
- [Neutral RoleInput](../../roleforge/src/core/role_input.rs)
- [Python Bridge](../../roleforge/src/core/bridges/python.rs)
- [Registry resolution](../../roleforge/src/core/registry.rs)
- [Public Python boundary](../../roleforge/src/python_api.rs)
- [Dynamic Registry](../../roleforge/python/roleforge/core/storage/dynamic_roles.json)
