"""Actual isolated Git dependency-tree discovery and immutable content checks."""

import os
from pathlib import Path
import subprocess
import tempfile
import unittest
import zlib

from downstream_git_binding import GitBindingError
from git_dependency_tree import (MAX_DEPENDENCY_BLOB_BYTES, MAX_DEPENDENCY_FILES,
                                 MAX_DEPENDENCY_TREE_BYTES,
                                 dependency_tree_changes, read_dependency_tree)
from test_downstream_git_object_format import create_record, git


class GitDependencyTreeTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        document = create_record(self.root, self.environment)
        self.base = document["changes"][0]["commit"]

    def commit(self):
        git(self.root, self.environment, "add", ".")
        git(self.root, self.environment, "commit", "-qm", "dependency change (#335)")
        return git(self.root, self.environment, "rev-parse", "HEAD")

    def test_tree_uses_committed_bytes_not_dirty_worktree(self):
        expected = (self.root / "Cargo.toml").read_bytes()
        (self.root / "Cargo.toml").write_text("dirty invalid TOML", encoding="utf-8")
        records = {item.path: item for item in read_dependency_tree(self.root, self.base)}
        self.assertEqual(records["Cargo.toml"].content, expected)

    def test_same_size_corrupted_loose_blob_is_rejected(self):
        oid = git(self.root, self.environment, "rev-parse", f"{self.base}:Cargo.toml")
        path = self.root / ".git" / "objects" / oid[:2] / oid[2:]
        header, content = zlib.decompress(path.read_bytes()).split(b"\0", 1)
        changed = content.replace(b"fixture", b"changed")
        self.assertNotEqual(content, changed)
        self.assertEqual(len(content), len(changed))
        path.chmod(0o600)
        path.write_bytes(zlib.compress(header + b"\0" + changed))
        self.assertEqual(git(self.root, self.environment, "cat-file", "-s", oid), str(len(content)))
        self.assertEqual(git(self.root, self.environment, "cat-file", "blob", oid), changed.decode().strip())
        with self.assertRaisesRegex(GitBindingError, "identity"):
            read_dependency_tree(self.root, self.base)

    def test_sha1_blob_content_is_accepted(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            git(root, self.environment, "init", "-q", "--object-format=sha1")
            for key, value in (("user.name", "Tree Test"), ("user.email", "tree@example.invalid"),
                               ("commit.gpgsign", "false"), ("core.hooksPath", str(root / "hooks"))):
                git(root, self.environment, "config", key, value)
            content = b"[package]\nname='fixture'\n"
            (root / "Cargo.toml").write_bytes(content)
            git(root, self.environment, "add", ".")
            git(root, self.environment, "commit", "-qm", "SHA1 fixture")
            commit = git(root, self.environment, "rev-parse", "HEAD")
            records = read_dependency_tree(root, commit)
            self.assertEqual(len(commit), 40)
            self.assertEqual(records[0].content, content)

    def test_corrupted_commit_cannot_redirect_to_another_valid_tree(self):
        old_tree = git(self.root, self.environment, "rev-parse", f"{self.base}^{{tree}}")
        (self.root / "Cargo.toml").write_text("[package]\nname='changed'\n", encoding="utf-8")
        after = self.commit()
        new_tree = git(self.root, self.environment, "rev-parse", f"{after}^{{tree}}")
        path = self.root / ".git" / "objects" / self.base[:2] / self.base[2:]
        raw = zlib.decompress(path.read_bytes())
        changed = raw.replace(old_tree.encode(), new_tree.encode(), 1)
        self.assertNotEqual(raw, changed)
        self.assertEqual(len(raw), len(changed))
        path.chmod(0o600)
        path.write_bytes(zlib.compress(changed))
        result = subprocess.run(["git", "-C", str(self.root), "rev-parse", f"{self.base}^{{tree}}"],
                                env=self.environment, capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("hash mismatch", result.stderr)
        with self.assertRaises(GitBindingError):
            read_dependency_tree(self.root, self.base)

    def test_corrupted_tree_cannot_hide_dependency_manifest(self):
        oid = git(self.root, self.environment, "rev-parse", f"{self.base}^{{tree}}")
        path = self.root / ".git" / "objects" / oid[:2] / oid[2:]
        raw = zlib.decompress(path.read_bytes())
        changed = raw.replace(b"Cargo.toml", b"Other.toml", 1)
        self.assertNotEqual(raw, changed)
        self.assertEqual(len(raw), len(changed))
        path.chmod(0o600)
        path.write_bytes(zlib.compress(changed))
        with self.assertRaises(GitBindingError):
            read_dependency_tree(self.root, self.base)

    def test_add_modify_remove_and_unmentioned_nested_files_are_discovered(self):
        nested = self.root / "nested space" / "日本語"
        nested.mkdir(parents=True)
        (nested / "Cargo.toml").write_text("[dependencies]\nkatana-x='1'\n", encoding="utf-8")
        old_content = (self.root / "Cargo.lock").read_text(encoding="utf-8")
        (self.root / "Cargo.lock").write_text(old_content + "\n# changed\n", encoding="utf-8")
        (self.root / "Cargo.toml").unlink()
        head = self.commit()
        changes = {item.path: item for item in dependency_tree_changes(self.root, self.base, head)}
        self.assertIsNone(changes["Cargo.toml"].after)
        self.assertIsNone(changes["nested space/日本語/Cargo.toml"].before)
        self.assertEqual(changes["Cargo.lock"].after.content, (old_content + "\n# changed\n").encode())

    def test_regular_source_change_does_not_invent_dependency_changes(self):
        (self.root / "source.rs").write_text("fn main() {}", encoding="utf-8")
        head = self.commit()
        self.assertEqual(dependency_tree_changes(self.root, self.base, head), ())

    def test_symlink_dependency_input_is_rejected(self):
        (self.root / "external").mkdir()
        (self.root / "external" / "Cargo.toml").symlink_to("../Cargo.toml")
        with self.assertRaises(GitBindingError):
            read_dependency_tree(self.root, self.commit())

    def test_missing_and_abbreviated_commit_are_rejected(self):
        for commit in (self.base[:40], "f" * 64, "--help"):
            with self.subTest(commit=commit), self.assertRaises(GitBindingError):
                read_dependency_tree(self.root, commit)

    def test_oversized_actual_blob_is_rejected(self):
        (self.root / "Cargo.toml").write_bytes(b"x" * (MAX_DEPENDENCY_BLOB_BYTES + 1))
        with self.assertRaises(GitBindingError):
            read_dependency_tree(self.root, self.commit())

    def test_actual_tree_file_count_limit_is_enforced(self):
        for index in range(MAX_DEPENDENCY_FILES - 1):
            directory = self.root / f"member-{index}"
            directory.mkdir()
            (directory / "Cargo.toml").write_bytes(b"")
        with self.assertRaisesRegex(GitBindingError, "file count"):
            read_dependency_tree(self.root, self.commit())

    def test_actual_tree_aggregate_content_limit_is_enforced(self):
        content = b"x" * MAX_DEPENDENCY_BLOB_BYTES
        for index in range(MAX_DEPENDENCY_TREE_BYTES // len(content) + 1):
            directory = self.root / f"member-{index}"
            directory.mkdir()
            (directory / "Cargo.toml").write_bytes(content)
        with self.assertRaisesRegex(GitBindingError, "aggregate limit"):
            read_dependency_tree(self.root, self.commit())

    def object(self, kind, content, *, literal=False):
        arguments = ["git", "-C", str(self.root), "hash-object", "-w", "-t", kind, "--stdin"]
        if literal:
            arguments.append("--literally")
        return subprocess.run(arguments, env=self.environment, input=content,
                              capture_output=True, check=True).stdout.decode().strip()

    def test_regular_mode_cannot_hide_tag_peeling_to_blob(self):
        blob = self.object("blob", b"x" * 200)
        prefix = f"object {blob}\ntype blob\ntag fixture\ntagger Test <test@example.invalid> 0 +0000\n\n".encode()
        tag = self.object("tag", prefix + b"x" * (200 - len(prefix)))
        tree = self.object("tree", b"100644 Cargo.toml\0" + bytes.fromhex(tag), literal=True)
        commit = git(self.root, self.environment, "commit-tree", tree, "-m", "invalid tree fixture")
        with self.assertRaises(GitBindingError):
            read_dependency_tree(self.root, commit)

    def test_duplicate_dependency_tree_path_is_rejected(self):
        first = self.object("blob", b"[dependencies]\nkatana-x='1'\n")
        last = self.object("blob", b"[dependencies]\nkatana-x='2'\n")
        payload = b"100644 Cargo.toml\0" + bytes.fromhex(first)
        payload += b"100644 Cargo.toml\0" + bytes.fromhex(last)
        tree = self.object("tree", payload, literal=True)
        commit = git(self.root, self.environment, "commit-tree", tree, "-m", "duplicate path fixture")
        with self.assertRaises(GitBindingError):
            read_dependency_tree(self.root, commit)

    def test_root_tree_has_all_dependency_inputs_and_unchanged_has_none(self):
        changes = dependency_tree_changes(self.root, None, self.base)
        self.assertEqual({item.path for item in changes}, {"Cargo.toml", "Cargo.lock"})
        self.assertTrue(all(item.before is None for item in changes))
        self.assertEqual(dependency_tree_changes(self.root, self.base, self.base), ())


if __name__ == "__main__":
    unittest.main()
