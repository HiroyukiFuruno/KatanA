"""Actual committed workspace ownership, compared to native Cargo membership."""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

from downstream_dependency_records import DependencySnapshotError
from git_workspace_context import GitWorkspaceContext
from test_downstream_git_object_format import create_record, git


class WorkspaceContextTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        create_record(self.root, self.environment)
        self.write("Cargo.toml", '[workspace]\nmembers=["member"]\nresolver="3"\n'
                   '[workspace.package]\nversion="1.0.0"\n[workspace.dependencies]\n'
                   'alias={package="katana-core",path="core"}\n')
        self.write("member/Cargo.toml", '[package]\nname="katana-member"\nversion.workspace=true\n'
                   '[dependencies]\nalias={workspace=true}\n')
        self.write("core/Cargo.toml", '[package]\nname="katana-core"\nversion.workspace=true\n')
        for directory in ("member", "core"):
            self.write(f"{directory}/src/lib.rs", "")

    def write(self, path, content):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(content, encoding="utf-8")

    def context(self):
        git(self.root, self.environment, "add", ".")
        git(self.root, self.environment, "commit", "-qm", "workspace registration (#335)")
        return GitWorkspaceContext(self.root, git(self.root, self.environment, "rev-parse", "HEAD"))

    def cargo_members(self):
        result = subprocess.run(["cargo", "metadata", "--offline", "--no-deps", "--format-version", "1"],
                                cwd=self.root, env=self.environment, capture_output=True,
                                text=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr)
        document = json.loads(result.stdout)
        return {Path(package["manifest_path"]).relative_to(self.root).as_posix()
                for package in document["packages"] if package["id"] in document["workspace_members"]}

    def test_inherited_path_auto_member_and_version_match_native_cargo(self):
        context = self.context()
        self.assertEqual(set(context.packages), self.cargo_members())
        binding = context.bindings("member/Cargo.toml")[0]
        self.assertEqual(binding.internal.manifest_path, "core/Cargo.toml")
        self.assertEqual(binding.internal.version, "1.0.0")
        self.assertEqual(binding.internal.workspace_path, "Cargo.toml")

    def test_excluded_same_name_path_retains_downstream_candidate(self):
        root = (self.root / "Cargo.toml").read_text().replace('resolver="3"', 'exclude=["core"]\nresolver="3"')
        self.write("Cargo.toml", root)
        self.write("core/Cargo.toml", '[package]\nname="katana-core"\nversion="1.0.0"\n')
        context = self.context()
        self.assertEqual(set(context.packages), self.cargo_members())
        binding = context.bindings("member/Cargo.toml")[0]
        self.assertIsNone(binding.internal)
        self.assertEqual(binding.origin.declaration.package, "katana-core")

    def test_unused_workspace_path_definition_does_not_register_package(self):
        self.write("member/Cargo.toml", '[package]\nname="katana-member"\nversion.workspace=true\n')
        context = self.context()
        self.assertEqual(set(context.packages), self.cargo_members())
        self.assertNotIn("core/Cargo.toml", context.packages)

    def test_nested_independent_workspace_keeps_its_own_version(self):
        self.write("tool/Cargo.toml", '[workspace]\n[package]\nname="katana-tool"\nversion="2.0.0"\n')
        self.write("tool/src/lib.rs", "")
        self.write("member/Cargo.toml", '[package]\nname="katana-member"\nversion.workspace=true\n'
                   '[dependencies]\nkatana-tool={path="../tool"}\n')
        root = (self.root / "Cargo.toml").read_text().replace('resolver="3"', 'exclude=["tool"]\nresolver="3"')
        self.write("Cargo.toml", root)
        context = self.context()
        binding = context.bindings("member/Cargo.toml")[0]
        self.assertEqual(binding.internal.workspace_path, "tool/Cargo.toml")
        self.assertEqual(binding.internal.version, "2.0.0")
        self.assertEqual(self.cargo_members(), {"member/Cargo.toml"})

    def test_unregistered_same_name_directory_cannot_supply_internal_identity(self):
        self.write("member/Cargo.toml", '[package]\nname="katana-member"\nversion.workspace=true\n')
        self.write("fake/Cargo.toml", '[package]\nname="katana-core"\nversion="1.0.0"\n')
        context = self.context()
        self.assertNotIn("fake/Cargo.toml", context.packages)

    def test_non_katana_path_dependency_still_registers_auto_member(self):
        self.write("core/Cargo.toml", '[package]\nname="utility-helper"\nversion.workspace=true\n')
        self.write("member/Cargo.toml", '[package]\nname="katana-member"\nversion.workspace=true\n'
                   '[dependencies]\nhelper={package="utility-helper",path="../core"}\n')
        context = self.context()
        self.assertEqual(set(context.packages), self.cargo_members())
        self.assertEqual(context.packages["core/Cargo.toml"].name, "utility-helper")

    def test_invalid_inheritance_and_explicit_workspace_are_rejected(self):
        for value in ('version={workspace=1}', 'version="1.0.0"\nworkspace="../tool"'):
            self.write("core/Cargo.toml", '[package]\nname="katana-core"\n' + value + "\n")
            with self.subTest(value=value), self.assertRaises(DependencySnapshotError):
                self.context()

    def test_root_only_version_change_rebinds_unchanged_member(self):
        before = self.context()
        root = (self.root / "Cargo.toml").read_text().replace('version="1.0.0"', 'version="2.0.0"')
        self.write("Cargo.toml", root)
        after = self.context()
        self.assertEqual(before.files["member/Cargo.toml"].blob, after.files["member/Cargo.toml"].blob)
        self.assertEqual(before.packages["core/Cargo.toml"].version, "1.0.0")
        self.assertEqual(after.packages["core/Cargo.toml"].version, "2.0.0")

    def test_nested_workspace_dependency_requires_parent_exclusion(self):
        self.write("tool/Cargo.toml", '[workspace]\n[package]\nname="katana-tool"\nversion="2.0.0"\n')
        self.write("tool/src/lib.rs", "")
        self.write("member/Cargo.toml", '[package]\nname="katana-member"\nversion.workspace=true\n'
                   '[dependencies]\nkatana-tool={path="../tool"}\n')
        result = subprocess.run(["cargo", "metadata", "--offline", "--no-deps", "--format-version", "1"],
                                cwd=self.root, env=self.environment, capture_output=True, text=True, timeout=30)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("multiple workspace roots", result.stderr)
        with self.assertRaises(DependencySnapshotError):
            self.context()

    def test_member_glob_does_not_cross_path_separator(self):
        self.write("Cargo.toml", '[workspace]\nmembers=["./packages/*"]\nexclude=["packages/nested"]\nresolver="3"\n')
        self.write("packages/one/Cargo.toml", '[package]\nname="katana-one"\nversion="1.0.0"\n')
        self.write("packages/one/src/lib.rs", "")
        self.write("packages/nested/two/Cargo.toml", '[package]\nname="katana-two"\nversion="1.0.0"\n')
        self.write("packages/nested/two/src/lib.rs", "")
        context = self.context()
        self.assertEqual(set(context.packages), self.cargo_members())
        self.assertNotIn("packages/nested/two/Cargo.toml", context.packages)

    def test_glob_matching_source_only_directory_is_rejected_like_cargo(self):
        self.write("Cargo.toml", '[workspace]\nmembers=["packages/*"]\nresolver="3"\n')
        self.write("packages/one/Cargo.toml", '[package]\nname="katana-one"\nversion="1.0.0"\n')
        self.write("packages/one/src/lib.rs", "")
        self.write("packages/no-manifest/source.rs", "")
        with self.assertRaises(DependencySnapshotError):
            self.context()

    def test_path_version_mismatch_is_rejected_like_native_cargo(self):
        self.write("member/Cargo.toml", '[package]\nname="katana-member"\nversion.workspace=true\n'
                   '[dependencies]\nalias={package="katana-core",path="../core",version="=2.0.0"}\n')
        context = self.context()
        result = subprocess.run(["cargo", "metadata", "--offline", "--format-version", "1"],
                                cwd=self.root, env=self.environment, capture_output=True, text=True, timeout=30)
        self.assertNotEqual(result.returncode, 0)
        with self.assertRaises(DependencySnapshotError):
            context.bindings("member/Cargo.toml")

    def test_caret_requirement_accepts_compatible_nonidentical_version_like_cargo(self):
        self.write("core/Cargo.toml", '[package]\nname="katana-core"\nversion="0.22.42"\n')
        self.write("member/Cargo.toml", '[package]\nname="katana-member"\nversion.workspace=true\n'
                   '[dependencies]\nalias={package="katana-core",path="../core",version="0.22.39"}\n')
        context = self.context()
        self.assertEqual(set(context.packages), self.cargo_members())
        self.assertEqual(context.bindings("member/Cargo.toml")[0].internal.version, "0.22.42")

    def test_omitted_package_version_matches_native_cargo_default(self):
        self.write("core/Cargo.toml", '[package]\nname="katana-core"\n')
        result = subprocess.run(["cargo", "metadata", "--offline", "--no-deps", "--format-version", "1"],
                                cwd=self.root, env=self.environment, capture_output=True, text=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr)
        package = next(item for item in json.loads(result.stdout)["packages"] if item["name"] == "katana-core")
        self.assertEqual(package["version"], "0.0.0")
        self.assertEqual(self.context().packages["core/Cargo.toml"].version, package["version"])

    def test_excluded_directory_descendant_is_not_an_auto_member(self):
        self.write("Cargo.toml", '[workspace]\nmembers=["member"]\nexclude=["excluded"]\nresolver="3"\n')
        self.write("member/Cargo.toml", '[package]\nname="katana-member"\nversion="1.0.0"\n'
                   '[dependencies]\nalias={package="katana-excluded",path="../excluded/nested"}\n')
        self.write("excluded/nested/Cargo.toml", '[package]\nname="katana-excluded"\nversion="1.0.0"\n')
        self.write("excluded/nested/src/lib.rs", "")
        context = self.context()
        self.assertEqual(set(context.packages), self.cargo_members())
        self.assertIsNone(context.bindings("member/Cargo.toml")[0].internal)

    def test_explicit_member_takes_precedence_over_exclusion(self):
        root = (self.root / "Cargo.toml").read_text().replace('members=["member"]',
                                                               'members=["member","core"]\nexclude=["core"]')
        self.write("Cargo.toml", root)
        context = self.context()
        self.assertEqual(set(context.packages), self.cargo_members())
        self.assertEqual(context.bindings("member/Cargo.toml")[0].internal.manifest_path, "core/Cargo.toml")


if __name__ == "__main__":
    unittest.main()
