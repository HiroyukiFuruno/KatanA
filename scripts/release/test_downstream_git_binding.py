"""Real Git fixtures for source binding, never command or publication proof."""

import copy
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

from downstream_git_binding import GitBindingError, validate_commit_tree_bindings
from test_downstream_evidence_fields import REPOSITORY, valid_document


class GitBindingTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        self.git("init", "-q", "-b", "master")
        self.git("config", "user.name", "Git Binding Test")
        self.git("config", "user.email", "binding@example.invalid")
        self.git("config", "commit.gpgsign", "false")
        self.git("config", "core.hooksPath", str(self.root / "empty-hooks"))
        (self.root / "Cargo.toml").write_text("[package]\nname='fixture'\n", encoding="utf-8")
        (self.root / "Cargo.lock").write_text("version = 4\n", encoding="utf-8")
        self.commit()
        self.document = self.record()

    def git(self, *arguments, input=None):
        return subprocess.run(["git", "-C", str(self.root), *arguments], env=self.environment,
                              input=input, check=True, capture_output=True, text=True).stdout.strip()

    def commit(self):
        self.git("add", ".")
        self.git("commit", "-qm", "fixture source")

    def record(self):
        document = valid_document()
        change = document["changes"][0]
        change["commit"] = self.git("rev-parse", "HEAD")
        change["verification"][0]["source_commit"] = change["commit"]
        for label, path in (("manifests", "Cargo.toml"), ("lockfiles", "Cargo.lock")):
            change[label] = [{"path": path, "blob": self.git("rev-parse", f"HEAD:{path}")}]
        return document

    def validate(self, document=None):
        validate_commit_tree_bindings(self.document if document is None else document,
                                     self.root, REPOSITORY)

    def test_real_commit_tree_and_unchanged_historical_commit_are_accepted(self):
        self.validate()
        (self.root / "Cargo.lock").write_text("version = 3\n", encoding="utf-8")
        self.commit()
        self.validate()

    def test_missing_or_noncommit_object_is_rejected(self):
        for oid in ("f" * 40, self.git("rev-parse", "HEAD^{tree}"), self.git("rev-parse", "HEAD:Cargo.lock")):
            document = copy.deepcopy(self.document)
            document["changes"][0]["commit"] = oid
            document["changes"][0]["verification"][0]["source_commit"] = oid
            with self.subTest(oid=oid), self.assertRaises(GitBindingError):
                self.validate(document)

    def test_absent_path_and_stale_blob_are_rejected(self):
        document = copy.deepcopy(self.document)
        document["changes"][0]["manifests"][0]["path"] = "missing/Cargo.toml"
        with self.assertRaises(GitBindingError):
            self.validate(document)
        (self.root / "Cargo.lock").write_text("version = 3\n", encoding="utf-8")
        self.commit()
        document = self.record()
        document["changes"][0]["lockfiles"] = self.document["changes"][0]["lockfiles"]
        with self.assertRaises(GitBindingError):
            self.validate(document)

    def test_swapped_file_classification_and_symlink_are_rejected(self):
        document = copy.deepcopy(self.document)
        change = document["changes"][0]
        change["manifests"], change["lockfiles"] = change["lockfiles"], change["manifests"]
        with self.assertRaises(GitBindingError):
            self.validate(document)
        (self.root / "linked").mkdir()
        (self.root / "linked" / "Cargo.toml").symlink_to("../Cargo.toml")
        self.commit()
        document = self.record()
        document["changes"][0]["manifests"] = [{"path": "linked/Cargo.toml",
                                              "blob": self.git("rev-parse", "HEAD:linked/Cargo.toml")}]
        with self.assertRaises(GitBindingError):
            self.validate(document)

    def test_pathspec_metacharacters_are_literal(self):
        directory = self.root / "[literal]*"
        directory.mkdir()
        (directory / "Cargo.toml").write_text("literal\n", encoding="utf-8")
        self.commit()
        document = self.record()
        path = "[literal]*/Cargo.toml"
        document["changes"][0]["manifests"] = [{"path": path,
                                              "blob": self.git("rev-parse", f"HEAD:{path}")}]
        self.validate(document)

    def test_replace_refs_do_not_change_identity(self):
        old_commit = self.git("rev-parse", "HEAD")
        (self.root / "Cargo.lock").write_text("version = 3\n", encoding="utf-8")
        self.commit()
        self.git("replace", old_commit, self.git("rev-parse", "HEAD"))
        self.assertNotEqual(self.git("rev-parse", f"{old_commit}:Cargo.lock"),
                            self.document["changes"][0]["lockfiles"][0]["blob"])
        self.validate()

    def test_gitlink_is_not_a_regular_manifest_blob(self):
        oid = self.git("rev-parse", "HEAD")
        self.git("update-index", "--add", "--cacheinfo", f"160000,{oid},submodule/Cargo.toml")
        self.git("commit", "-qm", "gitlink fixture")
        document = self.record()
        document["changes"][0]["manifests"] = [{"path": "submodule/Cargo.toml", "blob": oid}]
        with self.assertRaises(GitBindingError):
            self.validate(document)

    def test_tree_with_missing_blob_is_rejected(self):
        missing = "f" * 40
        with self.assertRaises(subprocess.CalledProcessError):
            self.git("cat-file", "-t", missing)
        lock_blob = self.document["changes"][0]["lockfiles"][0]["blob"]
        tree = self.git("mktree", "--missing", input=(
            f"100644 blob {missing}\tCargo.toml\n100644 blob {lock_blob}\tCargo.lock\n"))
        commit = self.git("commit-tree", tree, input="missing blob fixture\n")
        document = copy.deepcopy(self.document)
        change = document["changes"][0]
        change["commit"] = commit
        change["verification"][0]["source_commit"] = commit
        change["manifests"][0]["blob"] = missing
        with self.assertRaises(GitBindingError):
            self.validate(document)

    def test_unencodable_path_has_a_typed_rejection(self):
        for path in ("bad\ud800/Cargo.toml", "bad\udc80/Cargo.toml"):
            document = copy.deepcopy(self.document)
            document["changes"][0]["manifests"][0]["path"] = path
            with self.subTest(path=repr(path)), self.assertRaises(GitBindingError):
                self.validate(document)

    def test_partial_clone_does_not_lazily_fetch_missing_blob(self):
        origin = self.root / "origin.git"
        clone = self.root / "partial"
        self.git("clone", "--bare", "--no-local", str(self.root), str(origin))
        self.git("--git-dir", str(origin), "config", "uploadpack.allowFilter", "true")
        self.git("clone", "--filter=blob:none", "--no-checkout", origin.as_uri(), str(clone))
        origin.rename(self.root / "offline-origin.git")
        with self.assertRaises(GitBindingError) as rejection:
            validate_commit_tree_bindings(self.document, clone, REPOSITORY)
        cause = rejection.exception.__cause__
        self.assertIsInstance(cause, subprocess.CalledProcessError)
        self.assertIn(b"cat-file", b" ".join(argument.encode() for argument in cause.cmd))
        self.assertNotIn(b"could not fetch", cause.stderr)
        self.assertNotIn(str(origin).encode(), cause.stderr)

    def test_hostile_git_environment_does_not_mutate_caller(self):
        config = (self.root / ".git" / "config").read_bytes()
        before = dict(os.environ)
        try:
            os.environ.update(GIT_DIR="/nonexistent-binding-git", GIT_WORK_TREE="/nonexistent-binding-tree",
                              GIT_CONFIG_COUNT="1", GIT_CONFIG_KEY_0="core.bare", GIT_CONFIG_VALUE_0="true")
            self.validate()
        finally:
            os.environ.clear()
            os.environ.update(before)
        self.assertEqual((self.root / ".git" / "config").read_bytes(), config)
        self.assertEqual(self.git("rev-parse", "HEAD"), self.document["changes"][0]["commit"])


if __name__ == "__main__":
    unittest.main()
