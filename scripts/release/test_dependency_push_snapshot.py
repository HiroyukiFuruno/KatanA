"""Whole-tree dependency classification using real committed Git inputs."""

import os
from pathlib import Path
import tempfile
import unittest

from dependency_push_snapshot import DependencyPushSnapshot
from test_downstream_git_object_format import create_record, git


class PushSnapshotTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        create_record(self.root, self.environment)
        self.write("Cargo.lock", 'version=4\n[[package]]\nname="katana-member"\nversion="1.0.0"\n')
        self.write("Cargo.toml", '[workspace]\nmembers=["member"]\nresolver="3"\n'
                   '[workspace.package]\nversion="1.0.0"\n[workspace.dependencies]\nkatana-external="1.0.0"\n')
        self.write("member/Cargo.toml", '[package]\nname="katana-member"\nversion.workspace=true\n'
                   '[dependencies]\nkatana-external.workspace=true\n')
        self.write("member/src/lib.rs", "")

    def write(self, path, content):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(content, encoding="utf-8")

    def snapshot(self):
        git(self.root, self.environment, "add", ".")
        git(self.root, self.environment, "commit", "-qm", "dependency evidence (#335)")
        return DependencyPushSnapshot(self.root, git(self.root, self.environment, "rev-parse", "HEAD"))

    def test_root_only_external_update_reaches_unchanged_member(self):
        before = self.snapshot()
        root = (self.root / "Cargo.toml").read_text().replace('katana-external="1.0.0"', 'katana-external="2.0.0"')
        self.write("Cargo.toml", root)
        after = self.snapshot()
        self.assertEqual(before.context.files["member/Cargo.toml"].blob,
                         after.context.files["member/Cargo.toml"].blob)
        manifests, locks = after.compare(before)
        self.assertEqual({item.after.manifest_path for item in manifests}, {"Cargo.toml", "member/Cargo.toml"})
        self.assertEqual(locks, ())

    def test_source_less_lock_exemption_requires_exact_owned_version(self):
        self.write("Cargo.lock", 'version=4\n[[package]]\nname="katana-member"\nversion="1.0.0"\n'
                   '[[package]]\nname="katana-member"\nversion="2.0.0"\n'
                   '[[package]]\nname="katana-member"\nversion="1.0.0"\nsource="registry+https://example.invalid/index"\n')
        snapshot = self.snapshot()
        rows = snapshot.lockfiles["Cargo.lock"]
        self.assertEqual(len(rows), 2)
        self.assertEqual({(item.version, item.source is not None) for item in rows},
                         {("2.0.0", False), ("1.0.0", True)})

    def test_independent_lock_cannot_borrow_another_workspace_identity(self):
        self.write("tool/Cargo.toml", '[workspace]\n[package]\nname="katana-tool"\nversion="1.0.0"\n')
        self.write("tool/src/lib.rs", "")
        self.write("tool/Cargo.lock", 'version=4\n[[package]]\nname="katana-member"\nversion="1.0.0"\n')
        snapshot = self.snapshot()
        self.assertEqual(snapshot.lockfiles["tool/Cargo.lock"][0].name, "katana-member")

    def test_member_adjacent_lock_cannot_borrow_root_lock_exemption(self):
        lock = 'version=4\n[[package]]\nname="katana-member"\nversion="1.0.0"\ndependencies=[]\n'
        self.write("member/Cargo.lock", lock)
        before = self.snapshot()
        self.write("member/Cargo.lock", lock.replace('dependencies=[]', 'dependencies=["helper"]'))
        manifests, locks = self.snapshot().compare(before)
        self.assertEqual(manifests, ())
        self.assertEqual(len(locks), 1)
        self.assertEqual(locks[0][0], "member/Cargo.lock")
        self.assertEqual(locks[0][1].after.dependencies, ("helper",))

    def test_checksum_only_change_in_nested_lock_is_not_omitted(self):
        lock = ('version=4\n[[package]]\nname="katana-external"\nversion="1.0.0"\n'
                'source="registry+https://example.invalid/index"\nchecksum="old"\n')
        self.write("unregistered/deep/Cargo.lock", lock)
        before = self.snapshot()
        self.write("unregistered/deep/Cargo.lock", lock.replace('checksum="old"', 'checksum="new"'))
        manifests, locks = self.snapshot().compare(before)
        self.assertEqual(manifests, ())
        self.assertEqual(len(locks), 1)
        self.assertEqual(locks[0][0], "unregistered/deep/Cargo.lock")
        self.assertEqual(locks[0][1].after.checksum, "new")

    def test_dirty_manifest_cannot_replace_selected_commit(self):
        before = self.snapshot()
        self.write("Cargo.toml", "not valid toml")
        selected = DependencyPushSnapshot(self.root, before.context.commit)
        self.assertEqual(selected.compare(before), ((), ()))

    def test_unregistered_same_name_manifest_keeps_external_path_candidate(self):
        self.write("fake/Cargo.toml", '[package]\nname="katana-member"\nversion="1.0.0"\n')
        self.write("fake/src/lib.rs", "")
        self.write("loose/Cargo.toml", '[package]\nname="katana-loose"\nversion="1.0.0"\n'
                   '[dependencies]\nalias={package="katana-member",path="../fake"}\n')
        snapshot = self.snapshot()
        self.assertNotIn("fake/Cargo.toml", snapshot.context.packages)
        self.assertEqual(snapshot.manifests["loose/Cargo.toml"][0].declaration.package, "katana-member")


if __name__ == "__main__":
    unittest.main()
