#!/usr/bin/env python3
"""上流延期を未知のIssue、host不備、偽の成功へ拡張する誤判定を防ぐ。"""

import unittest
import importlib.util
import json
from pathlib import Path

from document_fidelity_divergence import HTML_ISSUE, DivergenceError, verify_disposition

SPEC = importlib.util.spec_from_file_location(
    "acceptance", Path(__file__).with_name("check-document-fidelity-acceptance-evidence.py"))
ACCEPTANCE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ACCEPTANCE)
FIXTURE_SPEC = importlib.util.spec_from_file_location(
    "acceptance_fixtures", Path(__file__).with_name("test-document-fidelity-acceptance-evidence.py"))
FIXTURES = importlib.util.module_from_spec(FIXTURE_SPEC)
FIXTURE_SPEC.loader.exec_module(FIXTURES)


class DivergenceTests(unittest.TestCase):
    def receipt(self, fields):
        return {"status": "known_upstream_divergence", "owner": "KRR",
                "issue": HTML_ISSUE, "divergent_fields": fields}

    def test_measured_divergence_remains_unresolved(self):
        verify_disposition(self.receipt(["active_toc"]), {"active_toc"})

    def test_matching_measurement_is_not_a_deferred_failure(self):
        verify_disposition({"status": "matched"}, set())
        with self.assertRaises(DivergenceError):
            verify_disposition(self.receipt(["active_toc"]), set())

    def test_host_deferral_and_unknown_fields_are_rejected(self):
        for field in ("reload", "normal_close", "source_identity", "unknown"):
            with self.subTest(field=field), self.assertRaises(DivergenceError):
                verify_disposition(self.receipt([field]), {field})

    def test_status_owner_and_issue_cannot_be_substituted(self):
        for field, value in (("status", "passed"), ("owner", "KatanA"),
                             ("issue", HTML_ISSUE.replace("106", "107"))):
            receipt = self.receipt(["active_toc"])
            receipt[field] = value
            with self.subTest(field=field), self.assertRaises(DivergenceError):
                verify_disposition(receipt, {"active_toc"})

    def test_geometry_keeps_strict_gate_and_measured_identity(self):
        reference = {"x": 0, "y": 0, "width": 100, "height": 100}
        measured = dict(reference, width=80)
        comparison = {"geometry": {"main": {"reference": reference, "measured": measured}}}
        contract = {"geometry": {"main": {"reference": reference, "tolerance": 0}}}
        with self.assertRaises(ACCEPTANCE.AcceptanceEvidenceError):
            ACCEPTANCE.verify_comparison_geometry(comparison, contract, {"main"}, "HTML",
                                                  {"geometry": {"main": reference}}, {"geometry": {"main": measured}})
        differences = ACCEPTANCE.verify_comparison_geometry(
            comparison, contract, {"main"}, "HTML", {"geometry": {"main": reference}},
            {"geometry": {"main": measured}}, collect_divergence=True)
        self.assertEqual(differences, {"geometry.main"})
        with self.assertRaises(ACCEPTANCE.AcceptanceEvidenceError):
            ACCEPTANCE.verify_comparison_geometry(comparison, contract, {"main"}, "HTML",
                                                  {"geometry": {"main": reference}}, {"geometry": {"main": reference}},
                                                  collect_divergence=True)

    def test_missing_extra_duplicate_and_unmeasured_fields_are_rejected(self):
        for fields in ([], ["active_toc", "active_toc"], ["geometry.main"],
                       ["active_toc", "visible_section_state"], [False]):
            with self.subTest(fields=fields), self.assertRaises(DivergenceError):
                verify_disposition(self.receipt(fields), {"active_toc"})
        for receipt in (None, {"status": "passed"}, dict(self.receipt(["active_toc"]), score=95)):
            with self.subTest(receipt=receipt), self.assertRaises(DivergenceError):
                verify_disposition(receipt, {"active_toc"})

    def host_actions(self):
        action = {"input_received": True, "frame_before": 1, "frame_after": 2,
                  "source_sha256": ACCEPTANCE.ORIGINAL_HTML_SHA256}
        return {"scroll": dict(action), "reload": dict(action),
                "dirty_source": {"before_sha256": "a" * 64, "after_sha256": "a" * 64,
                                 "modified_before": True, "modified_after": True}}

    def bind_host_measurement(self, root, evidence):
        html = evidence["html"]
        comparison = html["comparison"]
        path = root / comparison["measured_artifact"]
        measured = json.loads(path.read_text(encoding="utf-8"))
        measured["host_actions"] = self.host_actions()
        measured["geometry"]["main"]["width"] -= 20
        comparison["geometry"]["main"]["measured"]["width"] -= 20
        comparison["disposition"] = self.receipt(["geometry.main"])
        path.write_text(json.dumps(measured), encoding="utf-8")
        digest = ACCEPTANCE.sha256_bytes(path.read_bytes())
        comparison["measured_sha256"] = digest
        html["packaged_run"]["render_output"]["metrics_sha256"] = digest

    def test_full_host_scope_keeps_strict_fidelity_failure(self):
        fixture = FIXTURES.AcceptanceEvidenceTests()
        with fixture.repository() as directory:
            root = Path(directory)
            evidence = fixture.valid_evidence(root)
            self.bind_host_measurement(root, evidence)
            fixture.write_evidence(root, evidence)
            ACCEPTANCE.verify(root, scope="katana-host")
            with self.assertRaises(ACCEPTANCE.AcceptanceEvidenceError):
                ACCEPTANCE.verify(root)

    def test_full_host_scope_rejects_missing_packaged_and_normal_close(self):
        fixture = FIXTURES.AcceptanceEvidenceTests()
        for mutate in (lambda e: e["packaged_targets"].pop("linux-x86_64"),
                       lambda e: e["html"].update(normal_close=False),
                       lambda e: e["html"].update(runner_mode="in_process"),
                       lambda e: e["html"].update(close_ms=5001),
                       lambda e: e.update(source_tree_sha256="0" * 64)):
            with fixture.repository() as directory:
                root = Path(directory)
                evidence = fixture.valid_evidence(root)
                self.bind_host_measurement(root, evidence)
                mutate(evidence)
                fixture.write_evidence(root, evidence)
                with self.assertRaises(ACCEPTANCE.AcceptanceEvidenceError):
                    ACCEPTANCE.verify(root, scope="katana-host")

    def test_host_actions_cannot_be_deferred_or_faked_by_a_static_frame(self):
        ACCEPTANCE.verify_html_host_actions(self.host_actions(), ACCEPTANCE.ORIGINAL_HTML_SHA256)
        for mutate in (lambda a: a.pop("scroll"),
                       lambda a: a["reload"].update(input_received=False),
                       lambda a: a["scroll"].update(frame_after=1),
                       lambda a: a["reload"].update(source_sha256="0" * 64),
                       lambda a: a["dirty_source"].update(after_sha256="b" * 64),
                       lambda a: a["dirty_source"].update(modified_after=False)):
            actions = self.host_actions()
            mutate(actions)
            with self.assertRaises(ACCEPTANCE.AcceptanceEvidenceError):
                ACCEPTANCE.verify_html_host_actions(actions, ACCEPTANCE.ORIGINAL_HTML_SHA256)


if __name__ == "__main__":
    unittest.main()
