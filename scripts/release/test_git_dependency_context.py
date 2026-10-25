"""Path candidates bind only to regular manifests from the same Git commit."""

import os
from pathlib import Path
import tempfile
import unittest

from downstream_dependency_records import DependencySnapshotError
from git_dependency_context import GitDependencyContext
from test_downstream_git_object_format import create_record, git


class GitDependencyContextTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        create_record(self.root, self.environment)
        self.write("Cargo.toml", '[workspace]\nmembers=["member"]\n[workspace.dependencies]\n'
                   'alias={package="katana-core",path="core"}\n')
        self.write("member/Cargo.toml", '[dependencies]\nalias={workspace=true}\n')
        self.write("core/Cargo.toml", '[package]\nname="katana-core"\nversion="1.0.0"\n')

    def write(self, path, content):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(content, encoding="utf-8")

    def commit(self):
        git(self.root, self.environment, "add", ".")
        git(self.root, self.environment, "commit", "-qm", "path context (#335)")
        return git(self.root, self.environment, "rev-parse", "HEAD")

    def binding(self, commit):
        return GitDependencyContext(self.root, commit).bindings("member/Cargo.toml", "Cargo.toml")[0]

    def test_inherited_path_uses_provider_and_direct_path_uses_member(self):
        inherited = self.binding(self.commit())
        self.assertEqual(inherited.target.path, "core/Cargo.toml")
        self.write("member/core/Cargo.toml", '[package]\nname="katana-core"\nversion="2.0.0"\n')
        self.write("member/Cargo.toml", '[dependencies]\n'
                   'alias={package="katana-core",path="core"}\n')
        direct = self.binding(self.commit())
        self.assertEqual(direct.target.path, "member/core/Cargo.toml")
        self.assertNotEqual(direct.target.blob, inherited.target.blob)

    def test_legitimate_parent_components_stay_inside_repository(self):
        self.write("member/Cargo.toml", '[dependencies]\n'
                   'alias={package="katana-core",path="../core"}\n')
        self.assertEqual(self.binding(self.commit()).target.path, "core/Cargo.toml")

    def test_external_same_name_is_retained_without_a_target(self):
        for path in ("../../outside", "../../repo/core", "/outside", "C:/outside", "C:outside",
                     "//server/share", "..\\core"):
            with self.subTest(path=path):
                self.write("member/Cargo.toml", '[dependencies]\n'
                           f'alias={{package="katana-core",path=\'{path}\'}}\n')
                binding = self.binding(self.commit())
                self.assertIsNone(binding.target)
                self.assertEqual(binding.origin.declaration.source.locator, path)

    def test_inside_repository_missing_or_wrong_package_is_rejected(self):
        for value in ('[package]\nname="wrong-name"\n', '[dependencies]\nkatana-x="1"\n'):
            self.write("core/Cargo.toml", value)
            with self.assertRaises(DependencySnapshotError):
                self.binding(self.commit())
        self.write("Cargo.toml", '[workspace.dependencies]\n'
                   'alias={package="katana-core",path="absent"}\n')
        with self.assertRaises(DependencySnapshotError):
            self.binding(self.commit())

    def test_dirty_replacement_cannot_supply_a_target_blob(self):
        commit = self.commit()
        expected = git(self.root, self.environment, "rev-parse", "HEAD:core/Cargo.toml")
        self.write("core/Cargo.toml", "dirty invalid TOML")
        binding = self.binding(commit)
        self.assertEqual(binding.commit, commit)
        self.assertEqual(binding.target.blob, expected)

    def test_target_is_bound_to_requested_commit_not_latest_worktree(self):
        before = self.commit()
        self.write("core/Cargo.toml", '[package]\nname="katana-core"\nversion="2.0.0"\n')
        after = self.commit()
        self.assertEqual(self.binding(before).commit, before)
        self.assertEqual(self.binding(after).commit, after)
        self.assertNotEqual(self.binding(before).target.blob, self.binding(after).target.blob)

    def test_registry_candidate_is_not_reclassified_as_an_internal_path(self):
        self.write("member/Cargo.toml", '[dependencies]\nkatana-core="1.0.0"\n')
        binding = self.binding(self.commit())
        self.assertIsNone(binding.target)
        self.assertEqual(binding.origin.declaration.source.kind, "registry")

    def test_multiple_aliases_reuse_one_target_package_parse(self):
        aliases = "".join(f'alias{index}={{package="katana-core",path="../core"}}\n'
                          for index in range(128))
        self.write("member/Cargo.toml", "[dependencies]\n" + aliases)
        context = GitDependencyContext(self.root, self.commit())
        bindings = context.bindings("member/Cargo.toml")
        self.assertEqual(len(bindings), 128)
        self.assertTrue(all(binding.target is bindings[0].target for binding in bindings))
        self.assertEqual({binding.target.path for binding in bindings}, {"core/Cargo.toml"})

    def test_provider_change_is_bound_to_each_selected_commit(self):
        before = self.commit()
        self.write("other/Cargo.toml", '[package]\nname="katana-core"\nversion="2.0.0"\n')
        self.write("Cargo.toml", '[workspace.dependencies]\n'
                   'alias={package="katana-core",path="other"}\n')
        after = self.commit()
        self.assertEqual(self.binding(before).target.path, "core/Cargo.toml")
        self.assertEqual(self.binding(after).target.path, "other/Cargo.toml")


if __name__ == "__main__":
    unittest.main()
