#!/usr/bin/env python3
"""Regression tests for the v0.22.42 document-fidelity release binding."""

from __future__ import annotations

import contextlib
import io
import importlib.util
import json
import subprocess
import tempfile
import unittest
from pathlib import Path

import importlib.util


SCRIPT = Path(__file__).with_name("check-document-fidelity-release-gate.py")
SPEC = importlib.util.spec_from_file_location("document_fidelity_gate", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
EVIDENCE_SCRIPT = SCRIPT.with_name("check-document-fidelity-acceptance-evidence.py")
EVIDENCE_SPEC = importlib.util.spec_from_file_location("document_acceptance_evidence", EVIDENCE_SCRIPT)
assert EVIDENCE_SPEC is not None and EVIDENCE_SPEC.loader is not None
EVIDENCE = importlib.util.module_from_spec(EVIDENCE_SPEC)
EVIDENCE_SPEC.loader.exec_module(EVIDENCE)


class DocumentFidelityReleaseGateTests(unittest.TestCase):
    def completed_required_tasks(self) -> str:
        return "".join(f"- [x] {task_id} complete\n" for task_id in sorted(MODULE.CRITICAL_REQUIRED))

    def repository(
        self,
        tasks: str | None,
        *,
        archive_names: tuple[str, ...] = (),
        with_evidence: bool = False,
    ) -> tempfile.TemporaryDirectory[str]:
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
        if with_evidence:
            evidence_parent = (
                root / "openspec" / "changes" / MODULE.CHANGE_NAME
                if tasks is not None
                else root / "openspec" / "changes" / "archive" / archive_names[0]
            )
            self.write_valid_evidence(root, evidence_parent=evidence_parent)
        return directory

    def write_valid_evidence(self, root: Path, *, evidence_parent: Path | None = None) -> None:
        (root / "Cargo.toml").write_text("[workspace]\nmembers = []\n", encoding="utf-8")
        (root / "crates").mkdir()
        (root / "crates" / "source.rs").write_text("pub fn source() {}\n", encoding="utf-8")
        lock = """[[package]]
name = "katana-document-viewer"
version = "0.5.7"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "katana-render-runtime"
version = "0.4.21"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "katana-ui-core"
version = "0.3.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
"""
        (root / "Cargo.lock").write_text(lock, encoding="utf-8")
        subprocess.run(["git", "-C", str(root), "init", "-q"], check=True)
        digest = "a" * 64
        evidence = {
            "schema_version": 1,
            "target": "v0.22.42",
            "runner_mode": "packaged_main",
            "published_registry_graph": True,
            "cargo_lock_sha256": EVIDENCE.sha256_bytes((root / "Cargo.lock").read_bytes()),
            "source_tree_sha256": EVIDENCE.source_tree_sha256(root),
            "published_dependencies": {
                name: {"version": version, "source": EVIDENCE.CRATES_IO_SOURCE}
                for name, version in (
                    ("katana-document-viewer", "0.5.7"),
                    ("katana-render-runtime", "0.4.21"),
                    ("katana-ui-core", "0.3.17"),
                )
            },
            "html": {
                "fixture_sha256": EVIDENCE.ORIGINAL_HTML_SHA256,
                "runner_mode": "packaged_main",
                "status": "passed",
                "first_frame_ms": 1000,
                "cpu_percent": 50,
                "rss_bytes": 100000,
                "normal_close": True,
            },
            "packaged_targets": {
                target: {
                    "status": "passed",
                    "runner_mode": "packaged_main",
                    "clean_machine": True,
                    "normal_close": True,
                    "pid": 100,
                    "sidecar_pid": 101,
                    "heartbeat_frame_before": 1,
                    "heartbeat_frame_after": 2,
                    "cpu_percent": 50,
                    "rss_bytes": 100000,
                    "main_path": "/release/KatanA",
                    "sidecar_path": "/release/kdv-office-worker",
                    "observed_sidecar_path": "/release/kdv-office-worker",
                    "main_sha256": digest,
                    "sidecar_sha256": "b" * 64,
                    "observed_main_sha256": digest,
                    "observed_sidecar_sha256": "b" * 64,
                }
                for target in EVIDENCE.SUPPORTED_TARGETS
            },
            "office_fixtures": [
                {
                    "format": format_name,
                    "status": "passed",
                    "input_sha256": input_sha,
                    "first_frame_ms": 1000,
                    "item_count": 1,
                    "release_worker": True,
                }
                for input_sha, format_name in EVIDENCE.SUPPLIED_OFFICE_FIXTURES.items()
            ],
        }
        evidence_path = (
            evidence_parent or root / EVIDENCE.EVIDENCE_RELATIVE
        ) / "evidence" / "document-acceptance-v0.22.42.json"
        evidence_path.parent.mkdir(parents=True)
        evidence_path.write_text(json.dumps(evidence), encoding="utf-8")

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

    def test_pending_office_performance_evidence_only_allows_draft(self) -> None:
        tasks = "".join(
            f"- [x] {task_id} complete\n"
            for task_id in sorted(MODULE.CRITICAL_REQUIRED - {"4.23"})
        ) + "- [ ] 4.23 native Office performance evidence\n"
        with self.repository(tasks, with_evidence=True) as directory:
            root = Path(directory)
            result, _ = self.run_gate(root, mode="pr-bootstrap")
            self.assertEqual(result, 0)
            for mode in ("strict", "release-artifact-pending", "post-release-evidence"):
                result, output = self.run_gate(root, mode=mode)
                self.assertNotEqual(result, 0, mode)
                self.assertIn("4.23", output, mode)

    def test_office_performance_task_cannot_be_removed(self) -> None:
        tasks = "".join(
            f"- [x] {task_id} complete\n"
            for task_id in sorted(MODULE.CRITICAL_REQUIRED - {"4.23"})
        )
        with self.repository(tasks, with_evidence=True) as directory:
            for mode in ("pr-bootstrap", "strict", "release-artifact-pending", "post-release-evidence"):
                result, output = self.run_gate(Path(directory), mode=mode)
                self.assertNotEqual(result, 0, mode)
                self.assertIn("critical release task IDs are missing", output, mode)
                self.assertIn("4.23", output, mode)

    def test_completed_tasks_pass(self) -> None:
        for mode in ("strict", "release-artifact-pending", "post-release-evidence"):
            with self.repository(self.completed_required_tasks(), with_evidence=True) as directory:
                result, output = self.run_gate(Path(directory), mode=mode)
            self.assertEqual(result, 0, (mode, output))

    def test_completed_tasks_without_evidence_fail_closed(self) -> None:
        with self.repository(self.completed_required_tasks()) as directory:
            result, output = self.run_gate(Path(directory))
        self.assertNotEqual(result, 0)
        self.assertIn("acceptance evidence", output)

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
        with self.repository(None, archive_names=(archive_name,), with_evidence=True) as directory:
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
        with self.repository(self.completed_required_tasks(), with_evidence=True) as directory:
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
