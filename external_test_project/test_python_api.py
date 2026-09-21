"""Python-facing tests: python -m unittest discover -s external_test_project -v."""
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from roleforge import Project, load


class PythonApiTests(unittest.TestCase):
    def test_real_project_preserves_source_order_and_metadata(self):
        path = Path(__file__).with_name("test_role.rfg")
        project = load(path)
        self.assertIsInstance(project, Project)
        self.assertEqual(project.path, path)
        self.assertIsInstance(project.roles, tuple)
        source = path.read_bytes().decode("utf-8")
        newline = "\r\n" if "\r\n" in source else "\n"
        self.assertEqual(
            [(r.name, r.index, r.role_index, r.body, r.declaration_line)
             for r in project.roles],
            [("Directory", 0, 0, f"src/{newline}tests/{newline}{newline}", 1),
             ("Config", 1, 0, f"debug = true{newline}{newline}", 5),
             ("Directory", 2, 1, "assets/", 8)],
        )
        # Directory and Config are unregistered; discovery still succeeds.
        for role in project.roles:
            self.assertEqual(role.status, "unknown")
            self.assertIsNone(role.entry)
            self.assertIsNone(role.builtin_entry)
            self.assertIsNone(role.dynamic_entry)
            self.assertFalse(hasattr(role, "start"))
        with self.assertRaises(AttributeError):
            project.roles[0].role_index = 99
        with self.assertRaises(AttributeError):
            project.roles = ()

    def test_unknown_arbitrary_names_and_unicode_are_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "source.rfg"
            path.write_text("@role ThirdParty_Example\nשלום # opaque\n@role ThirdParty_Example\nlast", encoding="utf-8")
            project = load(str(path))
            self.assertEqual([r.name for r in project.roles], ["ThirdParty_Example"] * 2)
            self.assertEqual([r.index for r in project.roles], [0, 1])
            self.assertEqual([r.role_index for r in project.roles], [0, 1])
            self.assertIn("שלום # opaque", project.roles[0].body)
            self.assertEqual([r.status for r in project.roles], ["unknown"] * 2)

    def test_empty_source(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "empty.rfg"
            path.write_text("# comment\n", encoding="utf-8")
            self.assertEqual(load(path).roles, ())

    def test_source_errors_cross_the_python_boundary(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "source.rfg"
            with self.assertRaises(FileNotFoundError):
                load(path)
            for source, message in [("@role\n", "missing Role name"),
                                    ("ordinary text\n", "content before first Role")]:
                path.write_text(source, encoding="utf-8")
                with self.assertRaisesRegex(ValueError, f":1: {message}"):
                    load(path)
            path.write_bytes(b"\xff")
            with self.assertRaises(OSError):
                load(path)

    def test_unregistered_project_loads_from_another_working_directory(self):
        source = Path(__file__).with_name("test_role.rfg").resolve()
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run([sys.executable, "-c",
                                     "import sys; from roleforge import load; "
                                     "project = load(sys.argv[1]); "
                                     "print(f'Loaded {len(project.roles)} Roles')", str(source)], cwd=directory,
                                    capture_output=True, text=True, check=True)
        self.assertEqual(result.stdout.count("Unknown Role: Directory"), 2)
        self.assertEqual(result.stdout.count("Unknown Role: Config"), 1)
        self.assertNotIn("[CORE DEBUG]", result.stdout)
        self.assertIn("Loaded 3 Roles", result.stdout)
        self.assertNotIn("[HANDOFF]", result.stdout)

    def test_manual_example_delivers_both_instances_from_another_working_directory(self):
        script = Path(__file__).with_name("main.py").resolve()
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run([sys.executable, "-u", str(script)], cwd=directory,
                                    capture_output=True, text=True, check=True)
        output = result.stdout
        self.assertEqual(output.count("=== ROLEFORGE TEST ROLE ==="), 2)
        self.assertEqual(output.count("Role received successfully!"), 2)
        source = script.with_name("test.rfg").read_bytes().decode("utf-8")
        bodies = source.split("@role Test")[1:]
        for index, (body, line) in enumerate(zip(bodies, (1, 5))):
            # The declaration's newline is outside the Role body.
            body = body[2:] if body.startswith("\r\n") else body[1:]
            self.assertIn(f"name: Test\nglobal index: {index}\nrole index: {index}\n"
                          f"body: {body!r}\ndeclaration line: {line}", output)
        self.assertLess(output.index("global index: 0"), output.index("global index: 1"))
        self.assertIn("Loaded 2 Roles", output)
        self.assertNotIn("Unknown Role:", output)


if __name__ == "__main__":
    unittest.main()
