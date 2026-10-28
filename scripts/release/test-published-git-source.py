from __future__ import annotations

import copy
import json
import subprocess
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from published_git_source import (  # noqa: E402
    MAX_SYMLINK_DEPTH,
    PublishedGitSourceError,
    git_environment,
    git_source_identity,
    verify_git_provenance,
)

ROOT = Path(__file__).resolve().parents[2]
COMMIT = "8c89a1544e9ac693d5ba388792a843baa55c9363"
EVIDENCE = ROOT / "evidence/v0.22.42-public-download-40672472894-build-linux.json"
REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"
PACKAGES = {
    "katana-document-viewer": ("0.5.12", "4d39820039dee936b5ae1b7d9656751223421b5140c2b25f91d15ebe7519d635"),
    "katana-render-runtime": ("0.4.23", "8e5e0803f343d781dc015d1fe21b052492d67579ba303cf86203f60455852e00"),
    "katana-ui-core": ("0.4.1", "f8fd7856249d96c0ce65e68151a327d82511af70a8fbe7416058a1a0ba8e9d5c"),
}


def run_git(*args: str, input: bytes | None = None, env: dict[str, str] | None = None) -> bytes:
    result = subprocess.run(
        ["git", "-C", str(ROOT), *args],
        input=input,
        check=True,
        capture_output=True,
        env=env or git_environment(),
        timeout=60,
    )
    return result.stdout.strip()


def blob(data: bytes) -> str:
    return run_git("hash-object", "-w", "--stdin", input=data).decode()


def lockfile(registry: str = REGISTRY, *, checksum: str | None = None) -> bytes:
    records = []
    for name, (version, default_checksum) in PACKAGES.items():
        records.append(
            f'[[package]]\nname = "{name}"\nversion = "{version}"\n'
            f'source = "{registry}"\nchecksum = "{checksum or default_checksum}"\n'
        )
    return ("\n".join(records)).encode()


def fixture_commit(symlinks: dict[str, bytes] | None = None, *, files: dict[str, bytes] | None = None, submodule: bool = False) -> str:
    files = {"LICENSE": b"fixture license\n", **(files or {})}
    link_map = symlinks or {}
    scripts_entries = []
    for path, data in files.items():
        if not path.startswith("scripts/"):
            continue
        name = path.removeprefix("scripts/")
        scripts_entries.append(f"100644 blob {blob(data)}\t{name}\n")
    for name, target in link_map.items():
        scripts_entries.append(f"120000 blob {blob(target)}\t{name}\n")
    if submodule:
        # A commit object used with mode 160000 is a tree entry, never a regular file.
        child_tree = run_git("mktree", input=b"")
        child_commit = run_git("commit-tree", child_tree.decode().strip(), env=fixture_git_env())
        scripts_entries.append(f"160000 commit {child_commit.decode().strip()}\tsubmodule\n")
    scripts_tree = run_git("mktree", input="".join(scripts_entries).encode())
    root_entries = [f"100644 blob {blob(files.get('Cargo.lock', lockfile()))}\tCargo.lock\n", f"100644 blob {blob(files['LICENSE'])}\tLICENSE\n", f"040000 tree {scripts_tree.decode().strip()}\tscripts\n"]
    tree = run_git("mktree", input="".join(root_entries).encode())
    return run_git("commit-tree", tree.decode().strip(), env=fixture_git_env()).decode()


def fixture_git_env() -> dict[str, str]:
    env = git_environment()
    env.update({"GIT_AUTHOR_NAME": "fixture", "GIT_AUTHOR_EMAIL": "fixture@example.invalid", "GIT_COMMITTER_NAME": "fixture", "GIT_COMMITTER_EMAIL": "fixture@example.invalid", "GIT_AUTHOR_DATE": "2000-01-01T00:00:00Z", "GIT_COMMITTER_DATE": "2000-01-01T00:00:00Z"})
    return env


class PublishedGitSourceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.provenance = json.loads(EVIDENCE.read_text(encoding="utf-8"))["provenance"]

    def assert_fixture_rejected(self, **kwargs: object) -> None:
        commit = fixture_commit(**kwargs)
        with self.assertRaises(PublishedGitSourceError):
            git_source_identity(ROOT, commit)

    def test_identity_matches_published_producer_manifest(self) -> None:
        actual = git_source_identity(ROOT, COMMIT)
        self.assertEqual(actual, self.provenance)
        self.assertEqual(verify_git_provenance(ROOT, self.provenance, COMMIT), actual)

    def test_rejects_wrong_expected_commit_and_object_expressions(self) -> None:
        tree_oid = run_git("rev-parse", f"{COMMIT}^{{tree}}").decode()
        for value in ("0000000000000000000000000000000000000000", tree_oid, COMMIT + "^{tree}", "HEAD", "refs/tags/v0.22.42"):
            with self.subTest(value=value), self.assertRaises(PublishedGitSourceError):
                verify_git_provenance(ROOT, self.provenance, value)

    def test_rejects_modified_provenance_fields(self) -> None:
        for field in ("source_commit", "source_tree_sha256", "cargo_lock_sha256", "published_dependencies"):
            changed = copy.deepcopy(self.provenance)
            if field == "published_dependencies":
                changed[field]["katana-ui-core"]["checksum"] = "0" * 64
                changed[field]["katana-render-runtime"]["source"] = "git+https://example.invalid/renderer"
            elif field == "source_commit":
                changed[field] = "0" * 40
            else:
                changed[field] = "0" * 64
            with self.subTest(field=field), self.assertRaises(PublishedGitSourceError):
                verify_git_provenance(ROOT, changed, COMMIT)

    def test_rejects_incomplete_or_extra_fields(self) -> None:
        for value in ({}, {**self.provenance, "extra": True}):
            with self.subTest(value=value), self.assertRaises(PublishedGitSourceError):
                verify_git_provenance(ROOT, value, COMMIT)

    def test_resolves_valid_in_tree_link_from_commit_objects(self) -> None:
        commit = fixture_commit({"license-link": b"../LICENSE"})
        identity = git_source_identity(ROOT, commit)
        self.assertEqual(identity["source_commit"], commit)
        self.assertEqual(identity["published_dependencies"], self.provenance["published_dependencies"])
        self.assertEqual(identity["source_tree_sha256"], git_source_identity(ROOT, commit)["source_tree_sha256"])

    def test_rejects_absolute_escaping_missing_and_invalid_utf8_links(self) -> None:
        for target in (b"/etc/passwd", b"../../outside", b"missing", b"\xff"):
            with self.subTest(target=target):
                self.assert_fixture_rejected(symlinks={"link": target})

    def test_rejects_symlink_cycles_and_excessive_depth(self) -> None:
        self.assert_fixture_rejected(symlinks={"a": b"b", "b": b"a"})
        chain = {f"link-{index}": f"link-{index + 1}".encode() for index in range(MAX_SYMLINK_DEPTH)}
        chain[f"link-{MAX_SYMLINK_DEPTH}"] = b"../LICENSE"
        self.assert_fixture_rejected(symlinks=chain)

    def test_rejects_nonregular_symlink_target(self) -> None:
        commit = fixture_commit({"link": b"submodule"}, submodule=True)
        with self.assertRaises(PublishedGitSourceError):
            git_source_identity(ROOT, commit)

    def test_rejects_duplicate_renderer_lock_record(self) -> None:
        duplicated = lockfile() + b"\n" + lockfile().split(b"[[package]]\n", 1)[1].split(b"[[package]]\n", 1)[0]
        commit = fixture_commit(files={"Cargo.lock": duplicated})
        with self.assertRaises(PublishedGitSourceError):
            git_source_identity(ROOT, commit)

    def test_rejects_noncanonical_registry_lockfile(self) -> None:
        bad_lock = lockfile("git+https://example.invalid/renderer")
        commit = fixture_commit(files={"Cargo.lock": bad_lock})
        with self.assertRaises(PublishedGitSourceError):
            git_source_identity(ROOT, commit)


if __name__ == "__main__":
    unittest.main()
