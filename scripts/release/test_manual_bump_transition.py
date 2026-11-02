"""Manual bump provenance is checked against real Git, not commit messages."""
from pathlib import Path
import subprocess
import os
import sys
import hashlib
import base64
import io
import json
import zipfile
import tempfile
import unittest

from manual_bump_transition import verify_transition
from manual_bump_graphql import build_request, parse_response
from postpublication_run_admission import bind_generated_transition


def git(root, *args):
    return subprocess.run(["git", *args], cwd=root, check=True, text=True,
                          capture_output=True).stdout.strip()


class ManualBumpTransitionTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        git(self.root, "init", "-q")
        git(self.root, "config", "user.name", "Test")
        git(self.root, "config", "user.email", "test@example.invalid")
        self.write("Cargo.toml", '[workspace]\nmembers = ["crates/app"]\n[workspace.package]\nversion = "0.1.0"\n')
        self.write("crates/app/Cargo.toml", '[package]\nname = "example-app"\nversion.workspace = true\n')
        self.write("Cargo.lock", 'version = 4\n\n[[package]]\nname = "example-app"\nversion = "0.1.0"\n')
        self.write("crates/katana-ui/Info.plist", '<plist><key>CFBundleShortVersionString</key><string>v0.1.0</string></plist>\n')
        for path in (".github/workflows/build-and-release.yml", "scripts/release/bump-version.sh",
                     "scripts/release/manual_bump_transition.py", "scripts/release/manual_bump_graphql.py"):
            self.write(path, "trusted producer\n")
        self.commit("base")
        self.base = git(self.root, "rev-parse", "HEAD")
        for path in ("Cargo.toml", "Cargo.lock", "crates/katana-ui/Info.plist"):
            p = self.root / path
            p.write_text(p.read_text().replace("0.1.0", "0.1.1"))
            self.commit(path)
        self.source = git(self.root, "rev-parse", "HEAD")
        self.record = {"schema": "katana-manual-bump-transition-v1", "repository": "HiroyukiFuruno/KatanA",
                       "workflow": ".github/workflows/build-and-release.yml", "run_id": 123, "run_attempt": 2,
                       "event": "workflow_dispatch", "branch": "master", "trigger": self.base,
                       "source": self.source, "version": "v0.1.1"}

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, path, text):
        p = self.root / path
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)

    def commit(self, message):
        git(self.root, "add", ".")
        git(self.root, "commit", "-qm", message)

    def test_real_version_chain_is_accepted(self):
        verify_transition(self.root, self.record, self.base, self.source, "v0.1.1", 123, 2)

    def test_single_atomic_version_commit_is_accepted(self):
        git(self.root, "switch", "--detach", self.base)
        for path in ("Cargo.toml", "Cargo.lock", "crates/katana-ui/Info.plist"):
            p = self.root / path
            p.write_text(p.read_text().replace("0.1.0", "0.1.1"))
        self.commit("atomic bump")
        source = git(self.root, "rev-parse", "HEAD")
        self.assertEqual(len(verify_transition(self.root, self.record | {"source": source},
                                               self.base, source, "v0.1.1", 123, 2)), 1)

    def test_atomic_request_and_head_conflict_are_fail_closed(self):
        request = build_request(self.root, self.base, "v0.1.1")
        mutation = request["variables"]["input"]
        self.assertEqual(mutation["expectedHeadOid"], self.base)
        self.assertEqual({item["path"] for item in mutation["fileChanges"]["additions"]},
                         {"Cargo.toml", "Cargo.lock", "crates/katana-ui/Info.plist"})
        with self.assertRaises(ValueError):
            build_request(self.root, self.base, 'v0.1.1\nmalicious')
        self.assertEqual(parse_response({"data": {"createCommitOnBranch": {"commit": {"oid": self.source}}}}), self.source)
        for response in ({"errors": [{"message": "conflict"}]}, {"data": None}, {"data": {}}):
            with self.assertRaises(ValueError):
                parse_response(response)
        git(self.root, "update-ref", "refs/heads/cas-test", self.base)
        result = subprocess.run(["git", "update-ref", "refs/heads/cas-test", self.source, "0" * 40],
                                cwd=self.root, capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(git(self.root, "rev-parse", "refs/heads/cas-test"), self.base)

    def test_real_recorder_process_writes_the_validated_transition(self):
        script = Path(__file__).with_name("manual_bump_transition.py")
        output, result = self.root / "transition.json", self.root / "output.txt"
        env = os.environ | {"GITHUB_SHA": self.base, "GITHUB_RUN_ID": "123", "GITHUB_RUN_ATTEMPT": "2",
                            "GITHUB_REPOSITORY": "HiroyukiFuruno/KatanA", "GITHUB_EVENT_NAME": "workflow_dispatch",
                            "GITHUB_REF": "refs/heads/master", "GITHUB_OUTPUT": str(result)}
        subprocess.run([sys.executable, "-B", str(script), "--version", "v0.1.1", "--record", str(output)],
                       cwd=self.root, env=env, check=True, timeout=30)
        self.assertEqual(json.loads(output.read_text()), self.record)
        self.assertEqual(result.read_text(), "present=true\n")

    def test_different_identity_or_non_bump_is_rejected(self):
        for key, value in (("repository", "other/repo"), ("run_id", 999), ("run_attempt", 1),
                           ("trigger", self.source), ("source", self.base), ("version", "v0.1.2")):
            with self.subTest(key=key), self.assertRaises(ValueError):
                verify_transition(self.root, self.record | {key: value}, self.base, self.source, "v0.1.1", 123, 2)
        with self.assertRaises(ValueError):
            verify_transition(self.root, self.record, self.base, self.base, "v0.1.1", 123, 2)

    def test_run_artifact_and_real_git_chain_are_bound_together(self):
        run = {"id": 123, "run_attempt": 2, "head_sha": self.base, "event": "workflow_dispatch",
               "head_branch": "master", "path": ".github/workflows/build-and-release.yml",
               "status": "completed", "conclusion": "success",
               "repository": {"full_name": "HiroyukiFuruno/KatanA"},
               "head_repository": {"full_name": "HiroyukiFuruno/KatanA"}}
        buf = io.BytesIO()
        with zipfile.ZipFile(buf, "w") as archive:
            archive.writestr("transition.json", json.dumps(self.record))
        data = buf.getvalue()
        artifact = {"id": 456, "name": "manual-bump-transition-2", "expired": False,
                    "digest": "sha256:" + hashlib.sha256(data).hexdigest(),
                    "workflow_run": {"id": 123, "head_sha": self.base}}
        binding, evidence = bind_generated_transition(self.root, run, artifact, data, self.base, self.source, "v0.1.1")
        self.assertEqual(binding.kind, "generated_manual_bump")
        self.assertEqual(evidence["transition"], self.record)
        self.assertEqual(json.loads(base64.b64decode(evidence["transition_raw_base64"])), self.record)
        for bad_run in (run | {"repository": {"full_name": "other/repo"}}, run | {"run_attempt": 3},
                        run | {"head_sha": self.source}, run | {"head_branch": "other"}):
            with self.assertRaises(ValueError):
                bind_generated_transition(self.root, bad_run, artifact, data, self.base, self.source, "v0.1.1")

    def test_extra_or_modified_content_is_rejected(self):
        self.write("unexpected.txt", "not version-only\n")
        self.commit("forged")
        source = git(self.root, "rev-parse", "HEAD")
        with self.assertRaises(ValueError):
            verify_transition(self.root, self.record | {"source": source}, self.base, source, "v0.1.1", 123, 2)

    def test_version_commit_cannot_hide_other_manifest_changes(self):
        git(self.root, "switch", "--detach", self.base)
        for path in ("Cargo.toml", "Cargo.lock", "crates/katana-ui/Info.plist"):
            p = self.root / path
            updated = p.read_text().replace("0.1.0", "0.1.1")
            if path == "Cargo.toml":
                updated += '\n[profile.release]\nopt-level = 0\n'
            p.write_text(updated)
            self.commit(path)
        source = git(self.root, "rev-parse", "HEAD")
        with self.assertRaises(ValueError):
            verify_transition(self.root, self.record | {"source": source}, self.base, source, "v0.1.1", 123, 2)


if __name__ == "__main__":
    unittest.main()
