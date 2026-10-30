from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


PROPOSAL = Path(__file__).with_name("check-openspec-task-phase.py")
REPO = Path(__file__).resolve().parents[2]
EXISTING_CHECKER = REPO / "scripts/release/check-openspec-task-completion.py"
MODES = ("strict", "pr-bootstrap", "release-artifact-pending", "post-release-evidence")
CHANGE_43 = "v0-22-43-document-ui-regressions"
CHANGE_44 = "v0-22-44-document-loading-input-regressions"
VERSION_43 = "0.22.43"
VERSION_44 = "0.22.44"


def task_text(incomplete: set[str]) -> str:
    ids = (
        "1.2", "1.3", "2.2", "2.3", "3.2", "4.2", "5.3", "5.4", "5.5", "5.6",
        "6.4", "6.5", "7.2", "8.1", "8.2", "8.3", "8.4", "8.5", "8.6",
    )
    return "\n".join(f"- [{' ' if task_id in incomplete else 'x'}] {task_id} task" for task_id in ids) + "\n"


class PhaseGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.release = self.root / "scripts/release"
        self.release.mkdir(parents=True)
        shutil.copy2(PROPOSAL, self.release / PROPOSAL.name)
        shutil.copy2(EXISTING_CHECKER, self.release / "check-openspec-task-completion.py")
        self.change = self.root / "openspec/changes/v0-22-43-document-ui-regressions"
        self.change.mkdir(parents=True)
        self.tasks = self.change / "tasks.md"

    def tearDown(self) -> None:
        self.temp.cleanup()

    def check(self, mode: str, incomplete: set[str], *, version: str = "0.22.43", change: str = "v0-22-43-document-ui-regressions") -> subprocess.CompletedProcess[str]:
        self.tasks.write_text(task_text(incomplete), encoding="utf-8")
        return subprocess.run(
            [sys.executable, str(self.release / PROPOSAL.name), version, change, mode, str(self.tasks)],
            capture_output=True,
            text=True,
            check=False,
        )

    def test_strict_requires_all_tasks(self) -> None:
        self.assertEqual(self.check("strict", set()).returncode, 0)
        self.assertNotEqual(self.check("strict", {"8.6"}).returncode, 0)

    def test_pr_bootstrap_only_defers_push_and_later_phases(self) -> None:
        self.assertEqual(self.check("pr-bootstrap", {"8.2", "8.3", "8.4", "8.5", "8.6"}).returncode, 0)
        for pre_phase in ("1.2", "4.2", "6.4", "6.5", "7.2", "8.1"):
            with self.subTest(task=pre_phase):
                result = self.check("pr-bootstrap", {pre_phase, "8.2", "8.3", "8.4", "8.5", "8.6"})
                self.assertNotEqual(result.returncode, 0)

    def test_publication_modes_narrow_allowed_ids(self) -> None:
        self.assertEqual(self.check("release-artifact-pending", {"8.4", "8.5", "8.6"}).returncode, 0)
        self.assertNotEqual(self.check("release-artifact-pending", {"8.3", "8.4", "8.5", "8.6"}).returncode, 0)
        self.assertEqual(self.check("post-release-evidence", {"8.5", "8.6"}).returncode, 0)
        self.assertNotEqual(self.check("post-release-evidence", {"8.4", "8.5", "8.6"}).returncode, 0)

    def test_version_and_change_name_are_exact(self) -> None:
        self.assertNotEqual(self.check("pr-bootstrap", {"8.2"}, version="0.22.42").returncode, 0)
        self.assertNotEqual(self.check("pr-bootstrap", {"8.2"}, change="v0-22-42-other").returncode, 0)

    def test_tasks_path_must_be_the_bound_change_file(self) -> None:
        self.tasks.write_text(task_text(set()), encoding="utf-8")
        elsewhere = self.root / "tmp/v0-22-43-document-ui-regressions/tasks.md"
        elsewhere.parent.mkdir(parents=True)
        elsewhere.write_text(task_text(set()), encoding="utf-8")
        result = subprocess.run(
            [sys.executable, str(self.release / PROPOSAL.name), "0.22.43", "v0-22-43-document-ui-regressions", "strict", str(elsewhere)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertNotEqual(result.returncode, 0)

    def test_unknown_mode_is_rejected(self) -> None:
        result = self.check("unknown", set())
        self.assertNotEqual(result.returncode, 0)

    def test_partial_marker_remains_incomplete(self) -> None:
        self.tasks.write_text("- [/] 1.2 partial\n", encoding="utf-8")
        result = subprocess.run(
            [sys.executable, str(self.release / PROPOSAL.name), "0.22.43", "v0-22-43-document-ui-regressions", "pr-bootstrap", str(self.tasks)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertNotEqual(result.returncode, 0)


class PhaseGate44Test(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.release = self.root / "scripts/release"
        self.release.mkdir(parents=True)
        shutil.copy2(PROPOSAL, self.release / PROPOSAL.name)
        shutil.copy2(EXISTING_CHECKER, self.release / "check-openspec-task-completion.py")
        self.change = self.root / "openspec/changes" / CHANGE_44
        self.change.mkdir(parents=True)
        self.tasks = self.change / "tasks.md"

    def tearDown(self) -> None:
        self.temp.cleanup()

    def check(self, mode: str, incomplete: set[str], *, version: str = VERSION_44, change: str = CHANGE_44) -> subprocess.CompletedProcess[str]:
        self.tasks.write_text(task_text(incomplete), encoding="utf-8")
        return subprocess.run(
            [sys.executable, str(self.release / PROPOSAL.name), version, change, mode, str(self.tasks)],
            capture_output=True,
            text=True,
            check=False,
        )

    def test_strict_only_accepts_empty_incomplete_set(self) -> None:
        self.assertEqual(self.check("strict", set()).returncode, 0)
        for task_id in ("1.3", "5.3", "5.4", "5.5", "5.6", "1.2", "2.2"):
            with self.subTest(task=task_id):
                self.assertNotEqual(self.check("strict", {task_id}).returncode, 0)

    def test_pr_bootstrap_allows_only_declared_phase_tasks(self) -> None:
        allowed = {"1.3", "5.3", "5.4", "5.5", "5.6"}
        self.assertEqual(self.check("pr-bootstrap", allowed).returncode, 0)
        for task_id in ("1.2", "2.2", "2.3", "3.2", "4.2"):
            with self.subTest(task=task_id):
                self.assertNotEqual(self.check("pr-bootstrap", allowed | {task_id}).returncode, 0)
        for task_id in sorted(allowed):
            with self.subTest(allowed_task=task_id):
                self.assertEqual(self.check("pr-bootstrap", {task_id}).returncode, 0)

    def test_release_and_post_release_allowlists_are_exact(self) -> None:
        self.assertEqual(self.check("release-artifact-pending", {"1.3", "5.5", "5.6"}).returncode, 0)
        for task_id in ("5.3", "5.4"):
            with self.subTest(task=task_id):
                self.assertNotEqual(self.check("release-artifact-pending", {"1.3", "5.5", "5.6", task_id}).returncode, 0)
        self.assertEqual(self.check("post-release-evidence", {"5.6"}).returncode, 0)
        self.assertNotEqual(self.check("post-release-evidence", {"5.5", "5.6"}).returncode, 0)

    def test_version_change_path_mode_and_partial_marker_are_exact(self) -> None:
        self.assertNotEqual(self.check("pr-bootstrap", {"1.3"}, version="0.22.45").returncode, 0)
        self.assertNotEqual(self.check("pr-bootstrap", {"1.3"}, change=CHANGE_43).returncode, 0)
        elsewhere = self.root / "tmp" / CHANGE_44 / "tasks.md"
        elsewhere.parent.mkdir(parents=True)
        elsewhere.write_text(task_text(set()), encoding="utf-8")
        result = subprocess.run(
            [sys.executable, str(self.release / PROPOSAL.name), VERSION_44, CHANGE_44, "strict", str(elsewhere)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertNotEqual(self.check("unknown", set()).returncode, 0)
        self.tasks.write_text("- [/] 1.2 partial\n", encoding="utf-8")
        result = subprocess.run(
            [sys.executable, str(self.release / PROPOSAL.name), VERSION_44, CHANGE_44, "pr-bootstrap", str(self.tasks)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertNotEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
