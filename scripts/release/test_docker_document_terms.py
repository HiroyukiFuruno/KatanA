"""Candidate pure boundary checks for fresh-only native Terms acceptance."""

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

from docker_ui_document_fixture import fixture_row_position
from docker_ui_document_terms import TermsController, validate_runtime_config


ROOT = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location(
    "document_log_parser", ROOT / "docker-ui-document-log-parser.py"
)
PARSER = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
SPEC.loader.exec_module(PARSER)


class FreshTermsBoundaryTests(unittest.TestCase):
    def test_visible_version_requires_exact_fixed_token(self):
        self.assertTrue(TermsController.visible_version("Version: 0.22.42 English v"))
        self.assertFalse(TermsController.visible_version("Version: 0.22.420"))
        self.assertFalse(TermsController.visible_version("Version: 0.22.41"))

    def test_modal_requires_heading_version_and_both_controls(self):
        modal = "Terms of Use\nVersion: 0.22.42 English v\nAccept Decline"
        self.assertTrue(TermsController.is_terms_modal(modal))
        self.assertFalse(TermsController.is_terms_modal("please accept/decline terms 0.22.42"))
        self.assertFalse(TermsController.is_terms_modal("Terms of Use 0.22.42 Accept Decline"))
        self.assertFalse(TermsController.is_terms_modal("filename-terms.xlsx"))

    def test_runtime_normalization_allows_isolated_defaults_without_acceptance(self):
        settings = {
            "version": "0.2.3", "theme": {}, "workspace": {
                "last_workspace": "/fixture", "open_tabs": [], "active_tab_idx": None,
                "restore_session": True, "open_workspace_in_tabs": True,
            }, "terms_accepted_version": None,
        }
        validate_runtime_config(settings, "/fixture/only.xlsx")
        settings["terms_accepted_version"] = "0.22.42"
        with self.assertRaises(ValueError):
            validate_runtime_config(settings, "/fixture/only.xlsx")

    def test_fixture_row_requires_unique_left_pane_basename(self):
        header = "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext\n"
        row = "5\t1\t1\t1\t1\t1\t70\t112\t126\t20\t99\trepresentative.xlsx\n"
        self.assertEqual(fixture_row_position(header + row, "representative.xlsx", 1279), (133, 122))
        with self.assertRaises(AssertionError):
            fixture_row_position(header + row, "other.xlsx", 1279)

    def test_accept_button_requires_exactly_one_native_label(self):
        header = "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext\n"
        row = "5\t1\t1\t1\t1\t1\t10\t20\t80\t30\t99\tAccept\n"
        self.assertEqual(TermsController.accept_button_position(header + row), (50, 35))
        with self.assertRaises(AssertionError):
            TermsController.accept_button_position(header)

    def test_fresh_config_has_no_terms_and_one_fixture(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            settings = root / "settings.json"
            workspace = root / "workspace.json"
            PARSER.write_fresh_config(settings, workspace, "/fixture/only.xlsx")
            value = json.loads(settings.read_text(encoding="utf-8"))
            PARSER.validate_fresh_config(value, "/fixture/only.xlsx")
            self.assertNotIn("terms_accepted_version", value)

    def test_fresh_config_rejects_terms_and_extra_tab(self):
        value = {"terms_accepted_version": "0.22.42", "workspace": {
            "last_workspace": "/fixture", "open_tabs": ["/fixture/only.xlsx"],
            "active_tab_idx": 0, "restore_session": True,
        }}
        with self.assertRaises(ValueError):
            PARSER.validate_fresh_config(value, "/fixture/only.xlsx")
        value.pop("terms_accepted_version")
        value["workspace"]["open_tabs"].append("/fixture/other.xlsx")
        with self.assertRaises(ValueError):
            PARSER.validate_fresh_config(value, "/fixture/only.xlsx")


if __name__ == "__main__":
    unittest.main()
