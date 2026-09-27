"""Core error contracts against an installed wheel; run serially.

Registry fixtures are always restored byte-for-byte. Role implementation
exceptions after successful handoff are deliberately outside this system.
"""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import roleforge
import roleforge._live as live_adapter


class CoreErrorTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        self.source = self.root / "source.rfg"
        self.target = self.root / "receiver.py"
        self.source.write_text("@role Example\nbody\n", encoding="utf-8")
        self.target.write_text("def roleforge_receive(role): pass\n", encoding="utf-8")
        storage = Path(roleforge.__file__).parent / "core/storage"
        self.builtin, self.dynamic = [storage / f"{name}_roles.json"
                                      for name in ("builtin", "dynamic")]
        for path in (self.builtin, self.dynamic):
            self.addCleanup(path.write_bytes, path.read_bytes())
            path.write_text("{}", encoding="utf-8")
        self.register("Example")

    def register(self, *names, via="python"):
        self.dynamic.write_text(json.dumps({
            name: {"entry": {"via": via, "target": str(self.target)}}
            for name in names
        }), encoding="utf-8")

    def failure(self, exception_type, case):
        with self.assertRaises(exception_type) as caught:
            roleforge.load(self.source)
        error = caught.exception
        self.assertEqual(error.case, case)
        self.assertIsNone(error.__cause__)
        self.assertIsNone(error.__context__)
        self.assertNotIn("private detail", str(error))
        self.assertFalse(hasattr(error, "stage"))
        return error

    def test_source_failures_have_structured_paths_and_preserve_exception_classes(self):
        self.source.unlink()
        error = self.failure(FileNotFoundError, "FileNotFound")
        self.assertEqual(error.path, str(self.source))
        self.source.write_bytes(b"\xff")
        error = self.failure(OSError, "FileReadFailed")
        self.assertEqual(error.path, str(self.source))

    def test_registry_failures_identify_either_resource_without_fallback(self):
        for path in (self.builtin, self.dynamic):
            original = path.read_bytes()
            with self.subTest(path=path):
                try:
                    path.unlink()
                    error = self.failure(FileNotFoundError, "FileNotFound")
                    self.assertEqual(Path(error.path), path)
                    path.write_text('{"private detail":', encoding="utf-8")
                    error = self.failure(OSError, "InvalidRegistry")
                    self.assertEqual(Path(error.path), path)
                    path.write_bytes(b"\xff")
                    error = self.failure(OSError, "FileReadFailed")
                    self.assertEqual(Path(error.path), path)
                finally:
                    path.write_bytes(original)

    def test_registry_read_order_remains_before_json_validation_and_source_loading(self):
        self.builtin.write_text("{", encoding="utf-8")
        self.dynamic.unlink()
        self.source.unlink()
        error = self.failure(FileNotFoundError, "FileNotFound")
        self.assertEqual(Path(error.path), self.dynamic)

    def test_syntax_failures_keep_line_path_and_existing_messages(self):
        for source, case, line, message in [
            ("# comment\n@role # comment\n", "MissingRoleName", 2, "missing Role name"),
            ("\nordinary content\n", "ContentBeforeRole", 2, "content before first Role"),
        ]:
            with self.subTest(case=case):
                self.source.write_text(source, encoding="utf-8")
                error = self.failure(ValueError, case)
                self.assertEqual(error.line, line)
                self.assertEqual(error.path, str(self.source))
                self.assertEqual(str(error), f"{self.source}:{line}: {message}")

    def test_bridge_cases_keep_identity_without_original_python_exceptions(self):
        for implementation, case in [
            ("invalid python !!!", "PythonTargetLoadFailure"),
            ("raise RuntimeError('private detail')", "PythonTargetLoadFailure"),
            ("def main(): pass", "MissingReceiver"),
            ("roleforge_receive = None", "ReceiverNotCallable"),
            ("Role = 123\ndef roleforge_receive(role): pass", "InputConversion"),
            ("def roleforge_receive(role): raise ValueError('private detail')", "ReceiverRaised"),
            ("def roleforge_receive(): pass", "ReceiverRaised"),
        ]:
            with self.subTest(case=case, implementation=implementation):
                self.target.write_text(implementation, encoding="utf-8")
                error = self.failure(RuntimeError, case)
                self.assertEqual((error.role_name, error.index, error.role_index,
                                  error.declaration_line), ("Example", 0, 0, 1))
                self.assertEqual(error.target, str(self.target))
                self.assertIn(case, str(error))

    def test_all_target_preparation_failures_map_to_one_case(self):
        self.target.unlink()
        self.failure(RuntimeError, "PythonTargetLoadFailure")
        self.target.write_text("def roleforge_receive(role): pass", encoding="utf-8")
        with patch("importlib.util.spec_from_loader", side_effect=ValueError("private detail")):
            self.failure(RuntimeError, "PythonTargetLoadFailure")

    def test_unknown_bridge_keeps_identifier_and_never_loads_target(self):
        self.target.write_text("raise AssertionError('must not load')", encoding="utf-8")
        for identifier in ("", "rust", "Python", "python "):
            with self.subTest(identifier=identifier):
                self.register("Example", via=identifier)
                error = self.failure(RuntimeError, "UnknownBridge")
                self.assertEqual(error.identifier, identifier)

    def test_cleanup_failure_after_receipt_still_fails_and_stops_later_delivery(self):
        log = self.root / "received.txt"
        self.target.write_text(f"""import sys
from pathlib import Path
def roleforge_receive(role):
    with Path({str(log)!r}).open('a') as output:
        output.write(str(role.index))
    del sys.modules[__name__]
""", encoding="utf-8")
        self.source.write_text("@role Example\n@role Example\n", encoding="utf-8")
        self.failure(RuntimeError, "DeliveryCleanupFailed")
        self.assertEqual(log.read_text(), "0")

    def test_primary_receiver_failure_wins_over_cleanup_failure(self):
        self.target.write_text("""import sys
def roleforge_receive(role):
    del sys.modules[__name__]
    raise ValueError('private detail')
""", encoding="utf-8")
        self.failure(RuntimeError, "ReceiverRaised")

    def test_previous_module_binding_is_restored_on_success_and_failure(self):
        self.target.write_text("""from roleforge import Role as BaseRole
class Role(BaseRole): pass
def roleforge_receive(role): pass
""", encoding="utf-8")
        module_name = type(roleforge.load(self.source).example).__module__
        marker = object()
        with patch.dict(sys.modules, {module_name: marker}):
            roleforge.load(self.source)
            self.assertIs(sys.modules[module_name], marker)
            self.target.write_text("def roleforge_receive(role): raise ValueError('private detail')", encoding="utf-8")
            self.failure(RuntimeError, "ReceiverRaised")
            self.assertIs(sys.modules[module_name], marker)

    def test_project_construction_failure_keeps_delivery_effects_and_request(self):
        log = self.root / "received.txt"
        self.target.write_text(f"""import sys
from pathlib import Path
def roleforge_receive(role):
    Path({str(log)!r}).write_text('received')
    sys.modules['roleforge._live'] = None
""", encoding="utf-8")
        with patch.dict(sys.modules):
            error = self.failure(RuntimeError, "ProjectConstructionFailed")
        self.assertEqual(error.requested_file, str(self.source))
        self.assertEqual(log.read_text(), "received")

    def test_project_construction_process_control_propagates(self):
        signal = SystemExit(17)
        with patch.object(live_adapter, "_ProjectRoles", side_effect=signal):
            with self.assertRaises(SystemExit) as caught:
                roleforge.load(self.source)
        self.assertIs(caught.exception, signal)

    def test_role_property_failure_during_project_grouping_remains_role_owned(self):
        import builtins
        sentinel = ValueError("Role-owned property failure")
        self.target.write_text("""import builtins
from roleforge import Role as BaseRole
class Role(BaseRole):
    @property
    def name(self):
        raise builtins._roleforge_test_failure
def roleforge_receive(role):
    role.received = True
""", encoding="utf-8")
        with patch.object(builtins, "_roleforge_test_failure", sentinel, create=True):
            with self.assertRaises(ValueError) as caught:
                roleforge.load(self.source)
        self.assertIs(caught.exception, sentinel)
        self.assertFalse(hasattr(caught.exception, "case"))

    def test_broken_output_is_named_and_prevents_receipt(self):
        log = self.root / "received.txt"
        self.target.write_text(f"""from pathlib import Path
def roleforge_receive(role):
    Path({str(log)!r}).write_text('received')
""", encoding="utf-8")
        result = subprocess.run([sys.executable, "-I", "-B", "-c", """
import os
import sys
import roleforge
read_end, write_end = os.pipe()
os.close(read_end)
os.dup2(write_end, 1)
os.close(write_end)
try:
    roleforge.load(sys.argv[1])
except BrokenPipeError as error:
    assert error.case == 'OutputWriteFailed'
    assert error.__cause__ is None
    print('OUTPUT_FAILURE_VERIFIED', file=sys.stderr)
else:
    raise AssertionError('Expected output failure')
""", str(self.source)], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("OUTPUT_FAILURE_VERIFIED", result.stderr)
        self.assertFalse(log.exists())

    def test_core_access_errors_keep_built_in_types_and_case_specific_data(self):
        self.register("Example", "EXAMPLE")
        self.source.write_text("@role Example\n@role EXAMPLE\n", encoding="utf-8")
        project = roleforge.load(self.source)
        with self.assertRaises(KeyError) as caught:
            project.get_role("missing")
        self.assertEqual(caught.exception.case, "LiveRoleNotFound")
        self.assertEqual(caught.exception.role_name, "missing")
        with self.assertRaises(IndexError) as caught:
            project.get_role("Example", -1)
        self.assertEqual(caught.exception.case, "RoleOccurrenceNotFound")
        self.assertEqual(caught.exception.role_index, -1)
        with self.assertRaises(AttributeError) as caught:
            _ = project.missing
        self.assertEqual(caught.exception.case, "LiveRoleAttributeNotFound")
        self.assertEqual(caught.exception.attribute, "missing")
        with self.assertRaises(AttributeError) as caught:
            _ = project.example
        self.assertEqual(caught.exception.case, "AmbiguousRoleAttribute")
        self.assertEqual(caught.exception.role_names, ("Example", "EXAMPLE"))
        self.assertEqual(project.get_role("Example").name, "Example")

    def test_python_protocol_errors_propagate_without_core_reclassification(self):
        sentinel = ValueError("caller path protocol")

        class BadPath:
            def __fspath__(self):
                raise sentinel

        with self.assertRaises(ValueError) as caught:
            roleforge.load(BadPath())
        self.assertIs(caught.exception, sentinel)
        project = roleforge.load(self.source)

        class BadIndex:
            def __index__(self):
                raise sentinel

        with self.assertRaises(ValueError) as caught:
            _ = project.example[BadIndex()]
        self.assertIs(caught.exception, sentinel)
        with self.assertRaises(TypeError):
            project.get_role(123)
        with self.assertRaises(OverflowError):
            project.get_role("Example", 2**100)

    def test_role_failure_after_receipt_belongs_to_role(self):
        self.target.write_text("""from roleforge import Role as BaseRole
failure = ValueError('Role-owned failure')
class Role(BaseRole):
    def action(self):
        raise failure
def roleforge_receive(role):
    role.failure = failure
""", encoding="utf-8")
        role = roleforge.load(self.source).example
        with self.assertRaises(ValueError) as caught:
            role.action()
        self.assertIs(caught.exception, role.failure)
        self.assertFalse(hasattr(caught.exception, "case"))


if __name__ == "__main__":
    unittest.main()
