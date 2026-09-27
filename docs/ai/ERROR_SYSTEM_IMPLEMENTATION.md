# Core Error System — current implementation

Implemented and verified on 2026-09-27. This is the active implementation record;
the Iron Rules remain authoritative. Historical prompts and stage reports are
unchanged. Inventory IDs are observations, never public codes.

## Architecture

- `core/errors/` owns explicit neutral error definitions and their focused
  messages. It exports a small internal API, not a manager or a generic metadata
  framework. Rust tests are physically separate in `errors/tests.rs`.
- Loader and Registry map actual read/parse failures into named cases. Scanner
  uses the common syntax definitions. Registry/Dispatcher retain existing
  structured routing results and never print or assign severity.
- Handoff/Bridge return receipt failures. Runtime supplies affected Role identity
  and target, owns orchestration and continuation, and emits borrowed `CoreEvent`
  values through a synchronous reporter. No writer or presentation module enters
  Runtime production code.
- `core/output.rs` renders those events with the existing text, ordering, and
  stdout policy. It has no recovery or Role policy. `python_api.rs` wires it in.
- `python_errors.rs` adapts neutral errors into Python exceptions. `_errors.py`
  contains the focused Python-owned access-error constructors used by `_live.py`.
  Python objects and PyO3 never enter the neutral error definitions.
- Error construction never prints. Runtime reporting remains mandatory during
  normal operation; there are no disabling flags, logging services, speculative
  severity levels, caches, or lifecycle additions.

## Meaningful cases and data

| Group | Cases | Case-specific data |
|---|---|---|
| Package discovery | `PackagePathResolutionFailed` | Python load boundary retains `requested_file`; existing public type/message unchanged |
| Files and registries | `FileNotFound`, `FileReadFailed`, `InvalidRegistry` | Actual resource path, including which registry file |
| Outer syntax | `MissingRoleName`, `ContentBeforeRole` | One-based line; Python boundary adds requested source path |
| Delivery | `UnknownBridge`, `PythonTargetLoadFailure`, `MissingReceiver`, `ReceiverNotCallable`, `InputConversion`, `ReceiverRaised`, `DeliveryCleanupFailed` | Handoff context: Role name, global/local indexes, declaration line, target; unknown identifier only for UnknownBridge |
| Output / public result | `OutputWriteFailed`, `ProjectConstructionFailed` | Output exception compatibility; requested file for Project construction |
| Live access | `LiveRoleNotFound`, `RoleOccurrenceNotFound`, `LiveRoleAttributeNotFound`, `AmbiguousRoleAttribute` | Exact name, occurrence index, attribute, or conflicting exact names |

The Python adapter's `invalid_role_class()` defines the existing TypeError message
and the local `InvalidRoleClass` case. During normal load this is folded into the
single delivery case `InputConversion`, as are constructor/input adaptation
failures. Core does not model their internal reasons independently.

`CoreEvent` has `ResolvedRole`, `UnknownRole`, `RoleConflict`, and
`HandoffAborted`. These are observations, not severity assignments. They borrow
existing Role/entry data, preserving both indexes and both conflicting entries.
Resolved means routing succeeded, not that receipt has already occurred.

`FileReadFailed` and `OutputWriteFailed` retain `io::ErrorKind` solely for
Python exception-class compatibility. No original I/O exception, OS text,
serde message, Python exception/traceback, or cause chain is retained. Bridge
errors have no arbitrary diagnostic strings. Requested paths are not probed
again merely to format an error.

## Preserved behavior and deliberate message changes

- Unknown is reported and skipped; later resolved instances proceed in source
  order. Discovery and global/local identities remain available.
- Any Conflict prevents **all** target loading/delivery. Every conflict is
  reported, then HandoffAborted; discovery-only Project still returns if reporting
  and Project construction succeed. Unknown/resolved output remains suppressed
  on that branch. No precedence or wider registry validation was introduced.
- Bridge/receipt failure stops at the first failing delivery. Earlier effects
  remain; no rollback and no continuation to later Roles.
- Output failure stops immediately at the existing reporting point, potentially
  before a receiver call or after earlier successful effects. It remains an I/O
  exception, not a swallowed warning.
- Cleanup is attempted after the receipt block. A primary failure wins over a
  secondary cleanup failure. Cleanup-only failure still fails load; its official
  case is now `DeliveryCleanupFailed`, instead of misleading target-load text.
- Files retain Python I/O classes (FileNotFoundError, PermissionError, OSError,
  etc.); syntax keeps ValueError and its existing path:line message; delivery keeps
  RuntimeError. OS-specific errno/message text is not promised or retained.
- Existing Python access exception classes and messages stay unchanged. Known
  cases now expose `case` and focused attributes (e.g. role_index or role_names).
  Rust-mapped failures likewise expose `case` plus their relevant fields.
- PackagePathResolutionFailed retains its dedicated AttributeError subtype,
  category, requested_file and exact message. Its process-control behavior stays.
- Failures in Core-owned Project preparation (adapter import and native
  list/metadata/tuple allocation) become the explicit `RuntimeError` case
  `ProjectConstructionFailed`. Grouping already-received live Roles accesses
  overridable Python attributes, so that entire call preserves its original
  exception behavior: post-receipt Role property/setter failures are never
  wrapped as Core failures. Stop/rollback behavior is unchanged. Non-Exception
  BaseException subclasses also propagate unchanged from Core preparation.
- Native import/module initialization, argument binding, `__fspath__`,
  `operator.index`, descriptors/frozen metadata, and integer overflow keep their
  Python protocol behavior. They are not reimplemented or globally intercepted.
- Missing Role class still uses the generic Role. Return values are still ignored.
  Exact-name access, keyword/reserved names, and ambiguous attribute handling are
  preserved. Bridge registration replacement remains an explicit internal action.
