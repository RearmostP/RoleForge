"""D001 behavior against a freshly built, installed wheel (stdlib unittest)."""
import importlib
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import traceback
import types
import unittest
from unittest.mock import patch

import roleforge
from roleforge import PackagePathResolutionFailed, load


class PackagePathTests(unittest.TestCase):
    def check_failure(self, requested_file):
        try:
            load(requested_file)
        except PackagePathResolutionFailed as error:
            self.assertIsInstance(error, AttributeError)
            self.assertEqual(error.category, "RoleForge Error")
            self.assertEqual(error.requested_file, os.fspath(requested_file))
            self.assertEqual(str(error), "RoleForge Error: PackagePathResolutionFailed\n"
                             "Failed to resolve the RoleForge package path.\n"
                             f"Requested file: {os.fspath(requested_file)}")
            self.assertIsNone(error.__cause__)
            self.assertIsNone(error.__context__)
            self.assertFalse(hasattr(error, "stage"))
            self.assertNotIn("\x1b", str(error))
            frames = traceback.extract_tb(error.__traceback__)
            self.assertTrue(any(frame.filename == __file__ and
                                frame.name == "check_failure" and
                                frame.line == "load(requested_file)" for frame in frames))
        else:
            self.fail("Expected package discovery failure")

    def test_missing_package_file_and_distinct_relative_requests(self):
        original = roleforge.__file__
        try:
            del roleforge.__file__
            for requested_file in ("./folder/../first.rfg", "second.rfg", Path("third.rfg")):
                self.check_failure(requested_file)
        finally:
            roleforge.__file__ = original

    def test_representative_discovery_failures(self):
        for module in ("roleforge", "pathlib"):
            with self.subTest(module=module), patch.dict(sys.modules, {module: None}):
                self.check_failure("app.rfg")
        with patch.object(roleforge, "__file__", None):
            self.check_failure("app.rfg")
        with patch("pathlib.Path.resolve", side_effect=OSError("discard this detail")):
            self.check_failure("app.rfg")
        with patch("pathlib.Path") as constructor:
            constructor.return_value.resolve.return_value.parent = object()
            self.check_failure("app.rfg")

    def test_process_control_propagates_unchanged(self):
        for exception in (KeyboardInterrupt(), SystemExit(23), GeneratorExit()):
            with self.subTest(exception=type(exception).__name__):
                with patch("pathlib.Path.resolve", side_effect=exception):
                    try:
                        load("app.rfg")
                    except BaseException as caught:
                        self.assertIs(caught, exception)
                    else:
                        self.fail("Expected process-control exception")

    def test_custom_package_lookup_failure(self):
        class BrokenPackage(types.ModuleType):
            def __getattribute__(self, name):
                if name == "__file__":
                    raise RuntimeError("discard this detail")
                return super().__getattribute__(name)
        original = roleforge.__class__
        try:
            roleforge.__class__ = BrokenPackage
            self.check_failure("app.rfg")
        finally:
            roleforge.__class__ = original

    def test_registry_and_source_errors_remain_outside_boundary(self):
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(roleforge, "__file__", str(Path(directory) / "__init__.py")):
                with self.assertRaises(OSError):
                    load("app.rfg")
            with self.assertRaises(FileNotFoundError):
                load(Path(directory) / "absent.rfg")
            source = Path(directory) / "invalid.rfg"
            source.write_text("@role\n", encoding="utf-8")
            with self.assertRaises(ValueError):
                load(source)

    def test_successful_loading_and_live_role(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "valid.rfg"
            source.write_text("@role Test\nhello\n", encoding="utf-8")
            project = load(source)
            self.assertEqual(project.roles[0].status, "resolved")
            self.assertEqual(project.test.hello(), project.roles[0].body)

    def test_no_automatic_output_including_native_streams(self):
        result = subprocess.run([sys.executable, "-I", "-c", '''
import roleforge
del roleforge.__file__
try:
    roleforge.load("app.rfg")
except roleforge.PackagePathResolutionFailed:
    pass
'''], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertEqual(result.stderr, "")

    def test_active_caller_exception_does_not_reveal_internal_failure(self):
        with patch("pathlib.Path.resolve", side_effect=OSError("private discovery detail")):
            try:
                raise ValueError("caller context")
            except ValueError:
                try:
                    load("app.rfg")
                except PackagePathResolutionFailed as error:
                    rendered = "".join(traceback.format_exception(type(error), error, error.__traceback__))
                    self.assertNotIn("private discovery detail", rendered)
                    self.assertNotIn("caller context", rendered)
                    self.assertIsNone(error.__cause__)
                    self.assertNotIsInstance(error.__context__, OSError)
                else:
                    self.fail("Expected discovery failure")


if __name__ == "__main__":
    unittest.main()
