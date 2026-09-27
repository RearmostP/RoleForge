# D001 implementation and verification

Implemented on 2026-09-26 according to prompt 03. D001 is one inventory observation mapped to one error type, not an error code. D002 and all other inventory cases remain pending.

## Final contract

Core defines the unit struct `PackagePathResolutionFailed`, with fixed name `PackagePathResolutionFailed`, category `RoleForge Error`, and the focused explanation `Failed to resolve the RoleForge package path.` It contains no Python objects, internal stages, original causes, caller metadata, ANSI codes, or printing behavior.

The public exception is `roleforge.PackagePathResolutionFailed`, exported from the native extension and package. It subclasses `AttributeError` to preserve existing catch compatibility; consumers should identify it by the dedicated exception type. No broader RoleForge hierarchy was introduced.

For exceptions raised by `load()`:

- `category` is the class attribute `"RoleForge Error"`.
- `requested_file` is a Python `str` converted from the existing load argument's Rust path/OS-string representation. It preserves the supplied path without additional filesystem probing, resolution, or normalization. Path-like arguments retain the path obtained by the existing argument conversion, not the original Python object.
- `str(error)` includes category, official name, focused explanation, and requested file. This input is not the package directory and does not imply that the input file is missing or invalid.
- `__cause__` is unset; the discarded discovery exception is not retained as `__context__`. Implicit context display is suppressed. If called while handling an unrelated caller exception, Python may retain that caller exception as context; it is not displayed by default.
- The normal Python traceback records the user's actual load call. No hooks or manual source collection are used.

The Python boundary owns load-request context and exception adaptation. Core owns the failure definition and explanation. Ordinary `Exception` failures from package/pathlib imports, package-file lookup, Path construction, resolve, parent lookup, and PathBuf extraction map to this one error. Non-Exception BaseException subclasses, including KeyboardInterrupt, SystemExit and GeneratorExit, propagate unchanged. Registry loading and subsequent work stay outside this boundary. D001 introduces no automatic output or color formatting; pre-existing successful-runtime debug/Role output remains unchanged.

## Changed files

- `roleforge/src/core/errors/package_path_resolution_failed.rs`: replace the stage-bearing definition and obsolete inline test with a single focused Core definition.
- `roleforge/src/python_api.rs`: bounded discovery, exception type, request context, translation and export.
- `roleforge/python/roleforge/__init__.py`: public exception export.
- `tests/test_d001_package_path.py`: eight Python behavior tests, outside the protected external consumer project.
- `docs/ai/ERROR_SYSTEM_IRON_RULES.md`: permit meaningful request context while prohibiting internal-stage/cause metadata.
- `docs/ai/roleforge/src/core/errors/ERROR_SYSTEM_IRON_RULES.md`: refreshed reference copy with canonical notice and correct links.
- `docs/ai/system_message_mapping/error_metadata_inventory.md`: one error type, requested-file source and no extra collection; remove obsolete stage/cause counts.
- `docs/ai/roleforge/README.md`: prompt 03 and actual status links.
- `docs/ai/roleforge/src/core/errors/D001_IMPLEMENTATION.md`: this report.

Historical prompts 01, 02 and 03 were preserved. Existing unrelated working-tree changes were preserved. No commit or installation into the external Python test project was performed.

## Verification

On Windows x64 / Python 3.10.10:

- `maturin build --release`: passed; fresh ABI3 wheel built and installed with `--no-index --no-deps --force-reinstall` into `target/d001-venv`.
- `target/d001-venv/Scripts/python.exe -I -B -m unittest discover -s tests -v`: all 8 tests passed against that wheel. Coverage includes representative discovery failures, exact relative request context through a variable, native traceback frames, no retained internal cause, process control, registry/source failures, successful live Role loading, and subprocess stdout/stderr checks.
- `cargo test --release`: all 38 Rust tests passed (0 doc tests) before the final Python context conversion was changed from Path to OS string. After that change, a final release build succeeded, but Windows Application Control blocked execution of the rebuilt test executable (OS error 4551). The final Python tests above verify that exact context conversion against the fresh wheel.
- Default `cargo test`: blocked when Windows Application Control rejected execution of a PyO3 build script (OS error 4551). The earlier release test run passed, with the final rerun limitation recorded above.
- Installed native extension and package exports were checked byte-for-byte against the fresh wheel and matched.
- Rust formatting of changed implementation files and `git diff --check`: passed. Repository-wide formatting still reports pre-existing formatting in two unrelated test files; those files were preserved.

The old `external_test_project/test_d001_package_file.py` asserts the superseded missing-attribute-only contract and retained cause. It was neither changed nor run; the new tests replace that D001 verification for this task. No other Python consumer suite or platform was verified. D001 implementation is complete; final Rust test execution remains limited by the OS policy above.

## Copyable example

Run against the newly built installation. Removing the package attribute deliberately simulates discovery failure and is restored afterward:

```python
import traceback
import roleforge

requested_file = "./app.rfg"
original_file = roleforge.__file__
try:
    del roleforge.__file__
    try:
        roleforge.load(requested_file)
    except roleforge.PackagePathResolutionFailed as error:
        print(error.category)
        print(error.requested_file)  # ./app.rfg
        traceback.print_exception(type(error), error, error.__traceback__)
finally:
    roleforge.__file__ = original_file
```

Only this consumer example prints; `load()` does not print the D001 diagnostic. Traceback layout and exception prefix are controlled by Python.
