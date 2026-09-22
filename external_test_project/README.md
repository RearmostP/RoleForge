# External RoleForge test project

This directory simulates a separate Python project consuming RoleForge as an external library. It contains consumer-facing tests, source fixtures, and a live Role example; it is not part of the RoleForge runtime.

Install the checkout in your Python environment using the [development setup](../docs/human/en/README.md#run-the-existing-example), then run from the repository root:

```sh
python -m unittest discover -s external_test_project -v
python -u external_test_project/main.py
```

Run tests serially: handoff tests temporarily replace the imported package's Registry fixtures and restore their original bytes. Subprocess tests use the same Python environment, including from other working directories. The example loads its `.rfg` file beside the script and accesses live Roles through `Project`.

Internal Rust/Core tests remain in subsystem-local `tests.rs` files, compiled only under `#[cfg(test)]`; run them with `cargo test`.

For distribution verification, build `maturin build --release` and install the generated Wheel into the consumer environment. Run from an unrelated working directory with no repository `PYTHONPATH`. Tests locate mutable Registry fixtures via `roleforge.__file__` and restore their original bytes. The consumer environment must allow writes to those files. Run the suite with `python -I -m unittest discover -s <absolute-path-to-external_test_project> -v`.
