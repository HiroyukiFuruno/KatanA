"""Real SHA-256 repository boundaries for full commit identity."""

import copy
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

from downstream_git_binding import GitBindingError, validate_commit_tree_bindings
from test_downstream_evidence_fields import REPOSITORY, valid_document


def git(root, environment, *arguments):
    return subprocess.run(["git", "-C", str(root), *arguments], env=environment,
                          capture_output=True, check=True, text=True).stdout.strip()


def create_record(root, environment):
    git(root, environment, "init", "-q", "-b", "master", "--object-format=sha256")
    for key, value in (("user.name", "Object Format Test"), ("user.email", "format@example.invalid"),
                       ("commit.gpgsign", "false"), ("core.hooksPath", str(root / "empty-hooks"))):
        git(root, environment, "config", key, value)
    (root / "Cargo.toml").write_text("[package]\nname='fixture'\n", encoding="utf-8")
    (root / "Cargo.lock").write_text("version = 4\n", encoding="utf-8")
    git(root, environment, "add", ".")
    git(root, environment, "commit", "-qm", "SHA256 fixture")
    document = valid_document()
    change = document["changes"][0]
    change["commit"] = git(root, environment, "rev-parse", "HEAD")
    change["verification"][0]["source_commit"] = change["commit"]
    for label, path in (("manifests", "Cargo.toml"), ("lockfiles", "Cargo.lock")):
        change[label] = [{"path": path, "blob": git(root, environment, "rev-parse", f"HEAD:{path}")}]
    return document


class GitObjectFormatTests(unittest.TestCase):
    def test_sha256_full_oid_is_accepted_but_sha1_length_prefix_is_rejected(self):
        environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            document = create_record(root, environment)
            self.assertEqual(len(document["changes"][0]["commit"]), 64)
            validate_commit_tree_bindings(document, root, REPOSITORY)
            short = copy.deepcopy(document)
            change = short["changes"][0]
            change["commit"] = change["commit"][:40]
            change["verification"][0]["source_commit"] = change["commit"]
            self.assertEqual(git(root, environment, "cat-file", "-t", change["commit"]), "commit")
            with self.assertRaises(GitBindingError):
                validate_commit_tree_bindings(short, root, REPOSITORY)


if __name__ == "__main__":
    unittest.main()
