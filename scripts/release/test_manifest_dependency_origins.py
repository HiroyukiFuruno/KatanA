"""Immutable Git inputs retain direct and inherited dependency origins."""

import os
from pathlib import Path
import tempfile
import unittest

from git_dependency_tree import read_dependency_tree
from downstream_dependency_records import DependencySnapshotError
from manifest_dependency_origins import snapshot_manifest_origins
from test_downstream_git_object_format import create_record, git


class ManifestOriginTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        create_record(self.root, self.environment)
        (self.root / "member").mkdir()
        self.write("Cargo.toml", '[workspace]\nmembers=["member"]\n[workspace.dependencies]\n'
                   'alias={package="katana-core",path="core"}\n')
        self.write("member/Cargo.toml", '[package]\nname="katana-member"\nversion="1.0.0"\n'
                   '[dependencies]\nalias={workspace=true}\n')

    def write(self, path, content):
        (self.root / path).write_text(content, encoding="utf-8")

    def commit(self):
        git(self.root, self.environment, "add", ".")
        git(self.root, self.environment, "commit", "-qm", "origin input (#335)")
        commit = git(self.root, self.environment, "rev-parse", "HEAD")
        return {item.path: item for item in read_dependency_tree(self.root, commit)}

    def snapshot(self, files):
        return snapshot_manifest_origins(files["member/Cargo.toml"], files["Cargo.toml"])

    def test_inherited_and_direct_equal_paths_keep_distinct_actual_origins(self):
        inherited_files = self.commit()
        inherited = self.snapshot(inherited_files)[0]
        self.write("member/Cargo.toml", '[dependencies]\n'
                   'alias={package="katana-core",path="core"}\n')
        direct_files = self.commit()
        direct = self.snapshot(direct_files)[0]
        self.assertEqual(inherited.declaration, direct.declaration)
        self.assertEqual(inherited.origin_path, "Cargo.toml")
        self.assertEqual(direct.origin_path, "member/Cargo.toml")
        self.assertEqual(inherited.origin_blob, inherited_files["Cargo.toml"].blob)
        self.assertEqual(direct.origin_blob, direct_files["member/Cargo.toml"].blob)

    def test_same_name_external_path_is_retained_without_internal_exemption(self):
        self.write("Cargo.toml", '[workspace.dependencies]\n'
                   'alias={package="katana-core",path="../sibling"}\n')
        result = self.snapshot(self.commit())
        self.assertEqual(len(result), 1)
        self.assertEqual(result[0].declaration.source.locator, "../sibling")
        self.assertEqual(result[0].declaration.package, "katana-core")

    def test_root_only_change_updates_unchanged_member_snapshot(self):
        before = self.commit()
        self.write("Cargo.toml", '[workspace.dependencies]\n'
                   'alias={package="katana-core",path="elsewhere"}\n')
        after = self.commit()
        self.assertEqual(before["member/Cargo.toml"].blob, after["member/Cargo.toml"].blob)
        self.assertNotEqual(self.snapshot(before), self.snapshot(after))
        self.assertNotEqual(self.snapshot(before)[0].origin_blob, self.snapshot(after)[0].origin_blob)

    def test_dirty_workspace_does_not_replace_committed_origin(self):
        files = self.commit()
        self.write("Cargo.toml", "dirty invalid TOML")
        result = self.snapshot(files)[0]
        self.assertEqual(result.origin_blob, files["Cargo.toml"].blob)
        self.assertEqual(result.declaration.source.locator, "core")

    def test_wrong_kind_and_missing_workspace_alias_fail_closed(self):
        files = self.commit()
        with self.assertRaises(DependencySnapshotError):
            snapshot_manifest_origins(files["Cargo.lock"], files["Cargo.toml"])
        self.write("Cargo.toml", '[workspace.dependencies]\nother="1"\n')
        with self.assertRaises(DependencySnapshotError):
            self.snapshot(self.commit())

    def test_target_build_dev_patch_and_replace_keep_their_manifest_origin(self):
        self.write("member/Cargo.toml", '''
            [target.'cfg(unix)'.dependencies]
            alias={workspace=true}
            [build-dependencies]
            alias={workspace=true}
            [dev-dependencies]
            alias={workspace=true}
            [patch.crates-io]
            katana-core={path="../patched"}
            [replace]
            "katana-core:1.0.0"={path="../replacement"}
            ''')
        results = self.snapshot(self.commit())
        inherited = [item for item in results if item.declaration.alias == "alias"]
        direct = [item for item in results if item.declaration.alias != "alias"]
        self.assertEqual(len(inherited), 3)
        self.assertEqual(len(direct), 2)
        self.assertEqual({item.origin_path for item in inherited}, {"Cargo.toml"})
        self.assertEqual({item.origin_path for item in direct}, {"member/Cargo.toml"})


if __name__ == "__main__":
    unittest.main()
