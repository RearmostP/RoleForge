"""Stage 09 public API tests. Registry fixtures must be used serially."""
import gc
import json
from pathlib import Path
import sys
import tempfile
import unittest
import weakref

from roleforge import Role, RoleInfo, load


import roleforge

PACKAGE_ROOT = Path(roleforge.__file__).resolve().parent
STORAGE = PACKAGE_ROOT / "core/storage"
IMPLEMENTATION = '''from roleforge import Role as BaseRole

def describe(role):
    return role.name, role.index, role.role_index, role.body, role.source.declaration_line

class Role(BaseRole):
    def hello(self):
        self.calls.append(self.body)
        return describe(self)

    def start(self):
        raise AssertionError("start must not be called")

def roleforge_receive(role):
    role.calls = []
    role.received_identity = id(role)
    return "ignored return value"
'''


class LiveRoleTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.registries = [STORAGE / f"{name}_roles.json" for name in ("builtin", "dynamic")]
        for path in self.registries:
            self.addCleanup(path.write_bytes, path.read_bytes())
            path.write_text("{}", encoding="utf-8")
        self.target = self.root / "main.py"
        self.target.write_text(IMPLEMENTATION, encoding="utf-8")
        self.register("Test", "Config")

    def register(self, *names, builtin=False):
        self.registries[0 if builtin else 1].write_text(json.dumps({
            name: {"entry": {"via": "python", "target": str(self.target)}}
            for name in names
        }), encoding="utf-8")

    def project(self, source):
        path = self.root / "source.rfg"
        path.write_bytes(source.encode("utf-8"))
        return load(path)

    def test_single_instance_is_the_receiver_object_and_keeps_metadata(self):
        project = self.project("@role Test\nfirst")
        role = project.test
        self.assertIsInstance(role, Role)
        self.assertIs(role, project.test[0])
        self.assertIs(role, project.get_role("Test"))
        self.assertEqual(id(role), role.received_identity)
        self.assertEqual(role.hello(), ("Test", 0, 0, "first", 1))
        self.assertIsInstance(project.roles[0], RoleInfo)
        self.assertIsNot(role, project.roles[0])
        self.assertEqual(project.roles[0].status, "resolved")
        for field in ("name", "index", "role_index", "body", "source", "role_input"):
            with self.subTest(field=field), self.assertRaises(AttributeError):
                setattr(role, field, None)
        with self.assertRaises(AttributeError):
            role.source.declaration_line = 9

    def test_interleaved_instances_keep_core_identity_and_independent_state(self):
        project = self.project("@role Test\nfirst\n@role Config\nconfig\n@role Test\nsecond")
        first, second = project.test, project.test[1]
        self.assertIsNot(first, second)
        self.assertIs(second, project.get_role("Test", 1))
        self.assertEqual(first.hello(), ("Test", 0, 0, "first\n", 1))
        self.assertEqual(second.hello(), ("Test", 2, 1, "second", 5))
        self.assertEqual(project.config.hello(), ("Config", 1, 0, "config\n", 3))
        first.hello()
        self.assertEqual(first.calls, ["first\n", "first\n"])
        self.assertEqual(second.calls, ["second"])
        self.assertEqual([(r.index, r.role_index) for r in project.roles], [(0, 0), (1, 0), (2, 1)])

    def test_live_object_and_module_globals_survive_project_and_target_removal(self):
        project = self.project("@role Test\nfirst\n@role Test\nsecond")
        role = project.test[1]
        module_name = type(role).__module__
        self.assertNotIn(module_name, sys.modules)
        del project
        self.target.unlink()
        gc.collect()
        self.assertEqual(role.hello(), ("Test", 1, 1, "second", 3))
        reference = weakref.ref(role)
        del role
        gc.collect()
        self.assertIsNone(reference())

    def test_missing_and_invalid_access(self):
        project = self.project("@role Test\n")
        with self.assertRaises(AttributeError):
            _ = project.nonexistent
        with self.assertRaises(KeyError):
            project.get_role("test")
        for index in (-1, 1, 999):
            with self.subTest(index=index), self.assertRaises(IndexError):
                _ = project.test[index]
        for index in ("1", 1.5, slice(None)):
            with self.subTest(index=index), self.assertRaises(TypeError):
                _ = project.test[index]

    def test_unknown_roles_never_create_live_objects_or_reset_indexes(self):
        project = self.project("@role Missing\n@role Test\nfirst\n@role Missing\n@role Test\nlast")
        self.assertFalse(hasattr(project, "missing"))
        with self.assertRaises(KeyError):
            project.get_role("Missing")
        self.assertEqual([project.test[i].index for i in (0, 1)], [1, 3])

    def test_conflict_preflight_prevents_target_loading_and_all_live_objects(self):
        self.register("Config", builtin=True)
        self.target.write_text("raise AssertionError('must not import')", encoding="utf-8")
        project = self.project("@role Test\n@role Config\n")
        self.assertEqual([r.status for r in project.roles], ["resolved", "conflict"])
        self.assertFalse(hasattr(project, "test"))
        self.assertFalse(hasattr(project, "config"))
        with self.assertRaises(KeyError):
            project.get_role("Test")

    def test_case_collisions_reserved_names_and_unusual_names_have_exact_access(self):
        self.register("Test", "TEST", "Roles", "Path", "Get_role", "a-b", "Class", "CamelCase")
        project = self.project("".join(f"@role {name}\n" for name in
            ("Test", "TEST", "Roles", "Path", "Get_role", "a-b", "Class", "CamelCase")))
        with self.assertRaisesRegex(AttributeError, "Ambiguous"):
            _ = project.test
        self.assertIsInstance(project.roles, tuple)
        self.assertIsInstance(project.path, Path)
        self.assertTrue(callable(project.get_role))
        for name in ("Test", "TEST", "Roles", "Path", "Get_role", "a-b", "Class"):
            self.assertEqual(project.get_role(name).name, name)
        self.assertEqual(project.camelcase.name, "CamelCase")
        self.assertFalse(hasattr(project, "camel_case"))

    def test_legacy_receiver_without_class_retains_a_live_object_and_ignores_return(self):
        self.target.write_text("def roleforge_receive(role):\n    return object()\n", encoding="utf-8")
        project = self.project("@role Test\nbody")
        self.assertIs(type(project.test), Role)
        self.assertEqual(project.test.body, "body")

    def test_separate_loads_do_not_share_instances(self):
        first = self.project("@role Test\nfirst")
        second = self.project("@role Test\nsecond")
        self.assertIsNot(first.test, second.test)
        first.test.hello()
        self.assertEqual(second.test.calls, [])
        self.assertEqual(second.test.body, "second")

    def test_receiver_failure_stops_later_delivery_without_rolling_back(self):
        log = self.root / "received.txt"
        self.target.write_text(f'''from pathlib import Path
def roleforge_receive(role):
    if role.role_index == 1:
        raise ValueError("stop here")
    with Path({str(log)!r}).open("a") as stream:
        stream.write(str(role.role_index))
''', encoding="utf-8")
        with self.assertRaisesRegex(RuntimeError, "ReceiverRaised.*stop here"):
            self.project("@role Test\n@role Test\n@role Test\n")
        self.assertEqual(log.read_text(), "0")

    def test_invalid_role_class_is_a_contextual_conversion_error(self):
        self.target.write_text("Role = 123\ndef roleforge_receive(role): pass\n", encoding="utf-8")
        with self.assertRaisesRegex(RuntimeError, "Role Test.*InputConversion"):
            self.project("@role Test\n")

    def test_development_test_role_exposes_hello_on_each_instance(self):
        self.target = PACKAGE_ROOT / "roles/Test/main.py"
        self.register("Test")
        project = self.project("@role Test\nfirst\n@role Test\nsecond")
        self.assertEqual(project.test.hello(), "first\n")
        self.assertEqual(project.test[0].hello(), "first\n")
        self.assertEqual(project.test[1].hello(), "second")
        self.assertEqual((project.test.calls, project.test[1].calls), (2, 1))


if __name__ == "__main__":
    unittest.main()
