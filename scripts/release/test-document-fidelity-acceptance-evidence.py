#!/usr/bin/env python3
"""Regression tests for the v0.22.42 acceptance evidence checker."""

from __future__ import annotations

import importlib.util
import json
import math
import os
import subprocess
import tempfile
import unittest
from unittest import mock
from pathlib import Path


SCRIPT = Path(__file__).with_name("check-document-fidelity-acceptance-evidence.py")
SPEC = importlib.util.spec_from_file_location("document_acceptance_evidence", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class AcceptanceEvidenceTests(unittest.TestCase):
    def test_source_paths_use_fixture_git_metadata_not_inherited_caller(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            caller = root / "caller"
            caller.mkdir()
            metadata = root / "caller-metadata"
            clean_environment = {
                key: value for key, value in os.environ.items() if not key.startswith("GIT_")
            }
            subprocess.run(
                ["git", "init", "--separate-git-dir", str(metadata), "-q", str(caller)],
                check=True,
                env=clean_environment,
            )
            caller_source = caller / "crates" / "caller.rs"
            caller_source.parent.mkdir()
            caller_source.write_text("pub fn caller() {}\n", encoding="utf-8")
            subprocess.run(
                ["git", "-C", str(caller), "add", "crates/caller.rs"],
                check=True,
                env=clean_environment,
            )
            config = metadata / "config"
            before_config = config.read_bytes()
            before_status = subprocess.run(
                ["git", "-C", str(caller), "status", "--porcelain=v1"],
                check=True,
                capture_output=True,
                env=clean_environment,
            ).stdout
            inherited = {
                "GIT_DIR": str(metadata),
                "GIT_WORK_TREE": str(caller),
                "GIT_COMMON_DIR": str(metadata),
            }
            with mock.patch.dict(os.environ, {**clean_environment, **inherited}, clear=True):
                with self.repository() as fixture_dir:
                    fixture = Path(fixture_dir)
                    fixture_source = fixture / "crates" / "source.rs"
                    enumerated = MODULE.source_paths(fixture)
            after_status = subprocess.run(
                ["git", "-C", str(caller), "status", "--porcelain=v1"],
                check=True,
                capture_output=True,
                env=clean_environment,
            ).stdout
            self.assertEqual(config.read_bytes(), before_config)
            self.assertEqual(after_status, before_status)
            self.assertIn(fixture_source, enumerated)
            self.assertTrue(all(path.is_relative_to(fixture) for path in enumerated))
            self.assertNotIn(caller_source, enumerated)

    def repository(self) -> tempfile.TemporaryDirectory[str]:
        directory = tempfile.TemporaryDirectory()
        root = Path(directory.name)
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
            env=MODULE.git_environment(),
        )
        contracts = root / "scripts/release/document-fidelity-contracts"
        contracts.mkdir(parents=True)
        (contracts / "html-v0.22.42.json").write_text(json.dumps({
            "input_sha256": MODULE.ORIGINAL_HTML_SHA256,
            "reference_sha256": "c" * 64,
            "reference_renderer": "chromeHTML",
            "viewport": {"width": 1280, "height": 900},
            "geometry": {target: {"reference": rect, "tolerance": 1} for target, rect in {
                "sticky_toc": {"x": 0, "y": 0, "width": 240, "height": 900},
                "main": {"x": 240, "y": 0, "width": 1040, "height": 900},
                "visible_section": {"x": 240, "y": 0, "width": 1040, "height": 900},
            }.items()},
        }), encoding="utf-8")
        for index, input_sha in enumerate(MODULE.SUPPLIED_OFFICE_FIXTURES):
            (contracts / f"office-v0.22.42-{index}.json").write_text(json.dumps({
                "input_sha256": input_sha, "reference_sha256": "c" * 64,
                "reference_renderer": "sourceOffice", "viewport": {"width": 1280, "height": 900},
                "missing_elements_tolerance": 0,
                "geometry": {"synthetic-element": {"reference": {"x": 0, "y": 0, "width": 10, "height": 10}, "tolerance": 1}},
            }), encoding="utf-8")
        subprocess.run(["git", "-C", str(root), "add", "-f", str(contracts)], check=True, env=MODULE.git_environment())
        return directory

    def valid_evidence(self, root: Path) -> dict[str, object]:
        evidence = {
            "schema_version": 1,
            "target": "v0.22.42",
            "runner_mode": "packaged_main",
            "published_registry_graph": True,
            "cargo_lock_sha256": MODULE.sha256_bytes((root / "Cargo.lock").read_bytes()),
            "source_tree_sha256": MODULE.source_tree_sha256(root),
            "published_dependencies": {
                "katana-document-viewer": {"version": "0.5.7", "source": MODULE.CRATES_IO_SOURCE},
                "katana-render-runtime": {"version": "0.4.21", "source": MODULE.CRATES_IO_SOURCE},
                "katana-ui-core": {"version": "0.3.17", "source": MODULE.CRATES_IO_SOURCE},
            },
            "html": {
                "fixture_sha256": MODULE.ORIGINAL_HTML_SHA256,
                "runner_mode": "packaged_main",
                "status": "passed",
                "first_frame_ms": 1000,
                "cpu_percent": 50.0,
                "rss_bytes": 100000,
                "normal_close": True,
                "close_ms": 100,
                "comparison": {
                    "contract": "scripts/release/document-fidelity-contracts/html-v0.22.42.json",
                    "reference_sha256": "c" * 64,
                    "measured_sha256": "d" * 64,
                    "producer_mode": "packaged_main",
                    "viewports": {"reference": {"width": 1280, "height": 900}, "measured": {"width": 1280, "height": 900}},
                    "input": {"fixture_sha256": MODULE.ORIGINAL_HTML_SHA256, "anchor": "#s15"},
                    "navigation": {"from_fragment": "", "to_fragment": "#s15", "frame_before": 1, "frame_after": 2},
                    "geometry": {
                      "sticky_toc": {
                        "reference": {"x": 0, "y": 0, "width": 240, "height": 900},
                        "measured": {"x": 0, "y": 0, "width": 240, "height": 900},
                      },
                      "main": {
                        "reference": {"x": 240, "y": 0, "width": 1040, "height": 900},
                        "measured": {"x": 240, "y": 0, "width": 1040, "height": 900},
                      },
                      "visible_section": {
                        "reference": {"x": 240, "y": 0, "width": 1040, "height": 900},
                        "measured": {"x": 240, "y": 0, "width": 1040, "height": 900},
                      },
                    },
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
                    "cpu_percent": 50.0,
                    "rss_bytes": 100000,
                    "main_path": "/release/KatanA",
                    "sidecar_path": "/release/kdv-office-worker",
                    "observed_sidecar_path": "/release/kdv-office-worker",
                    "main_sha256": "a" * 64,
                    "sidecar_sha256": "b" * 64,
                    "observed_main_sha256": "a" * 64,
                    "observed_sidecar_sha256": "b" * 64,
                }
                for target in MODULE.SUPPORTED_TARGETS
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
                        "main_sha256": "a" * 64,
                        "sidecar_sha256": "b" * 64,
                        "observed_main_sha256": "a" * 64,
                        "observed_sidecar_sha256": "b" * 64,
                        "cold_rss_bytes": 200_000_000,
                        "after_close_rss_bytes": 200_000_001,
                    },
                }
                for index, (input_sha, format_name) in enumerate(MODULE.SUPPLIED_OFFICE_FIXTURES.items())
            ],
        }
        evidence["html"]["comparison"]["contract_sha256"] = MODULE.sha256_bytes((root / "scripts/release/document-fidelity-contracts/html-v0.22.42.json").read_bytes())
        for index, record in enumerate(evidence["office_fixtures"]):
            record["fidelity"]["contract_sha256"] = MODULE.sha256_bytes((root / f"scripts/release/document-fidelity-contracts/office-v0.22.42-{index}.json").read_bytes())
        return evidence

    def write_evidence(self, root: Path, evidence: dict[str, object]) -> None:
        path = root / MODULE.EVIDENCE_RELATIVE
        path.parent.mkdir(parents=True)
        path.write_text(json.dumps(evidence), encoding="utf-8")

    def assert_rejected(self, mutate) -> None:
        with self.repository() as directory:
            root = Path(directory)
            evidence = self.valid_evidence(root)
            mutate(evidence)
            self.write_evidence(root, evidence)
            with self.assertRaises(MODULE.AcceptanceEvidenceError):
                MODULE.verify(root)

    def test_valid_evidence_passes(self) -> None:
        with self.repository() as directory:
            root = Path(directory)
            self.write_evidence(root, self.valid_evidence(root))
            MODULE.verify(root)

    def test_missing_evidence_fails_closed(self) -> None:
        with self.repository() as directory:
            with self.assertRaises(MODULE.AcceptanceEvidenceError):
                MODULE.verify(Path(directory))

    def test_in_process_and_nonfinite_values_fail(self) -> None:
        with self.repository() as directory:
            root = Path(directory)
            evidence = self.valid_evidence(root)
            evidence["runner_mode"] = "in_process"
            evidence["html"]["cpu_percent"] = math.nan  # type: ignore[index]
            self.write_evidence(root, evidence)
            with self.assertRaises(MODULE.AcceptanceEvidenceError):
                MODULE.verify(root)

    def test_nonfinite_metric_fails(self) -> None:
        with self.repository() as directory:
            root = Path(directory)
            evidence = self.valid_evidence(root)
            evidence["html"]["cpu_percent"] = math.nan  # type: ignore[index]
            self.write_evidence(root, evidence)
            with self.assertRaises(MODULE.AcceptanceEvidenceError):
                MODULE.verify(root)

    def test_invalid_hash_type_fails(self) -> None:
        with self.repository() as directory:
            root = Path(directory)
            evidence = self.valid_evidence(root)
            evidence["cargo_lock_sha256"] = math.inf
            self.write_evidence(root, evidence)
            with self.assertRaises(MODULE.AcceptanceEvidenceError):
                MODULE.verify(root)

    def test_stale_lock_hash_fails(self) -> None:
        with self.repository() as directory:
            root = Path(directory)
            evidence = self.valid_evidence(root)
            evidence["cargo_lock_sha256"] = "b" * 64
            self.write_evidence(root, evidence)
            with self.assertRaises(MODULE.AcceptanceEvidenceError):
                MODULE.verify(root)

    def test_unpublished_graph_fails(self) -> None:
        with self.repository() as directory:
            root = Path(directory)
            evidence = self.valid_evidence(root)
            evidence["published_registry_graph"] = False
            self.write_evidence(root, evidence)
            with self.assertRaises(MODULE.AcceptanceEvidenceError):
                MODULE.verify(root)

    def test_invalid_identity_type_fails(self) -> None:
        self.assert_rejected(
            lambda evidence: evidence["packaged_targets"]["linux-x86_64"].update(pid="100")
        )

    def test_sidecar_identity_fields_are_required(self) -> None:
        self.assert_rejected(
            lambda evidence: evidence["packaged_targets"]["linux-x86_64"].pop("sidecar_pid")
        )
        self.assert_rejected(
            lambda evidence: evidence["packaged_targets"]["linux-x86_64"].update(sidecar_pid=0)
        )
        self.assert_rejected(
            lambda evidence: evidence["packaged_targets"]["linux-x86_64"].update(sidecar_pid=True)
        )
        self.assert_rejected(
            lambda evidence: evidence["packaged_targets"]["linux-x86_64"].update(sidecar_pid=100)
        )
        self.assert_rejected(
            lambda evidence: evidence["packaged_targets"]["linux-x86_64"].update(
                observed_sidecar_path="/release/other-worker"
            )
        )

    def test_source_tree_requires_git(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
            with self.assertRaises(MODULE.AcceptanceEvidenceError):
                MODULE.source_paths(root)

    def test_html_measurements_must_be_positive(self) -> None:
        for field in ("first_frame_ms", "rss_bytes"):
            self.assert_rejected(lambda evidence, field=field: evidence["html"].update({field: 0}))

    def test_html_static_pass_does_not_replace_differential_comparison(self) -> None:
        self.assert_rejected(lambda evidence: evidence["html"].pop("comparison"))
        self.assert_rejected(
            lambda evidence: evidence["html"]["comparison"].update(
                input={"fixture_sha256": MODULE.ORIGINAL_HTML_SHA256, "anchor": "#other"}
            )
        )
        self.assert_rejected(
            lambda evidence: evidence["html"]["comparison"]["geometry"]["sticky_toc"]["measured"].update(x=2)
        )
        self.assert_rejected(
            lambda evidence: evidence["html"]["comparison"].update(
                active_toc={"reference": "#s15", "measured": "#s14"}
            )
        )

    def test_office_item_count_does_not_replace_fidelity_comparison(self) -> None:
        self.assert_rejected(lambda evidence: evidence["office_fixtures"][0].pop("fidelity"))
        self.assert_rejected(
            lambda evidence: evidence["office_fixtures"][0]["fidelity"].update(
                missing_elements={"count": 1}
            )
        )
        self.assert_rejected(
            lambda evidence: evidence["office_fixtures"][0]["fidelity"].update(
                geometry={"synthetic-element": {"reference": {"x": 0, "y": 0, "width": 10, "height": 10}, "measured": {"x": 2, "y": 0, "width": 10, "height": 10}}}
            )
        )
        self.assert_rejected(
            lambda evidence: evidence["office_fixtures"][0]["fidelity"].update(
                contract_sha256="a" * 64
            )
        )

    def test_comparison_contract_edges_fail_closed_and_signed_coordinates_are_valid(self) -> None:
        self.assert_rejected(lambda evidence: evidence["html"]["comparison"].update(contract="/tmp/contract.json"))
        self.assert_rejected(lambda evidence: evidence["html"]["comparison"].update(contract="scripts/release/document-fidelity-contracts/../x.json"))
        self.assert_rejected(lambda evidence: evidence["html"]["comparison"].pop("producer_mode"))
        self.assert_rejected(lambda evidence: evidence["html"]["comparison"]["navigation"].update(to_fragment="#other"))
        self.assert_rejected(lambda evidence: evidence["html"]["comparison"]["navigation"].update(frame_after=1))
        self.assert_rejected(lambda evidence: evidence["html"]["comparison"]["navigation"].update(frame_before=True))
        self.assert_rejected(lambda evidence: evidence["html"]["comparison"]["viewports"].update(measured={"width": 1, "height": 1}))
        self.assert_rejected(lambda evidence: evidence["html"]["comparison"].update(active_toc={"reference": None, "measured": None}))
        self.assert_rejected(lambda evidence: evidence["office_fixtures"][0]["fidelity"].update(geometry={}))
        self.assert_rejected(lambda evidence: evidence["office_fixtures"][0]["fidelity"].update(missing_elements={"count": -1}))
        self.assert_rejected(lambda evidence: evidence["office_fixtures"][0]["fidelity"].update(geometry={"synthetic-element": {"reference": {"x": 0, "y": 0, "width": 10, "height": 10}, "measured": {"x": 0, "y": 0, "width": 10, "height": 10}, "delta": 999}}))

        with self.repository() as directory:
            root = Path(directory)
            evidence = self.valid_evidence(root)
            evidence["html"]["comparison"]["geometry"]["sticky_toc"]["measured"]["x"] = -0.5
            self.write_evidence(root, evidence)
            MODULE.verify(root)

    def test_contract_files_are_versioned_bound_and_not_excluded_from_source_hash(self) -> None:
        for mutation in ("changed", "untracked", "symlink"):
            with self.subTest(mutation=mutation), self.repository() as directory:
                root = Path(directory)
                evidence = self.valid_evidence(root)
                contract = root / evidence["html"]["comparison"]["contract"]
                if mutation == "changed":
                    previous_hash = MODULE.source_tree_sha256(root)
                    contract.write_text(contract.read_text() + "\n", encoding="utf-8")
                    self.assertNotEqual(previous_hash, MODULE.source_tree_sha256(root))
                elif mutation == "untracked":
                    subprocess.run(["git", "-C", str(root), "rm", "--cached", "--", str(contract)], check=True, capture_output=True, env=MODULE.git_environment())
                else:
                    target = contract.with_name("replacement.json")
                    contract.rename(target)
                    contract.symlink_to(target.name)
                self.write_evidence(root, evidence)
                with self.assertRaises(MODULE.AcceptanceEvidenceError):
                    MODULE.verify(root)
                with self.assertRaises(MODULE.AcceptanceEvidenceError):
                    MODULE.verify_html_comparison(root, evidence["html"])

    def test_missing_contract_hash_is_not_repaired_and_identical_render_hashes_are_valid(self) -> None:
        self.assert_rejected(lambda evidence: evidence["html"]["comparison"].pop("contract_sha256"))
        with self.repository() as directory:
            root = Path(directory)
            evidence = self.valid_evidence(root)
            comparison = evidence["html"]["comparison"]
            comparison["measured_sha256"] = comparison["reference_sha256"]
            self.write_evidence(root, evidence)
            MODULE.verify(root)

    def test_normal_close_requires_measured_duration_for_every_run_kind(self) -> None:
        selectors = (
            lambda evidence: evidence["html"],
            lambda evidence: evidence["packaged_targets"]["linux-x86_64"],
            lambda evidence: evidence["office_fixtures"][0]["packaged_run"],
        )
        for select in selectors:
            self.assert_rejected(lambda evidence: select(evidence).pop("close_ms"))
            for duration in (5000.1, -1, True, "100", None, math.nan, math.inf):
                with self.subTest(select=select, duration=duration):
                    self.assert_rejected(
                        lambda evidence: select(evidence).update(close_ms=duration)
                    )

    def test_normal_close_accepts_exact_deadline_for_every_run_kind(self) -> None:
        with self.repository() as directory:
            root = Path(directory)
            evidence = self.valid_evidence(root)
            evidence["html"]["close_ms"] = 5000
            for record in evidence["packaged_targets"].values():
                record["close_ms"] = 5000
            for record in evidence["office_fixtures"]:
                record["packaged_run"]["close_ms"] = 5000
            self.write_evidence(root, evidence)
            MODULE.verify(root)

    def test_packaged_target_contract_fields_are_required(self) -> None:
        mutations = (
            lambda evidence: evidence["packaged_targets"]["linux-x86_64"].update(runner_mode="in_process"),
            lambda evidence: evidence["packaged_targets"]["linux-x86_64"].update(clean_machine=False),
            lambda evidence: evidence["packaged_targets"]["linux-x86_64"].update(normal_close=False),
            lambda evidence: evidence["packaged_targets"]["linux-x86_64"].update(heartbeat_frame_after=1),
            lambda evidence: evidence["packaged_targets"]["linux-x86_64"].update(cpu_percent=float("nan")),
            lambda evidence: evidence["packaged_targets"]["linux-x86_64"].update(rss_bytes=0),
            lambda evidence: evidence["packaged_targets"]["linux-x86_64"].update(main_path="KatanA"),
        )
        for mutation in mutations:
            self.assert_rejected(mutation)

    def test_office_formats_and_input_hashes_are_distinct(self) -> None:
        self.assert_rejected(
            lambda evidence: evidence["office_fixtures"][0].update(format="pdf")
        )
        self.assert_rejected(
            lambda evidence: evidence["office_fixtures"][1].update(
                input_sha256=evidence["office_fixtures"][0]["input_sha256"]
            )
        )

    def test_office_requires_six_distinct_results_and_xlsx_pptx(self) -> None:
        self.assert_rejected(lambda evidence: evidence["office_fixtures"].pop())
        self.assert_rejected(
            lambda evidence: [record.update(format="docx") for record in evidence["office_fixtures"]]
        )

    def test_sha_requires_exact_hex(self) -> None:
        self.assert_rejected(
            lambda evidence: evidence.update(cargo_lock_sha256="+" + "a" * 63)
        )

    def test_schema_version_boolean_fails(self) -> None:
        self.assert_rejected(lambda evidence: evidence.update(schema_version=True))

    def test_schema_version_float_and_string_fail(self) -> None:
        self.assert_rejected(lambda evidence: evidence.update(schema_version=1.0))
        self.assert_rejected(lambda evidence: evidence.update(schema_version="1"))

    def test_supplied_input_format_mapping_is_enforced(self) -> None:
        self.assert_rejected(
            lambda evidence: evidence["office_fixtures"][0].update(format="pptx")
        )
        self.assert_rejected(
            lambda evidence: evidence["office_fixtures"][0].update(input_sha256="c" * 64)
        )
        self.assert_rejected(
            lambda evidence: evidence["office_fixtures"].pop()
        )

    def test_unexecuted_zero_measurements_fail(self) -> None:
        self.assert_rejected(
            lambda evidence: evidence["office_fixtures"][0].update(first_frame_ms=0)
        )
        self.assert_rejected(
            lambda evidence: evidence["office_fixtures"][0].update(item_count=0)
        )

    def test_office_requires_packaged_run_instead_of_boolean_worker_claim(self) -> None:
        self.assert_rejected(
            lambda evidence: evidence["office_fixtures"][0].pop("packaged_run")
        )

    def test_office_packaged_run_requires_passed_packaged_main_identity(self) -> None:
        mutations = (
            ("in_process", lambda evidence: evidence["office_fixtures"][0]["packaged_run"].update(runner_mode="in_process")),
            ("normal_close_false", lambda evidence: evidence["office_fixtures"][0]["packaged_run"].update(normal_close=False)),
            ("clean_machine_false", lambda evidence: evidence["office_fixtures"][0]["packaged_run"].update(clean_machine=False)),
            ("heartbeat_stalled", lambda evidence: evidence["office_fixtures"][0]["packaged_run"].update(heartbeat_frame_after=10)),
            ("main_hash_mismatch", lambda evidence: evidence["office_fixtures"][0]["packaged_run"].update(main_sha256="c" * 64)),
            ("different_main_artifact", lambda evidence: evidence["office_fixtures"][0]["packaged_run"].update(main_sha256="c" * 64, observed_main_sha256="c" * 64)),
            ("different_sidecar_artifact", lambda evidence: evidence["office_fixtures"][0]["packaged_run"].update(sidecar_sha256="c" * 64, observed_sidecar_sha256="c" * 64)),
            ("different_main_path", lambda evidence: evidence["office_fixtures"][0]["packaged_run"].update(main_path="/another/KatanA")),
            ("different_sidecar_path", lambda evidence: evidence["office_fixtures"][0]["packaged_run"].update(sidecar_path="/another/kdv-office-worker", observed_sidecar_path="/another/kdv-office-worker")),
            ("fixture_hash_mismatch", lambda evidence: evidence["office_fixtures"][0]["packaged_run"].update(fixture_sha256="c" * 64)),
            ("missing_target", lambda evidence: evidence["office_fixtures"][0].pop("packaged_target")),
            ("unknown_target", lambda evidence: evidence["office_fixtures"][0].update(packaged_target="unknown-target")),
        )
        for case, mutation in mutations:
            with self.subTest(case=case):
                self.assert_rejected(mutation)

    def test_office_packaged_run_ids_are_required_and_unique(self) -> None:
        cases = (
            ("missing", lambda evidence: evidence["office_fixtures"][0]["packaged_run"].pop("run_id")),
            ("empty", lambda evidence: evidence["office_fixtures"][0]["packaged_run"].update(run_id=" ")),
            ("surrounding_whitespace", lambda evidence: evidence["office_fixtures"][0]["packaged_run"].update(run_id=" run-1 ")),
            (
                "duplicate",
                lambda evidence: evidence["office_fixtures"][1]["packaged_run"].update(
                    run_id=evidence["office_fixtures"][0]["packaged_run"]["run_id"]
                ),
            ),
        )
        for case, mutation in cases:
            with self.subTest(case=case):
                self.assert_rejected(mutation)

    def test_office_packaged_run_requires_positive_rss_measurements(self) -> None:
        for field in ("cold_rss_bytes", "after_close_rss_bytes"):
            with self.subTest(field=field, case="missing"):
                self.assert_rejected(
                    lambda evidence, field=field: evidence["office_fixtures"][0]["packaged_run"].pop(field)
                )
            with self.subTest(field=field, case="zero"):
                self.assert_rejected(
                    lambda evidence, field=field: evidence["office_fixtures"][0]["packaged_run"].update({field: 0})
                )

    def test_office_cold_rss_delta_keeps_existing_budget(self) -> None:
        with self.repository() as directory:
            root = Path(directory)
            evidence = self.valid_evidence(root)
            evidence["office_fixtures"][0]["packaged_run"].update(
                cold_rss_bytes=100_000_000,
                after_close_rss_bytes=100_000_000 + 196_608 * 1024,
            )
            self.write_evidence(root, evidence)
            MODULE.verify(root)
        self.assert_rejected(
            lambda evidence: evidence["office_fixtures"][0]["packaged_run"].update(
                cold_rss_bytes=100_000_000,
                after_close_rss_bytes=100_000_000 + 196_608 * 1024 + 1,
            )
        )

    def test_office_first_frame_has_15000_ms_inclusive_limit(self) -> None:
        with self.repository() as directory:
            root = Path(directory)
            evidence = self.valid_evidence(root)
            evidence["office_fixtures"][0]["first_frame_ms"] = 15000
            self.write_evidence(root, evidence)
            MODULE.verify(root)

        self.assert_rejected(
            lambda evidence: evidence["office_fixtures"][0].update(first_frame_ms=15000.001)
        )

    def test_assets_are_in_source_tree_hash(self) -> None:
        with self.repository() as directory:
            root = Path(directory)
            assets = root / "assets"
            assets.mkdir()
            (assets / "fixture.bin").write_bytes(b"before")
            evidence = self.valid_evidence(root)
            (assets / "fixture.bin").write_bytes(b"after")
            self.write_evidence(root, evidence)
            with self.assertRaises(MODULE.AcceptanceEvidenceError):
                MODULE.verify(root)


if __name__ == "__main__":
    unittest.main()
