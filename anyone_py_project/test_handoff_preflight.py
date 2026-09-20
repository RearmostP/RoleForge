"""Stage 06 preflight tests, not proof of physical Role delivery.

Run serially: the current Registry reads source-tree storage. Each test lends
that storage fixture data and restores the exact original bytes in finally.
No Role destination is loaded or executed; the receiving protocol is undecided.
"""
import json
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
    def run_project(self, source, builtin, dynamic):
        paths = [STORAGE / "builtin_roles.json", STORAGE / "dynamic_roles.json"]
        originals = [path.read_bytes() for path in paths]
        try:
            for path, registrations in zip(paths, [builtin, dynamic]):
                path.write_text(json.dumps({name: {"entry": entry}
                                            for name, entry in registrations.items()}),
                                encoding="utf-8")
            with tempfile.TemporaryDirectory() as directory:
                source_path = Path(directory) / "source.rfg"
                source_path.write_bytes(source.encode("utf-8"))
                result = subprocess.run(
                    [sys.executable, "-c", INSPECT_PROJECT, str(source_path)],
                    cwd=directory, capture_output=True, text=True,
                    encoding="utf-8", check=True,
                )
        finally:
            for path, original in zip(paths, originals):
                path.write_bytes(original)
        trace, marker, serialized = result.stdout.rpartition("RESULT=")
        self.assertTrue(marker, result.stdout)
        return trace, json.loads(serialized)

    def test_resolved_metadata_remains_in_source_order(self):
        trace, roles = self.run_project(
            "@role ThirdParty_06\nשלום # opaque\n@role Other_06\nconfig\n@role ThirdParty_06\nlast",
            {"Other_06": "other/entry"}, {"ThirdParty_06": "external/entry"},
        )
        self.assertEqual([r["name"] for r in roles],
                         ["ThirdParty_06", "Other_06", "ThirdParty_06"])
        self.assertEqual([r["index"] for r in roles], [0, 1, 2])
        self.assertEqual([r["role_index"] for r in roles], [0, 0, 1])
        self.assertEqual([r["declaration_line"] for r in roles], [1, 3, 5])
        self.assertEqual([r["body"] for r in roles], ["שלום # opaque\n", "config\n", "last"])
        self.assertEqual([r["status"] for r in roles], ["resolved"] * 3)
        self.assertEqual([Path(r["entry"]) for r in roles], [
            ROOT / "roleforge/roles/external/entry",
            ROOT / "roleforge/builtin_roles/other/entry",
            ROOT / "roleforge/roles/external/entry",
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
        self.assertNotIn("[CORE DEBUG]", trace)
        self.assertNotIn("ThirdParty_06", trace)
        self.assertIn("Role conflict: Shared_06", trace)
        self.assertIn("Handoff aborted.", trace)
        self.assertIsNone(roles[2]["entry"])
        self.assertEqual(Path(roles[2]["builtin_entry"]),
                         ROOT / "roleforge/builtin_roles/builtin/entry")
        self.assertEqual(Path(roles[2]["dynamic_entry"]),
                         ROOT / "roleforge/roles/dynamic/entry")
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
        self.assertNotIn("[CORE DEBUG]", trace)
        self.assertEqual(trace.count("Role conflict:"), 2)
        self.assertLess(trace.index("Role conflict: A_06"), trace.index("Role conflict: B_06"))
        self.assertLess(trace.index("Role conflict: B_06"), trace.index("Handoff aborted."))


if __name__ == "__main__":
    unittest.main()
