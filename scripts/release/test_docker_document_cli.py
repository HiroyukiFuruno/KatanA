"""Validate the external observer's command-line format boundary."""

import importlib.util
from pathlib import Path
import unittest


SPEC = importlib.util.spec_from_file_location(
    "document_cli", Path(__file__).with_name("docker-ui-document-cli.py")
)
CLI = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
SPEC.loader.exec_module(CLI)


class DocumentCliTests(unittest.TestCase):
    def test_cli_accepts_all_existing_document_surface_formats(self):
        arguments = [
            "--evidence", "/evidence", "--expected-sha256", "a" * 64,
            "--document", "/fixture/only", "--debug-log", "/evidence/app.log",
            "--receipt-dir", "/evidence/receipts", "--receipt-run-id", "run-1",
        ]
        for format_name in ("docx", "xlsx", "pptx", "pdf"):
            with self.subTest(format=format_name):
                options = CLI.options_from(arguments + ["--format", format_name])
                self.assertEqual(options.format, format_name)
                self.assertFalse(options.accept_terms)

    def test_terms_opt_in_is_explicit_and_fresh_only(self):
        arguments = [
            "--evidence", "/evidence", "--expected-sha256", "a" * 64,
            "--document", "/fixture/only", "--format", "xlsx",
            "--debug-log", "/evidence/app.log", "--receipt-dir", "/evidence/receipts",
            "--receipt-run-id", "run-1", "--accept-terms",
        ]
        self.assertTrue(CLI.options_from(arguments).accept_terms)
        with self.assertRaises(SystemExit):
            CLI.options_from(arguments + ["--prepared-settings", "/prepared/settings.json"])


if __name__ == "__main__":
    unittest.main()
