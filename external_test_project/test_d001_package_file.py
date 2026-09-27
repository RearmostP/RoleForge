"""Regression coverage for the first PackagePathResolutionFailed case."""
from pathlib import Path
from contextlib import redirect_stdout
import importlib
import io
import types
import unittest

from roleforge import PackagePathResolutionFailed, load


class D001PackageFileTests(unittest.TestCase):
    def setUp(self):
        self.package = importlib.import_module("roleforge")
        self.original_class = type(self.package)
        self.original_file = self.package.__file__

    def tearDown(self):
        self.package.__class__ = self.original_class
        self.package.__file__ = self.original_file

    def test_missing_file_attribute_reports_named_error_without_internal_cause(self):
        del self.package.__file__

        output = io.StringIO()
        with redirect_stdout(output):
            with self.assertRaises(AttributeError) as raised:
                load(Path(__file__).with_name("test.rfg"))

        self.assertEqual(
            str(raised.exception),
            "RoleForge Error: PackagePathResolutionFailed\n"
            "Failed to resolve the RoleForge package path.\n"
            f"Requested file: {Path(__file__).with_name('test.rfg')}",
        )
        self.assertIsInstance(raised.exception, PackagePathResolutionFailed)
        self.assertIsNone(raised.exception.__cause__)
        self.assertFalse(hasattr(raised.exception, "stage"))
        self.assertEqual(output.getvalue(), "")

    def test_non_attribute_error_during_file_lookup_maps_to_the_same_case(self):
        class FailingFileModule(types.ModuleType):
            def __getattribute__(self, name):
                if name == "__file__":
                    raise RuntimeError("custom module lookup failure")
                return super().__getattribute__(name)

        self.package.__class__ = FailingFileModule

        with self.assertRaises(PackagePathResolutionFailed):
            load(Path(__file__).with_name("missing-source.rfg"))

    def test_present_none_value_maps_to_package_path_failure(self):
        self.package.__file__ = None

        with self.assertRaises(PackagePathResolutionFailed) as raised:
            load(Path(__file__).with_name("missing-source.rfg"))

        self.assertIn("PackagePathResolutionFailed", str(raised.exception))
        self.assertIsNone(raised.exception.__cause__)

    def test_normal_loading_and_live_test_role_access(self):
        project = load(Path(__file__).with_name("test.rfg"))

        self.assertEqual([role.status for role in project.roles], ["resolved", "resolved"])
        self.assertEqual(project.test[0].hello(), project.roles[0].body)
        self.assertEqual(project.test[1].hello(), project.roles[1].body)


if __name__ == "__main__":
    unittest.main()
