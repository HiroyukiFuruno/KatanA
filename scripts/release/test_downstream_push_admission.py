"""Real Git input and local-log contracts; no mocked publication success."""

import hashlib
import json
import os
from pathlib import Path
import tempfile
import unittest

from dependency_push_snapshot import DependencyPushSnapshot
from downstream_issue_evidence import MARKER
from downstream_push_admission import DownstreamPushAdmission, _lock_coverage, _log_digest, _manifest_coverage, _records, _source_coverage
from push_ref_updates import PushRefUpdate
from test_downstream_git_object_format import create_record, git

REPOSITORY = "HiroyukiFuruno/KatanA"
CHECKSUM = "4d39820039dee936b5ae1b7d9656751223421b5140c2b25f91d15ebe7519d635"


class DownstreamAdmissionTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        create_record(self.root, self.environment)
        self.write(".gitignore", "tmp/\n")
        self.write("Cargo.toml", '[package]\nname="fixture"\nversion="1.0.0"\n')
        self.write("Cargo.lock", 'version=4\npackage=[]\n')
        self.base = self.commit()
        self.write("Cargo.toml", '[package]\nname="fixture"\nversion="1.0.0"\n[dependencies]\nkatana-document-viewer="=0.5.12"\n')
        self.write("Cargo.lock", 'version=4\n[[package]]\nname="katana-document-viewer"\nversion="0.5.12"\n'
                   'source="registry+https://github.com/rust-lang/crates.io-index"\nchecksum="' + CHECKSUM + '"\n')
        self.head = self.commit()
        self.snapshot = DependencyPushSnapshot(self.root, self.head)
        self.write("tmp/verification.log", "local fixture verification\n")

    def write(self, path, content):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(content, encoding="utf-8")

    def commit(self):
        git(self.root, self.environment, "add", ".")
        git(self.root, self.environment, "commit", "-qm", "input contract (#335)")
        return git(self.root, self.environment, "rev-parse", "HEAD")

    def change(self):
        files = self.snapshot.context.files
        return {"commit": self.head, "migration_note": "Fixture covers registry and log binding.",
                "upstream": [{"repository": "HiroyukiFuruno/katana-document-viewer", "package": "katana-document-viewer",
                              "version": "0.5.12", "registry_checksum": CHECKSUM,
                              "release_url": "https://github.com/HiroyukiFuruno/katana-document-viewer/releases/tag/v0.5.12"}],
                "manifests": [{"path": "Cargo.toml", "blob": files["Cargo.toml"].blob}],
                "lockfiles": [{"path": "Cargo.lock", "blob": files["Cargo.lock"].blob}],
                "verification": [{"command": "existing pre-push", "exit_code": 0, "source_commit": self.head,
                                  "log_path": "tmp/verification.log",
                                  "log_sha256": hashlib.sha256(b"local fixture verification\n").hexdigest()}]}

    def body(self, change):
        document = {"schema_version": 1, "kind": "downstream", "repository": REPOSITORY, "changes": [change]}
        return MARKER + "\n```json\n" + json.dumps(document) + "\n```\n"

    def admission(self, body, verified_source=None):
        update = PushRefUpdate("HEAD", self.head, "refs/heads/release/test", self.base)
        return DownstreamPushAdmission(self.root, REPOSITORY, update, self.base, [{"body": body}], verified_source)

    def test_missing_downstream_body_is_rejected_on_actual_dependency_change(self):
        with self.assertRaisesRegex(ValueError, "no unique Issue evidence"):
            self.admission("Issue exists but has no dependency evidence", self.head).verify()

    def test_author_exit_zero_does_not_replace_fresh_existing_hook_success(self):
        with self.assertRaisesRegex(ValueError, "fresh existing-hook success"):
            self.admission(self.body(self.change())).verify()

    def test_bound_local_inputs_and_native_version_match(self):
        change = self.change()
        _source_coverage(change, self.snapshot)
        records = _records(change)
        _lock_coverage(self.snapshot, records)
        _manifest_coverage(self.snapshot, records)
        self.assertEqual(_log_digest(self.root, change["verification"][0]["log_path"]),
                         change["verification"][0]["log_sha256"])

    def test_omitted_nested_committed_manifest_is_not_hidden(self):
        self.write("nested/Cargo.toml", '[package]\nname="katana-nested"\nversion="1.0.0"\n')
        snapshot = DependencyPushSnapshot(self.root, self.commit())
        with self.assertRaisesRegex(ValueError, "all committed dependency inputs"):
            _source_coverage(self.change(), snapshot)

    def test_lock_checksum_and_published_version_mismatches_are_rejected(self):
        for key, value in (("registry_checksum", "0" * 64), ("version", "0.5.11")):
            change = self.change()
            change["upstream"][0][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                _lock_coverage(self.snapshot, _records(change))

    def test_verification_log_tamper_is_rejected_before_publication(self):
        body = self.body(self.change())
        self.write("tmp/verification.log", "changed log\n")
        with self.assertRaisesRegex(ValueError, "log checksum"):
            self.admission(body, self.head).verify()

    def test_foreign_and_nonregular_log_paths_are_rejected(self):
        with tempfile.TemporaryDirectory() as foreign:
            path = Path(foreign) / "log"
            path.write_bytes(b"outside")
            (self.root / "tmp/foreign").symlink_to(path)
            with self.assertRaisesRegex(ValueError, "leaves"):
                _log_digest(self.root, "tmp/foreign")
        with self.assertRaisesRegex(ValueError, "regular file"):
            _log_digest(self.root, "tmp")


if __name__ == "__main__":
    unittest.main()