- Exceptions from Role methods after successful handoff remain the Role's actual
  exception object, with normal Python behavior. No automatic start() or unrelated
  future features were added.

## Complete inventory disposition

| Inventory observations | Current handling |
|---|---|
| D001 | Existing PackagePathResolutionFailed contract retained |
| D002–D003 | FileNotFound / FileReadFailed with registry resource path |
| D004 | InvalidRegistry; malformed JSON and invalid schema are one case |
| D005–D006 | FileNotFound / FileReadFailed with source path; invalid UTF-8 does not create an internal parser taxonomy |
| D007–D008 | Central syntax cases, existing first-failure policy |
| D009–D011 | Structured UnknownRole / RoleConflict / HandoffAborted; existing preflight policy |
| D012 | OutputWriteFailed at synchronous reporter boundary |
| D013 | UnknownBridge, exact identifier only |
| D014 | HandoffFailed carries affected identity/destination and the actual case |
| D015 | Explicit Bridge registration replacement; not a new failure |
| D016–D024 | One PythonTargetLoadFailure; no internal stages or cause strings |
| D025–D026 | MissingReceiver / ReceiverNotCallable; non-AttributeError lookup remains target-load failure |
| D027–D028 | InputConversion; local class validation message lives in Python error definitions |
| D029 | ReceiverRaised, until synchronous receipt succeeds |
| D030 | DeliveryCleanupFailed; still aborts load after receipt |
| D031–D033 | Native module/import/binding protocol errors preserved |
| D034–D035 | ProjectConstructionFailed for Core preparation/allocation; live grouping exceptions propagate unchanged to preserve post-receipt Role ownership; no rollback |
| D036 | LiveRoleNotFound, still KeyError |
| D037 | Native operator.index errors preserved |
| D038 | RoleOccurrenceNotFound, still IndexError |
| D039–D040 | LiveRoleAttributeNotFound / AmbiguousRoleAttribute, still AttributeError |
| D041–D043 | Existing attribute eligibility, member precedence, and descriptors; no invented events |
| D044–D047 | Structured runtime events rendered only by output adapter |
| D048–D049 | Role/example output outside Core Error System; unchanged |
| D050–D051 | Approved generic-Role fallback and ignored receiver return; unchanged |
| D052 | Primary receipt/load failure wins over secondary cleanup failure |
| D053 | Original lower-level exceptions discarded; focused Core message conversion |
| D054 | Infallible String formatting assumption; no invented user failure |

## Files

New Rust definitions: `roleforge/src/core/errors/{files,syntax,delivery,runtime,events,tests}.rs`.
New adapters: `roleforge/src/core/output.rs`, `roleforge/src/python_errors.rs`,
`roleforge/python/roleforge/_errors.py`. The former `final_core_debug.rs` is replaced
by the output adapter. Loader, Scanner type exports, Registry, Dispatcher,
Handoff, Python Bridge, Runtime and Python API are integrated with these types.

Behavior tests: separate subsystem tests, `tests/test_core_errors.py`, existing
D001 tests and consumer suites. Two superseded consumer expectations were
updated: old D001 stages/causes and receiver exception text. Their continuation,
exception-class and successful-live-object assertions remain intact.

## Validation

Windows x64, Python 3.10.10. Baseline before implementation: 38 Rust tests passed.
Final command results are recorded below. Test installation is
isolated under `target/error-system-venv`, using a fresh release wheel, not an
editable install. The external experimentation project's files and its existing
environment were not modified.

No unresolved continuation/severity policy was required for this implementation.
Severity classification and future features remain intentionally undecided.

| Validation actually run | Result |
|---|---|
| `cargo fmt --check` | Passed |
| `cargo check` | Passed |
| `cargo build` | Passed |
| `cargo test` | 46 passed, 0 failed; 0 doc tests |
| `cargo test --release` | 46 passed, 0 failed; 0 doc tests |
| `.venv/Scripts/maturin.exe build --release` | Passed; fresh cp310-abi3 Windows x64 wheel |
| Isolated Python `-I -B -m unittest discover -s <repo>/tests -v` | 25 passed, 0 failed |
| Isolated Python `-I -B -m unittest discover -s <repo>/external_test_project -v` | 31 passed, 0 failed |
| Wheel content / installed-byte / source-byte comparison | 12 entries, 8 runtime files; matching bytes; no Rust, tests, prompts, docs or bytecode caches |
| External consumer live flow | Passed from the actual consumer cwd using its existing test.rfg; two live instances, exact-name/index access, independent state, retained lifetime |
| `git diff --check` | Passed |

The isolated interpreter used for Python validation was
`D:/David/Projects/Rust/RoleForge/target/error-system-venv/Scripts/python.exe`.
It was created with `.venv/Scripts/python.exe -m venv target/error-system-venv`.
The wheel was installed using `pip install --no-index --no-deps --force-reinstall`
with `target/wheels/roleforge-0.1.0-cp310-abi3-win_amd64.whl`.
Both Python suites ran serially from
`D:/David/Projects/Python/test_my_libraty`, with absolute discovery paths and no
source-tree PYTHONPATH. Installed registries matched wheel/source bytes after
the tests, confirming fixture restoration. The consumer's current main.py is
an error-text experiment, so the live flow used its existing test.rfg directly;
the real external_test_project/main.py is also exercised by the consumer suite.

Logs: `target/error-system-python.log`, `target/error-system-consumer-tests.log`,
and `target/error-system-wheel-live.log`.

Known limitation: the pre-existing unused `LoadedFile.path` warning remains.
Validation was on Windows x64 / Python 3.10.10 only; other platforms were not run.
No commits, staging changes, publishing, or installation into the user's external
environment were performed. Existing unrelated work and staged content remain.
