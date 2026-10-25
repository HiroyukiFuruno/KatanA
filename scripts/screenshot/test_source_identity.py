import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("source_identity.py")
MODULE = "scripts.screenshot.test_source_identity"


def sanitized_environment() -> dict[str, str]:
    return {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}


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
        return self.git_at(self.repo, *args)

    def git_at(self, repository: Path, *args: str) -> str:
        result = subprocess.run(
            ["git", "-C", str(repository), *args],
            check=False,
            capture_output=True,
            text=True,
            env=sanitized_environment(),
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        return result.stdout

    def run_guard(
        self, action: str, extra_environment: dict[str, str] | None = None
    ) -> subprocess.CompletedProcess[str]:
        environment = sanitized_environment()
        if extra_environment:
            environment.update(extra_environment)
        return subprocess.run(
            ["python3", str(SCRIPT), action, "--repository", str(self.repo), "--identity", str(self.identity)],
            capture_output=True,
            text=True,
            env=environment,
        )

    def donor_repository(self) -> tuple[Path, dict[str, str]]:
        donor = Path(self.temp.name) / "donor"
        donor.mkdir()
        self.git_at(donor, "init", "-q")
        (donor / "donor.txt").write_text("donor\n", encoding="utf-8")
        self.git_at(donor, "add", ".")
        self.git_at(donor, "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "donor")
        git_dir = donor / ".git"
        return donor, {
            "GIT_DIR": str(git_dir),
            "GIT_WORK_TREE": str(donor),
            "GIT_INDEX_FILE": str(git_dir / "index"),
            "GIT_COMMON_DIR": str(git_dir),
        }

    def donor_snapshot(self, donor: Path) -> tuple[str, ...]:
        return (
            self.git_at(donor, "rev-parse", "HEAD"),
            self.git_at(donor, "config", "--local", "--list"),
            self.git_at(donor, "config", "--local", "--get", "core.bare"),
            self.git_at(donor, "ls-files", "--stage"),
            self.git_at(donor, "ls-tree", "-r", "HEAD"),
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

    def test_guard_uses_requested_repo_when_hook_environment_is_injected(self) -> None:
        donor, git_environment = self.donor_repository()
        donor_before = self.donor_snapshot(donor)
        self.assertEqual(self.run_guard("begin").returncode, 0)
        (self.repo / "source.txt").write_text("dirty requested repository\n", encoding="utf-8")
        result = self.run_guard("verify", extra_environment=git_environment)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertIn("Git status is not clean", result.stderr)
        self.assertEqual(self.donor_snapshot(donor), donor_before)

    def test_complete_suite_under_hook_environment_preserves_donor_repository(self) -> None:
        donor, git_environment = self.donor_repository()
        before = self.donor_snapshot(donor)
        environment = sanitized_environment()
        environment.update(git_environment)
        test_names = sorted(
            test.id().rsplit(".", 1)[-1]
            for test in unittest.defaultTestLoader.loadTestsFromTestCase(type(self))
        )
        nested_names = [name for name in test_names if name != self._testMethodName]
        test_ids = [f"{MODULE}.SourceIdentityTest.{name}" for name in nested_names]
        result = subprocess.run(
            [sys.executable, "-m", "unittest", *test_ids],
            cwd=SCRIPT.parents[2],
            env=environment,
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 0, f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}")
        self.assertIn(f"Ran {len(test_ids)} tests", result.stderr)
        self.assertEqual(self.donor_snapshot(donor), before)


if __name__ == "__main__":
    unittest.main()
