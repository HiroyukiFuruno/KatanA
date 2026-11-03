#!/usr/bin/env python3
"""Unit regressions for the versioned HTML host QA contract."""

import copy
import unittest

from document_host_html_policy import required_html_operations, validate_html_operations
from document_host_io import HostEvidenceError


def html_operation(name):
    source = {"dirty": False, "preview_input_fnv1a64": "0123456789abcdef",
              "last_imported_disk_fnv1a64": "0123456789abcdef"}
    return {"kind": "html", "name": name,
            "before": {"loaded_source": copy.deepcopy(source)},
            "after": {"loaded_source": copy.deepcopy(source)}}


class HtmlHostPolicyTests(unittest.TestCase):
    def test_public_v47_accepts_clean_scroll_and_reload(self):
        validate_html_operations([html_operation("scroll"), html_operation("reload")],
                                 publication_version="v0.22.47", phase="postpublication")

    def test_public_v47_still_rejects_each_missing_html_operation(self):
        for names in ([], ["scroll"], ["reload"]):
            with self.subTest(names=names), self.assertRaisesRegex(HostEvidenceError, "operations missing"):
                validate_html_operations([html_operation(name) for name in names],
                                         publication_version="v0.22.47", phase="postpublication")

    def test_office_operations_cannot_substitute_for_html(self):
        operations = [{"kind": "office", "name": name} for name in ("scroll", "reload")]
        with self.assertRaises(HostEvidenceError):
            validate_html_operations(operations, publication_version="v0.22.47", phase="postpublication")

    def test_legacy_candidate_and_publication_still_require_dirty_source(self):
        for phase in ("prepublish", "postpublication"):
            with self.subTest(phase=phase), self.assertRaises(HostEvidenceError):
                validate_html_operations([html_operation("scroll"), html_operation("reload")],
                                         publication_version="v0.22.42", phase=phase)
            validate_html_operations([html_operation(name) for name in ("scroll", "reload", "dirty_source")],
                                     publication_version="v0.22.42", phase=phase)

    def test_public_v47_rejects_dirty_or_missing_boundary_state(self):
        for boundary in ("before", "after"):
            for dirty in (True, None, 0, "false"):
                operations = [html_operation("scroll"), html_operation("reload")]
                operations[0][boundary]["loaded_source"]["dirty"] = dirty
                with self.subTest(boundary=boundary, dirty=dirty), self.assertRaises(HostEvidenceError):
                    validate_html_operations(operations, publication_version="v0.22.47", phase="postpublication")

    def test_public_v47_rejects_source_hash_drift_at_either_boundary(self):
        for boundary in ("before", "after"):
            operations = [html_operation("scroll"), html_operation("reload")]
            operations[0][boundary]["loaded_source"]["last_imported_disk_fnv1a64"] = "fedcba9876543210"
            with self.subTest(boundary=boundary), self.assertRaisesRegex(HostEvidenceError, "disk input"):
                validate_html_operations(operations, publication_version="v0.22.47", phase="postpublication")

    def test_public_v47_requires_both_actual_source_boundaries(self):
        for boundary in ("before", "after"):
            for mutation in ("missing_boundary", "missing_source", "missing_hash"):
                operations = [html_operation("scroll"), html_operation("reload")]
                if mutation == "missing_boundary":
                    del operations[0][boundary]
                elif mutation == "missing_source":
                    del operations[0][boundary]["loaded_source"]
                else:
                    del operations[0][boundary]["loaded_source"]["preview_input_fnv1a64"]
                with self.subTest(boundary=boundary, mutation=mutation), self.assertRaises(HostEvidenceError):
                    validate_html_operations(operations, publication_version="v0.22.47", phase="postpublication")

    def test_non_string_html_operation_names_are_rejected(self):
        for name in (None, {}, [], True):
            operations = [html_operation("scroll"), html_operation("reload")]
            operations[0]["name"] = name
            with self.subTest(name=name), self.assertRaises(HostEvidenceError):
                validate_html_operations(operations, publication_version="v0.22.47", phase="postpublication")

    def test_public_v47_does_not_accept_fabricated_dirty_source_operation(self):
        operations = [html_operation(name) for name in ("scroll", "reload", "dirty_source")]
        with self.assertRaisesRegex(HostEvidenceError, "editing operation"):
            validate_html_operations(operations, publication_version="v0.22.47", phase="postpublication")

    def test_unknown_versions_phases_and_v47_candidate_are_not_waivers(self):
        for version, phase in (("v0.22.48", "postpublication"), ("v0.22.47", "prepublish"),
                               ("v0.22.42", "unknown"), (None, "postpublication"),
                               ("v0.22.47", None)):
            with self.subTest(version=version, phase=phase), self.assertRaises(HostEvidenceError):
                required_html_operations(version, phase)

    def test_malformed_operations_are_rejected(self):
        for operations in (None, {}, [None], ["scroll"]):
            with self.subTest(operations=operations), self.assertRaises(HostEvidenceError):
                validate_html_operations(operations, publication_version="v0.22.47", phase="postpublication")

    def test_validation_does_not_mutate_existing_evidence(self):
        operations = [html_operation("scroll"), html_operation("reload")]
        before = copy.deepcopy(operations)
        validate_html_operations(operations, publication_version="v0.22.47", phase="postpublication")
        self.assertEqual(operations, before)


if __name__ == "__main__":
    unittest.main()
