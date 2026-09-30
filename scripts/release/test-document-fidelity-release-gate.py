#!/usr/bin/env python3
"""Regression tests for the v0.22.42 document-fidelity release binding."""

from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path

import importlib.util


SCRIPT = Path(__file__).with_name("check-document-fidelity-release-gate.py")
SPEC = importlib.util.spec_from_file_location("document_fidelity_gate", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class DocumentFidelityReleaseGateTests(unittest.TestCase):
    def completed_required_tasks(self) -> str:
        return "".join(f"- [x] {task_id} complete\n" for task_id in sorted(MODULE.CRITICAL_REQUIRED))

    def repository(self, tasks: str | None, *, archive_names: tuple[str, ...] = ()) -> tempfile.TemporaryDirectory[str]:
        directory = tempfile.TemporaryDirectory()
        root = Path(directory.name)
        if tasks is not None:
            active = root / "openspec" / "changes" / MODULE.CHANGE_NAME
            active.mkdir(parents=True)
            (active / "tasks.md").write_text(tasks, encoding="utf-8")
        for archive_name in archive_names:
            archived = root / "openspec" / "changes" / "archive" / archive_name
            archived.mkdir(parents=True)
            (archived / "tasks.md").write_text(
                tasks or self.completed_required_tasks(), encoding="utf-8"
            )
        return directory

    def run_gate(self, root: Path, version: str = "0.22.42", mode: str = "strict") -> tuple[int, str]:
        output = io.StringIO()
        with contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
            try:
                result = MODULE.verify(version, mode, root)
            except (MODULE.GateError, OSError) as error:
                print(f"ERROR: document-fidelity release gate: {error}")
                result = 1
        return result, output.getvalue()

    def test_missing_tasks_fails_closed(self) -> None:
        with self.repository(None) as directory:
            result, output = self.run_gate(Path(directory))
        self.assertNotEqual(result, 0)
        self.assertIn("expected exactly one archived", output)

    def test_required_incomplete_tasks_fail_strict_artifact_and_postrelease(self) -> None:
        tasks = self.completed_required_tasks().replace("- [x] 4.4 complete", "- [/] 4.4 acceptance")
        with self.repository(tasks) as directory:
            for mode in ("strict", "release-artifact-pending", "post-release-evidence"):
                result, output = self.run_gate(Path(directory), mode=mode)
                self.assertNotEqual(result, 0, mode)
                self.assertIn("4.4", output, mode)

    def test_bootstrap_allows_only_explicit_pending_evidence(self) -> None:
        tasks = self.completed_required_tasks()
        tasks = tasks.replace("- [x] 1.2 complete", "- [/] 1.2 upstream evidence")
        tasks = tasks.replace("- [x] 4.17 complete", "- [ ] 4.17 runner evidence")
        with self.repository(tasks) as directory:
            result, _ = self.run_gate(Path(directory), mode="pr-bootstrap")
        self.assertEqual(result, 0)

        tasks = tasks + "- [ ] 9.1 future task\n"
        with self.repository(tasks) as directory:
            result, output = self.run_gate(Path(directory), mode="pr-bootstrap")
        self.assertNotEqual(result, 0)
        self.assertIn("9.1", output)

    def test_completed_tasks_pass(self) -> None:
        with self.repository(self.completed_required_tasks()) as directory:
            result, _ = self.run_gate(Path(directory))
        self.assertEqual(result, 0)

    def test_other_versions_are_unchanged(self) -> None:
        with self.repository(None) as directory:
            result, _ = self.run_gate(Path(directory), version="0.22.41", mode="unknown")
        self.assertEqual(result, 0)

    def test_unknown_mode_fails_closed_for_bound_version(self) -> None:
        with self.repository(self.completed_required_tasks()) as directory:
            result, output = self.run_gate(Path(directory), mode="unknown")
        self.assertNotEqual(result, 0)
        self.assertIn("unsupported", output)

    def test_unique_archive_fallback_passes_and_ambiguous_fallback_fails(self) -> None:
        archive_name = f"2026-10-01-{MODULE.CHANGE_NAME}"
        with self.repository(None, archive_names=(archive_name,)) as directory:
            result, _ = self.run_gate(Path(directory))
        self.assertEqual(result, 0)

        with self.repository(
            None,
            archive_names=(archive_name, f"2026-10-02-{MODULE.CHANGE_NAME}"),
        ) as directory:
            result, output = self.run_gate(Path(directory))
        self.assertNotEqual(result, 0)
        self.assertIn("found 2", output)

    def test_v_prefixed_bound_version_uses_the_gate(self) -> None:
        with self.repository(self.completed_required_tasks()) as directory:
            result, _ = self.run_gate(Path(directory), version="v0.22.42")
        self.assertEqual(result, 0)

    def test_active_directory_without_tasks_fails_closed(self) -> None:
        with self.repository(None) as directory:
            active = Path(directory) / "openspec" / "changes" / MODULE.CHANGE_NAME
            active.mkdir(parents=True)
            result, output = self.run_gate(Path(directory))
        self.assertNotEqual(result, 0)
        self.assertIn("active change tasks file is missing", output)

    def test_archived_directory_without_tasks_fails_closed(self) -> None:
        archive_name = f"2026-10-01-{MODULE.CHANGE_NAME}"
        with self.repository(None, archive_names=()) as directory:
            archived = Path(directory) / "openspec" / "changes" / "archive" / archive_name
            archived.mkdir(parents=True)
            result, output = self.run_gate(Path(directory))
        self.assertNotEqual(result, 0)
        self.assertIn("archived change tasks file is missing", output)

    def test_empty_or_truncated_tasks_fail_closed(self) -> None:
        for tasks in ("", "- [x] 1.2 only\n"):
            with self.repository(tasks) as directory:
                result, output = self.run_gate(Path(directory))
            self.assertNotEqual(result, 0)
            self.assertIn("critical release task IDs are missing", output)

    def test_preflight_runs_self_test_and_binding_before_version_glob(self) -> None:
        preflight = SCRIPT.parents[2] / "scripts" / "release" / "preflight.sh"
        source = preflight.read_text(encoding="utf-8")
        self_test = "python3 scripts/release/test-document-fidelity-release-gate.py"
        gate = 'python3 scripts/release/check-document-fidelity-release-gate.py "$VERSION" "$TASK_GATE_MODE"'
        self.assertIn(self_test, source)
        self.assertIn(gate, source)
        self.assertLess(source.index(self_test), source.index("for CHANGE_DIR in"))
        self.assertLess(source.index(gate), source.index("for CHANGE_DIR in"))


if __name__ == "__main__":
    unittest.main()
