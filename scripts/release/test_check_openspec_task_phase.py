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


def task_text(incomplete: set[str]) -> str:
    ids = ("1.2", "4.2", "6.4", "6.5", "7.2", "8.1", "8.2", "8.3", "8.4", "8.5", "8.6")
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


if __name__ == "__main__":
    unittest.main()
