"""Real Python handoff and conflict preflight through public load().

Run serially: the current Registry reads source-tree storage. Each test lends
that storage fixture data and restores the exact original bytes in finally.
Receivers record actual delivery, including order and immutable input metadata.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
STORAGE = ROOT / "roleforge" / "core" / "storage"

INSPECT_PROJECT = """
import json
import sys
from roleforge import load
project = load(sys.argv[1])
print('RESULT=' + json.dumps([
    dict(name=r.name, index=r.index, role_index=r.role_index, body=r.body,
         declaration_line=r.declaration_line, status=r.status,
         entry=str(r.entry) if r.entry is not None else None,
         builtin_entry=str(r.builtin_entry) if r.builtin_entry is not None else None,
         dynamic_entry=str(r.dynamic_entry) if r.dynamic_entry is not None else None)
    for r in project.roles
]))
"""


class HandoffPreflightTests(unittest.TestCase):
    def run_project(self, source, builtin, dynamic, receiver=None, via="python", missing=False, error=None, relative=False):
        paths = [STORAGE / "builtin_roles.json", STORAGE / "dynamic_roles.json"]
        originals = [path.read_bytes() for path in paths]
        self.received = []
        try:
            with tempfile.TemporaryDirectory(dir=ROOT / "roleforge" if relative else None) as directory:
                log = Path(directory) / "received.jsonl"
                default_receiver = f'''import json
from pathlib import Path

def roleforge_receive(role):
    for obj, attr in [(role, "name"), (role.source, "declaration_line")]:
        try:
            setattr(obj, attr, None)
        except AttributeError:
            pass
        else:
            raise AssertionError("handoff input must be read-only")
    with Path({str(log)!r}).open("a", encoding="utf-8") as stream:
        stream.write(json.dumps(dict(name=role.name, index=role.index,
            role_index=role.role_index, body=role.body,
            declaration_line=role.source.declaration_line)) + "\\n")
    return object()

def start():
    raise AssertionError("start must not be called")
'''
                self.targets = {}
                for target in set(builtin.values()) | set(dynamic.values()):
                    destination = Path(directory) / target
                    self.targets[target] = destination
                    if not missing:
                        destination.parent.mkdir(parents=True, exist_ok=True)
                        implementation = default_receiver if receiver is None else receiver
                        if receiver is None:
                            expected_names = [name for registrations in (builtin, dynamic)
                                              for name, registered in registrations.items() if registered == target]
                            implementation = implementation.replace("def roleforge_receive(role):",
                                f"def roleforge_receive(role):\n    assert role.name in {expected_names!r}")
                        destination.write_text(implementation, encoding="utf-8")
                for path, registrations, base in zip(paths, [builtin, dynamic],
                                                      [ROOT / "roleforge/builtin_roles", ROOT / "roleforge/roles"]):
                    path.write_text(json.dumps({name: {"entry": {"via": via, "target": (
                        os.path.relpath(self.targets[target], base) if relative else str(self.targets[target]))}}
                                                for name, target in registrations.items()}), encoding="utf-8")
                source_path = Path(directory) / "source.rfg"
                source_path.write_bytes(source.encode("utf-8"))
                result = subprocess.run(
                    [sys.executable, "-c", INSPECT_PROJECT, str(source_path)],
                    cwd=directory, capture_output=True, text=True,
                    encoding="utf-8",
                )
                if log.exists():
                    self.received = [json.loads(line) for line in log.read_text(encoding="utf-8").splitlines()]
        finally:
            for path, original in zip(paths, originals):
                path.write_bytes(original)
        if error:
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(error, result.stderr)
            self.assertNotIn("RESULT=", result.stdout)
            return
        self.assertEqual(result.returncode, 0, result.stderr)
        trace, marker, serialized = result.stdout.rpartition("RESULT=")
        self.assertTrue(marker, result.stdout)
        return trace, json.loads(serialized)

    def test_resolved_metadata_remains_in_source_order(self):
        trace, roles = self.run_project(
            "@role ThirdParty_06\nשלום # opaque\n@role Other_06\nconfig\n@role ThirdParty_06\nlast",
            {"Other_06": "other/entry"}, {"ThirdParty_06": "external/entry"},
        )
        self.assertEqual(self.received, [{key: r[key] for key in ("name", "index", "role_index", "body", "declaration_line")} for r in roles])
        self.assertEqual([r["name"] for r in roles],
                         ["ThirdParty_06", "Other_06", "ThirdParty_06"])
        self.assertEqual([r["index"] for r in roles], [0, 1, 2])
        self.assertEqual([r["role_index"] for r in roles], [0, 0, 1])
        self.assertEqual([r["declaration_line"] for r in roles], [1, 3, 5])
        self.assertEqual([r["body"] for r in roles], ["שלום # opaque\n", "config\n", "last"])
        self.assertEqual([r["status"] for r in roles], ["resolved"] * 3)
        self.assertEqual([Path(r["entry"]) for r in roles], [
            self.targets["external/entry"],
            self.targets["other/entry"],
            self.targets["external/entry"],
        ])
        self.assertEqual([line for line in trace.splitlines() if line.startswith("Role: ")],
                         ["Role: ThirdParty_06", "Role: Other_06", "Role: ThirdParty_06"])
        self.assertIn("שלום # opaque", trace)
        self.assertNotIn("Handoff aborted", trace)

    def test_unknown_is_reported_and_later_resolved_role_is_processed(self):
        trace, roles = self.run_project(
            "@role ThirdParty_06\nfirst\n@role Missing_06\nunknown\n@role ThirdParty_06\nlast",
            {}, {"ThirdParty_06": "external/entry"},
        )
        self.assertEqual([r["status"] for r in roles], ["resolved", "unknown", "resolved"])
        self.assertEqual([r["index"] for r in roles], [0, 1, 2])
        self.assertEqual([r["role_index"] for r in roles], [0, 0, 1])
        self.assertLess(trace.index("Global Index: 0"), trace.index("Unknown Role: Missing_06"))
        self.assertLess(trace.index("Unknown Role: Missing_06"), trace.index("Global Index: 2"))
        self.assertEqual(trace.count("[CORE DEBUG]"), 2)
        self.assertEqual([r["index"] for r in self.received], [0, 2])
        self.assertNotIn("Handoff aborted", trace)

    def test_late_conflict_prevents_processing_even_earlier_resolved_roles(self):
        trace, roles = self.run_project(
            "@role ThirdParty_06\none\n@role ThirdParty_06\ntwo\n@role Shared_06\nconflict\n@role ThirdParty_06\nlast",
            {"Shared_06": "builtin/entry"},
            {"Shared_06": "dynamic/entry", "ThirdParty_06": "external/entry"},
        )
        self.assertEqual([r["status"] for r in roles],
                         ["resolved", "resolved", "conflict", "resolved"])
        self.assertEqual([r["role_index"] for r in roles], [0, 1, 0, 2])
        self.assertEqual(self.received, [])
        self.assertNotIn("[CORE DEBUG]", trace)
        self.assertNotIn("ThirdParty_06", trace)
        self.assertIn("Role conflict: Shared_06", trace)
        self.assertIn("Handoff aborted.", trace)
        self.assertIsNone(roles[2]["entry"])
        self.assertEqual(Path(roles[2]["builtin_entry"]),
                         self.targets["builtin/entry"])
        self.assertEqual(Path(roles[2]["dynamic_entry"]),
                         self.targets["dynamic/entry"])
        printed_entries = dict(line.split(": ", 1) for line in trace.splitlines()
                               if line.startswith(("Builtin entry:", "Dynamic entry:")))
        self.assertEqual(Path(printed_entries["Builtin entry"]), Path(roles[2]["builtin_entry"]))
        self.assertEqual(Path(printed_entries["Dynamic entry"]), Path(roles[2]["dynamic_entry"]))

    def test_all_conflicts_are_reported_before_aborting(self):
        trace, roles = self.run_project(
            "@role A_06\n@role Valid_06\n@role B_06\n",
            {"A_06": "a/builtin", "B_06": "b/builtin"},
            {"A_06": "a/dynamic", "B_06": "b/dynamic", "Valid_06": "valid/entry"},
        )
        self.assertEqual([r["status"] for r in roles], ["conflict", "resolved", "conflict"])
        self.assertEqual(self.received, [])
        self.assertNotIn("[CORE DEBUG]", trace)
        self.assertEqual(trace.count("Role conflict:"), 2)
        self.assertLess(trace.index("Role conflict: A_06"), trace.index("Role conflict: B_06"))
        self.assertLess(trace.index("Role conflict: B_06"), trace.index("Handoff aborted."))

    def test_receiving_contract_failures_are_distinct(self):
        for receiver, error in [
            ("def main(): pass", "MissingReceiver"),
            ("roleforge_receive = 123", "ReceiverNotCallable"),
            ("def roleforge_receive(role): raise ValueError('receiver boom')", "ReceiverRaised"),
            ("invalid python !!!", "PythonTargetLoadFailure"),
            ("raise RuntimeError('import boom')", "PythonTargetLoadFailure"),
        ]:
            with self.subTest(error=error, receiver=receiver):
                self.run_project("@role Example\nbody", {}, {"Example": "main.py"}, receiver=receiver, error=error)
                self.assertEqual(self.received, [])

    def test_missing_target_fails(self):
        self.run_project("@role Example\nbody", {}, {"Example": "missing.py"}, missing=True, error="PythonTargetLoadFailure")

    def test_unknown_bridge_never_infers_python_from_extension(self):
        for via in ["hii_im_boby", "rust", "Python"]:
            with self.subTest(via=via):
                self.run_project("@role Example\nbody", {}, {"Example": "main.py"}, via=via, error="UnknownBridge")
                self.assertEqual(self.received, [])

    def test_same_basename_targets_do_not_collide(self):
        self.run_project("@role First\nfirst\n@role Second\nsecond", {},
                         {"First": "one/main.py", "Second": "two/main.py"})
        self.assertEqual([r["name"] for r in self.received], ["First", "Second"])

    def test_relative_targets_use_registry_bases_from_another_cwd(self):
        _, roles = self.run_project("@role First\nfirst\n@role Second\nsecond",
                                    {"First": "one/main.py"}, {"Second": "two/main.py"}, relative=True)
        self.assertEqual([r["name"] for r in self.received], ["First", "Second"])
        self.assertEqual([Path(r["entry"]).resolve() for r in roles],
                         [self.targets["one/main.py"], self.targets["two/main.py"]])


if __name__ == "__main__":
    unittest.main()
