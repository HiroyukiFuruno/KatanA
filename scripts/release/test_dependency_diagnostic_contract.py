#!/usr/bin/env python3
"""Protect the non-mutating recipe and manifest overlay parser."""

from pathlib import Path
import importlib.util
import tempfile
import unittest


ROOT = Path(__file__).parents[2]
SPEC = importlib.util.spec_from_file_location(
    "real_outdated_contract", ROOT / "scripts/dependency-diagnostic/real_contract.py")
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class DependencyDiagnosticContractTests(unittest.TestCase):
    def test_recipe_is_a_single_non_mutating_tool_call(self) -> None:
        source = (ROOT / "just/maintenance.just").read_text(encoding="utf-8")
        section = source.split("outdated:\n", 1)[1].split("\n\n", 1)[0]
        commands = [line.strip() for line in section.splitlines() if line.strip()]
        self.assertEqual(commands, [
            '@{{RTK_CMD}}python3 -B scripts/dependency-diagnostic/real_contract.py '
            '--root . --cargo "{{CARGO}}"'
        ])
        self.assertIn("dependency-diagnostic:", source)
        self.assertIn("scripts/dependency-diagnostic/fixture_contract.py", source)

    def test_manifest_roundtrip_preserves_nested_patch_and_member_path_tree(self) -> None:
        original = {
            "workspace": {"members": ["app"], "resolver": "2"},
            "patch": {"crates-io": {"fixture": {"path": "vendor/fixture"}}},
            "dependencies": {"app": {"path": "app"}},
            "target": {"cfg(target_os = \"windows\")": {
                "dependencies": {"odd package": {"version": "1.0"}}}},
        }
        rendered = MODULE.serialize_manifest(original)
        self.assertEqual(MODULE.tomllib.loads(rendered), original)

    def test_workspace_exclude_removes_globbed_member(self) -> None:
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            (root / "packages/app").mkdir(parents=True)
            (root / "packages/helper").mkdir()
            (root / "packages/excluded").mkdir()
            manifest = {"workspace": {"members": ["packages/*"],
                                       "exclude": ["packages/excluded"]}}
            members = MODULE._workspace_members(root, manifest)
            self.assertEqual({path.name for path in members}, {"app", "helper"})


if __name__ == "__main__":
    unittest.main()
