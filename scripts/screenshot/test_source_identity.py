import json
import subprocess
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("source_identity.py")


class SourceIdentityTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.repo = Path(self.temp.name) / "repo"
        self.repo.mkdir()
        self.git("init", "-q")
        (self.repo / ".gitignore").write_text("tmp/\ntarget/\n", encoding="utf-8")
        (self.repo / "source.txt").write_text("source\n", encoding="utf-8")
        self.git("add", ".")
        self.git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "fixture")
        self.output = self.repo / "tmp" / "capture"
        self.output.mkdir(parents=True)
        self.identity = self.output / "source-identity.json"

    def tearDown(self) -> None:
        self.temp.cleanup()

    def git(self, *args: str) -> str:
        return subprocess.run(
            ["git", "-C", str(self.repo), *args],
            check=True,
            capture_output=True,
            text=True,
        ).stdout

    def run_guard(self, action: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["python3", str(SCRIPT), action, "--repository", str(self.repo), "--identity", str(self.identity)],
            capture_output=True,
            text=True,
        )

    def test_clean_stable_capture_in_ignored_output_succeeds(self) -> None:
        self.assertEqual(self.run_guard("begin").returncode, 0)
        self.assertEqual(self.run_guard("verify").returncode, 0)
        self.assertEqual(json.loads(self.identity.read_text())["head"], self.git("rev-parse", "HEAD").strip())

    def test_external_output_is_allowed(self) -> None:
        self.identity = Path(self.temp.name) / "external" / "capture" / "source-identity.json"
        self.identity.parent.mkdir(parents=True)
        self.assertEqual(self.run_guard("begin").returncode, 0)
        self.assertEqual(self.run_guard("verify").returncode, 0)

    def test_dirty_tracked_staged_and_untracked_fail_closed(self) -> None:
        (self.repo / "source.txt").write_text("changed\n", encoding="utf-8")
        self.assertNotEqual(self.run_guard("begin").returncode, 0)
        (self.repo / "source.txt").write_text("source\n", encoding="utf-8")
        (self.repo / "source.txt").write_text("staged\n", encoding="utf-8")
        self.git("add", "source.txt")
        self.assertNotEqual(self.run_guard("begin").returncode, 0)
        self.git("reset", "-q", "--", "source.txt")
        (self.repo / "source.txt").write_text("source\n", encoding="utf-8")
        (self.repo / "new.txt").write_text("untracked\n", encoding="utf-8")
        self.assertNotEqual(self.run_guard("begin").returncode, 0)

    def test_head_change_after_begin_fails_verify(self) -> None:
        self.assertEqual(self.run_guard("begin").returncode, 0)
        (self.repo / "next.txt").write_text("next\n", encoding="utf-8")
        self.git("add", "next.txt")
        self.git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "next")
        self.assertNotEqual(self.run_guard("verify").returncode, 0)

    def test_tracked_worktree_change_after_begin_fails_verify(self) -> None:
        self.assertEqual(self.run_guard("begin").returncode, 0)
        (self.repo / "source.txt").write_text("changed during capture\n", encoding="utf-8")
        self.assertNotEqual(self.run_guard("verify").returncode, 0)

    def test_staged_index_change_after_begin_fails_verify(self) -> None:
        self.assertEqual(self.run_guard("begin").returncode, 0)
        (self.repo / "source.txt").write_text("staged during capture\n", encoding="utf-8")
        self.git("add", "source.txt")
        self.assertNotEqual(self.run_guard("verify").returncode, 0)

    def test_untracked_file_after_begin_fails_verify(self) -> None:
        self.assertEqual(self.run_guard("begin").returncode, 0)
        (self.repo / "new.txt").write_text("untracked during capture\n", encoding="utf-8")
        self.assertNotEqual(self.run_guard("verify").returncode, 0)

    def test_invalid_identity_json_schema_and_head_fail_verify(self) -> None:
        valid_head = self.git("rev-parse", "HEAD").strip()
        invalid_values = [
            "{invalid",
            json.dumps({"schema_version": 99, "head": valid_head}),
            json.dumps({"schema_version": True, "head": valid_head}),
            json.dumps({"schema_version": 1.0, "head": valid_head}),
            json.dumps({"schema_version": 1, "head": "abc"}),
            json.dumps({"schema_version": 1, "head": ""}),
            json.dumps({"schema_version": 1, "head": "A" * 40}),
            json.dumps({"schema_version": 1, "head": valid_head, "extra": True}),
        ]
        for value in invalid_values:
            with self.subTest(value=value):
                self.identity.write_text(value, encoding="utf-8")
                self.assertNotEqual(self.run_guard("verify").returncode, 0)

    def test_nonignored_repo_output_is_rejected(self) -> None:
        self.identity = self.repo / "output" / "capture" / "source-identity.json"
        self.identity.parent.mkdir(parents=True)
        self.assertNotEqual(self.run_guard("begin").returncode, 0)


if __name__ == "__main__":
    unittest.main()
