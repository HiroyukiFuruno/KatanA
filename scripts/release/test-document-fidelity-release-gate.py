#!/usr/bin/env python3
"""Regression tests for the v0.22.42 document-fidelity release binding."""

from __future__ import annotations

import contextlib
import io
import importlib.util
import json
import os
import subprocess
import tempfile
import unittest
from unittest import mock
from pathlib import Path

SCRIPT = Path(__file__).with_name("check-document-fidelity-release-gate.py")
HELPER_SPEC = importlib.util.spec_from_file_location("document_fidelity_test_artifacts", SCRIPT.with_name("document_fidelity_test_artifacts.py"))
assert HELPER_SPEC is not None and HELPER_SPEC.loader is not None
HELPER = importlib.util.module_from_spec(HELPER_SPEC)
HELPER_SPEC.loader.exec_module(HELPER)
bind_render_artifacts = HELPER.bind_render_artifacts
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
    def test_git_fixture_init_isolated_from_inherited_git_environment(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            clean_environment = {
                key: value for key, value in os.environ.items() if not key.startswith("GIT_")
            }
            for index, inherited_vars in enumerate((
                ("GIT_DIR",),
                ("GIT_DIR", "GIT_WORK_TREE", "GIT_COMMON_DIR"),
            )):
                with self.subTest(inherited=inherited_vars):
                    caller = root / f"caller-{index}"
                    caller.mkdir()
                    metadata = root / f"caller-metadata-{index}"
                    subprocess.run(
                        ["git", "init", "--separate-git-dir", str(metadata), "-q", str(caller)],
                        check=True,
                        env=clean_environment,
                    )
                    inherited = {"GIT_DIR": str(metadata)}
                    if len(inherited_vars) > 1:
                        inherited.update(
                            {
                                "GIT_WORK_TREE": str(caller),
                                "GIT_COMMON_DIR": str(metadata),
                            }
                        )
                    config = metadata / "config"
                    before_config = config.read_bytes()
                    before_status = subprocess.run(
                        [
                            "git",
                            "--git-dir",
                            str(metadata),
                            "--work-tree",
                            str(caller),
                            "status",
                            "--porcelain=v1",
                        ],
                        check=True,
                        capture_output=True,
                        env=clean_environment,
                    ).stdout
                    error = None
                    try:
                        with mock.patch.dict(os.environ, {**clean_environment, **inherited}, clear=True):
                            with self.repository(
                                self.completed_required_tasks(), with_evidence=True
                            ) as fixture_dir:
                                fixture = Path(fixture_dir)
                                self.assertTrue((fixture / ".git").is_dir())
                    except Exception as caught:
                        error = caught
                    after_status = subprocess.run(
                        [
                            "git",
                            "--git-dir",
                            str(metadata),
                            "--work-tree",
                            str(caller),
                            "status",
                            "--porcelain=v1",
                        ],
                        check=True,
                        capture_output=True,
                        env=clean_environment,
                    ).stdout
                    self.assertEqual(config.read_bytes(), before_config)
                    self.assertEqual(after_status, before_status)
                    if error is not None:
                        raise error

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
        subprocess.run(
            ["git", "-C", str(root), "init", "-q"],
            check=True,
            env=EVIDENCE.git_environment(),
        )
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
                "close_ms": 100,
                "comparison": {
                    "contract": "scripts/release/document-fidelity-contracts/html-v0.22.42.json",
                    "contract_sha256": "",
                    "reference_sha256": "c" * 64,
                    "measured_sha256": "d" * 64,
                    "input_sha256": EVIDENCE.ORIGINAL_HTML_SHA256,
                    "producer_mode": "packaged_main",
                    "viewports": {"reference": {"width": 1280, "height": 900}, "measured": {"width": 1280, "height": 900}},
                    "input": {"fixture_sha256": EVIDENCE.ORIGINAL_HTML_SHA256, "anchor": "#s15"},
                    "navigation": {"from_fragment": "", "to_fragment": "#s15", "frame_before": 1, "frame_after": 2},
                    "geometry": {"sticky_toc": {
                        "reference": {"x": 0, "y": 0, "width": 240, "height": 900},
                        "measured": {"x": 0, "y": 0, "width": 240, "height": 900},
                    }, "main": {
                        "reference": {"x": 240, "y": 0, "width": 1040, "height": 900},
                        "measured": {"x": 240, "y": 0, "width": 1040, "height": 900},
                    }, "visible_section": {
                        "reference": {"x": 240, "y": 0, "width": 1040, "height": 900},
                        "measured": {"x": 240, "y": 0, "width": 1040, "height": 900},
                    }},
                    "active_toc": {"reference": "#s15", "measured": "#s15"},
                    "visible_section_state": {"reference": "#s15", "measured": "#s15"},
                },
            },
            "packaged_targets": {
                target: {
                    "status": "passed",
                    "runner_mode": "packaged_main",
                    "clean_machine": True,
                    "normal_close": True,
                    "close_ms": 100,
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
                    "fidelity": {
                        "contract": f"scripts/release/document-fidelity-contracts/office-v0.22.42-{index}.json",
                        "run_id": f"synthetic-office-run-{index}",
                        "reference_sha256": "c" * 64,
                        "measured_sha256": "d" * 64,
                        "input_sha256": input_sha,
                        "contract_sha256": "",
                        "producer_mode": "packaged_main",
                        "viewports": {"reference": {"width": 1280, "height": 900}, "measured": {"width": 1280, "height": 900}},
                        "missing_elements": {"count": 0},
                        "geometry": {"synthetic-element": {"reference": {"x": 0, "y": 0, "width": 10, "height": 10}, "measured": {"x": 0, "y": 0, "width": 10, "height": 10}}},
                    },
                    "release_worker": True,
                    "packaged_target": "linux-x86_64",
                    "packaged_run": {
                        "run_id": f"synthetic-office-run-{index}",
                        "fixture_sha256": input_sha,
                        "status": "passed",
                        "runner_mode": "packaged_main",
                        "clean_machine": True,
                        "normal_close": True,
                        "close_ms": 100,
                        "pid": 200 + index * 2,
                        "sidecar_pid": 201 + index * 2,
                        "heartbeat_frame_before": 10,
                        "heartbeat_frame_after": 11,
                        "cpu_percent": 50.0,
                        "rss_bytes": 200_000_001,
                        "main_path": "/release/KatanA",
                        "sidecar_path": "/release/kdv-office-worker",
                        "observed_sidecar_path": "/release/kdv-office-worker",
                        "main_sha256": digest,
                        "sidecar_sha256": "b" * 64,
                        "observed_main_sha256": digest,
                        "observed_sidecar_sha256": "b" * 64,
                        "cold_rss_bytes": 200_000_000,
                        "after_close_rss_bytes": 200_000_001,
                    },
                }
                for index, (input_sha, format_name) in enumerate(EVIDENCE.SUPPLIED_OFFICE_FIXTURES.items())
            ],
        }
        evidence_path = (
            evidence_parent or root / EVIDENCE.EVIDENCE_RELATIVE
        ) / "evidence" / "document-acceptance-v0.22.42.json"
        contracts = root / "scripts/release/document-fidelity-contracts"
        contracts.mkdir(parents=True)
        artifacts = root / "evidence-artifacts"
        artifacts.mkdir()
        html_geometry = {"sticky_toc": {"x": 0, "y": 0, "width": 240, "height": 900}, "main": {"x": 240, "y": 0, "width": 1040, "height": 900}, "visible_section": {"x": 240, "y": 0, "width": 1040, "height": 900}}
        for filename in ("html-reference.json", "html-measured.json"):
            (artifacts / filename).write_text(json.dumps({"input_sha256": EVIDENCE.ORIGINAL_HTML_SHA256, "geometry": html_geometry}), encoding="utf-8")
        html_contract = {"input_sha256": EVIDENCE.ORIGINAL_HTML_SHA256, "reference_sha256": EVIDENCE.sha256_bytes((artifacts / "html-reference.json").read_bytes()), "reference_renderer": "chromeHTML", "viewport": {"width": 1280, "height": 900}, "reference_artifact": "evidence-artifacts/html-reference.json", "geometry": {target: {"reference": rect, "tolerance": 1} for target, rect in {"sticky_toc": {"x": 0, "y": 0, "width": 240, "height": 900}, "main": {"x": 240, "y": 0, "width": 1040, "height": 900}, "visible_section": {"x": 240, "y": 0, "width": 1040, "height": 900}}.items()}}
        (contracts / "html-v0.22.42.json").write_text(json.dumps(html_contract), encoding="utf-8")
        evidence["html"]["comparison"]["contract_sha256"] = EVIDENCE.sha256_bytes((contracts / "html-v0.22.42.json").read_bytes())
        evidence["html"]["comparison"]["reference_artifact"] = "evidence-artifacts/html-reference.json"
        evidence["html"]["comparison"]["measured_artifact"] = "evidence-artifacts/html-measured.json"
        evidence["html"]["comparison"]["reference_sha256"] = EVIDENCE.sha256_bytes((artifacts / "html-reference.json").read_bytes())
        evidence["html"]["comparison"]["measured_sha256"] = EVIDENCE.sha256_bytes((artifacts / "html-measured.json").read_bytes())
        for index, record in enumerate(evidence["office_fixtures"]):
            for suffix in ("reference", "measured"):
                (artifacts / f"office-{index}-{suffix}.json").write_text(json.dumps({"input_sha256": record["input_sha256"], "geometry": {"synthetic-element": {"x": 0, "y": 0, "width": 10, "height": 10}}}), encoding="utf-8")
            contract = {"input_sha256": record["input_sha256"], "reference_sha256": EVIDENCE.sha256_bytes((artifacts / f"office-{index}-reference.json").read_bytes()), "reference_renderer": "sourceOffice", "viewport": {"width": 1280, "height": 900}, "reference_artifact": f"evidence-artifacts/office-{index}-reference.json", "missing_elements_tolerance": 0, "geometry": {"synthetic-element": {"reference": {"x": 0, "y": 0, "width": 10, "height": 10}, "tolerance": 1}}}
            contract_path = contracts / f"office-v0.22.42-{index}.json"
            contract_path.write_text(json.dumps(contract), encoding="utf-8")
            record["fidelity"]["contract_sha256"] = EVIDENCE.sha256_bytes(contract_path.read_bytes())
            record["fidelity"]["reference_artifact"] = f"evidence-artifacts/office-{index}-reference.json"
            record["fidelity"]["measured_artifact"] = f"evidence-artifacts/office-{index}-measured.json"
            record["fidelity"]["reference_sha256"] = EVIDENCE.sha256_bytes((artifacts / f"office-{index}-reference.json").read_bytes())
            record["fidelity"]["measured_sha256"] = EVIDENCE.sha256_bytes((artifacts / f"office-{index}-measured.json").read_bytes())
        bind_render_artifacts(root, evidence, EVIDENCE)
        subprocess.run(["git", "-C", str(root), "add", "-f", str(contracts)], check=True, env=EVIDENCE.git_environment())
        evidence["source_tree_sha256"] = EVIDENCE.source_tree_sha256(root)
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
        install = '"$EVIDENCE_PYTHON_ENV/bin/python" -m pip install --disable-pip-version-check --only-binary=:all: -r scripts/release/evidence-requirements.txt'
        self_test = '"$EVIDENCE_PYTHON_ENV/bin/python" scripts/release/test-document-fidelity-release-gate.py'
        gate = '"$EVIDENCE_PYTHON_ENV/bin/python" scripts/release/check-document-fidelity-release-gate.py "$VERSION" "$TASK_GATE_MODE"'
        self.assertIn(install, source)
        self.assertIn(self_test, source)
        self.assertIn(gate, source)
        self.assertLess(source.index(install), source.index(self_test))
        self.assertLess(source.index(self_test), source.index("for CHANGE_DIR in"))
        self.assertLess(source.index(gate), source.index("for CHANGE_DIR in"))


if __name__ == "__main__":
    unittest.main()
