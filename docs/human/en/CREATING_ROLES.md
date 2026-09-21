# Creating a dynamic Python Role

[Documentation](README.md) · [עברית](../he/CREATING_ROLES.md) · [Iron Rules](IRON_RULES.md)

Python is currently the only implemented Role delivery environment. Follow the [checkout setup](README.md#run-the-existing-example) first. There is no Role installation CLI or registration API: current registration is a manual JSON edit.

The checkout already contains `Test`. The following is a complete minimal version of that same pattern; when trying a new Role, choose a new name and preserve other Registry entries.

## 1. Write the receiver

The current dynamic Role area is `roleforge/roles/`:

```text
roleforge/
└── roles/
    └── Test/
        └── main.py
```

In `roleforge/roles/Test/main.py`:

```python
from roleforge import Role as BaseRole

class Role(BaseRole):
    def hello(self):
        self.calls += 1
        return self.body

def roleforge_receive(role):
    role.calls = 0
    print(role.name)
    print(role.index)
    print(role.role_index)
    print(repr(role.body))
    print(role.source.declaration_line)
```

`roleforge_receive` is the mandatory receiving entry point. Core will not try `main`, `run`, `receive`, or `start` instead. Use an ordinary synchronous callable accepting one object. Async Role execution is not implemented.

Receipt means that RoleForge delivered a discovered instance. It does not mean that Core should execute your user-facing operations. `start()` is separate and is not automatically called. Returning normally counts as successful receipt; any return value is ignored. Raising an exception fails delivery.

The existing [Test implementation](../../../roleforge/roles/Test/main.py) also prints labels and exposes `hello()`. The optional target-module `Role` class subclasses `roleforge.Role`. Its inherited constructor adapts RoleInput without field-copying boilerplate; initialize state in `roleforge_receive`. Targets without a `Role` class receive a generic live Role. A declared `Role` must be a subclass and accept the inherited one-input construction contract. The Bridge does not infer APIs from other module functions or interpret method names.

## 2. Register the destination

Edit `roleforge/core/storage/dynamic_roles.json`:

```json
{
  "Test": {
    "entry": {
      "via": "python",
      "target": "Test/main.py"
    }
  }
}
```

| Field | Meaning |
| --- | --- |
| `Test` | The name to match against a discovered `@role Test` declaration. |
| `via` | Exact identifier of a registered Bridge. `python` is the sole built-in registration today. It is not a generic programming-language field. |
| `target` | Destination interpreted by the selected Bridge; currently a Python source file for Python delivery. |

Selection never infers a Bridge from `.py`. An unregistered identifier fails even if the file contains valid Python. Bridge registration itself is internal Rust construction, not a public Python registration API.

Current target resolution:

- Dynamic relative targets use `roleforge/roles/` as their base.
- Built-in relative targets use `roleforge/builtin_roles/`.
- Absolute targets remain absolute.

Thus `Test/main.py` resolves to `roleforge/roles/Test/main.py`, not beside the `.rfg` file and not relative to the Python working directory. These bases and Registry storage are derived from the checkout location at build time. Keep that checkout available; final packaging rules are not decided.

Register a name in only one of `dynamic_roles.json` and `builtin_roles.json`. Registering it in both creates a Conflict and prevents all delivery for that load. Repeating a declaration in source is allowed and does not create a Registry conflict.

## 3. Create and load source

Save this as UTF-8 `test.rfg` in the directory from which you will run the following Python snippet:

```text
@role Test

hello = world
number = 123
```

```python
from roleforge import load

project = load("test.rfg")
print(project.test.hello())
```

Do not import the Role manually. `load()` reads the file, discovers `Test`, resolves its entry, checks conflicts, creates RoleInput, selects the Python Bridge, loads the target, and calls `roleforge_receive(role)`.

Here `name` is `Test`, both indexes are `0`, and `source.declaration_line` is `1`. With LF line endings and a final newline, the body is `'\nhello = world\nnumber = 123\n'`. CRLF source retains CRLF; the leading blank line is part of the body. Core does not interpret assignments in it.

A relative path passed to `load()` is relative to the caller's working directory. That is separate from Registry target resolution. The repository's [main.py](../../../anyone_py_project/main.py) uses `Path(__file__).with_name("test.rfg")` so the example finds its own source from another working directory.

## 4. Understand RoleInput

Python receives a live object whose input-derived fields and nested source metadata remain read-only. Other attributes can hold per-instance state:

| Attribute | Meaning |
| --- | --- |
| `role.name` | Name discovered in `@role Name`. |
| `role.index` | Zero-based position among all Roles in this source. |
| `role.role_index` | Zero-based occurrence among Roles with exactly the same name. |
| `role.body` | Body after existing Core-level comment processing; Role syntax remains uninterpreted. |
| `role.source.declaration_line` | One-based line of the declaration in the source file. |

Core establishes indexes during discovery, including for Unknown or conflicting Roles. Bridges do not recalculate them. Internally, Handoff explicitly converts discovery data (`CleanRole`) into the neutral RoleInput contract. The Python Bridge adapts its representation.

For this conceptual discovery example (it does not register these Roles):

```text
@role Directory
@role Config
@role Directory
```

| Name | index | role_index |
| --- | --- | --- |
| Directory | 0 | 0 |
| Config | 1 | 0 |
| Directory | 2 | 1 |

## 5. Deliver multiple instances

The repository's [test.rfg](../../../anyone_py_project/test.rfg) contains:

```text
@role Test

hello = first

@role Test

hello = second
```

One registration delivers both instances independently to the same receiver, in source order. The calls receive `(index, role_index)` values `(0, 0)` and `(1, 1)`, with declaration lines `1` and `5`. With LF endings, their bodies are `'\nhello = first\n\n'` and `'\nhello = second\n'`.

`project.test is project.test[0]` is true. `project.test[1]` is a distinct live object, and `hello()` observes that object's body and state. `project.roles` remains ordered `RoleInfo` discovery/routing metadata; it is separate from both neutral RoleInput and live instances. The source filename identifies the Project context, while `(name, role_index)` identifies an occurrence within it; `index` retains global source order.

Dynamic attributes use only `name.lower()`, with no snake_case conversion. Only valid non-keyword Python identifiers are exposed this way. Existing Project attributes/methods take precedence. Case collisions raise `AttributeError`; use `project.get_role("ExactName", role_index)` for exact-name access, including reserved or unusual names. A missing dynamic attribute raises `AttributeError`, a missing exact name raises `KeyError`, and a nonexistent index (including negative indexes) raises `IndexError`. Indexing selects Core's Role-local identity, not Python sequence offsets; slices are unsupported. Broader naming and alias policies remain open.

For advanced implementations managing their own representation, `role.role_input` exposes the frozen input; retain custom objects as Role state without changing the ignored receiver return value.

## Outer syntax to respect

- `@role Name` starts at column 0. Indented `@role` inside a body is ordinary body text.
- A block ends at the next valid declaration or EOF. There is no `@end` directive.
- Blank and full-line comment text may precede the first Role; other content there is an error. A missing Role name is an error.
- Full-line `#` comments, including indented ones, are removed while preserving their newline structure. Inline `#` in bodies remains untouched. A declaration can have an inline comment, as in `@role Test # comment`.
- Empty bodies and repeated names are allowed. Core is not quote-aware within Role bodies; a column-0 declaration is still a boundary there.

## Loading limits and diagnosing failures

The Bridge loads an explicit file in the current Python process. It loads source afresh for each delivery, uses the full canonical path to avoid basename collisions, and restores its temporary `sys.modules` binding afterward. Do not rely on persistent module globals across deliveries. Live instances retain their input, class, and method globals after `load()` returns, including when an instance is retained after Project is released. It does not discover packages, search for Roles, or add target directories to `sys.path`.

Unknown Bridge identifiers, unloadable targets, missing receivers, non-callable receivers, and receiver exceptions produce distinct internal errors exposed as `RuntimeError` with context. Fix the registration/file/receiver indicated. No alternative receiver or Bridge is tried. See the [routing table](README.md#routing-and-failures-today) for Unknown Roles and Conflict preflight.

Rust Role delivery is intentionally absent; no Rust loading or receiving protocol has been selected. Role aliases, an installation API, final execution lifecycle, and final error/console infrastructure are also open work, not features this guide relies on.
