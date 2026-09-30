#!/usr/bin/env python3
"""Regression tests for the v0.22.42 acceptance evidence checker."""

from __future__ import annotations

import importlib.util
import json
import math
import subprocess
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("check-document-fidelity-acceptance-evidence.py")
SPEC = importlib.util.spec_from_file_location("document_acceptance_evidence", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class AcceptanceEvidenceTests(unittest.TestCase):
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
        subprocess.run(["git", "-C", str(root), "init", "-q"], check=True)
        return directory

    def valid_evidence(self, root: Path) -> dict[str, object]:
        return {
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
            },
            "packaged_targets": {
                target: {
                    "status": "passed",
                    "runner_mode": "packaged_main",
                    "clean_machine": True,
                    "normal_close": True,
                    "pid": 100,
                    "heartbeat_frame_before": 1,
                    "heartbeat_frame_after": 2,
                    "cpu_percent": 50.0,
                    "rss_bytes": 100000,
                    "main_path": "/release/KatanA",
                    "sidecar_path": "/release/kdv-office-worker",
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
                    "release_worker": True,
                }
                for input_sha, format_name in MODULE.SUPPLIED_OFFICE_FIXTURES.items()
            ],
        }

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

    def test_source_tree_requires_git(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
            with self.assertRaises(MODULE.AcceptanceEvidenceError):
                MODULE.source_paths(root)

    def test_html_measurements_must_be_positive(self) -> None:
        for field in ("first_frame_ms", "rss_bytes"):
            self.assert_rejected(lambda evidence, field=field: evidence["html"].update({field: 0}))

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
