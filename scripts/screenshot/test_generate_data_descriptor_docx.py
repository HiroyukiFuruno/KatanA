#!/usr/bin/env python3
"""Regression tests for the deterministic DOCX data-descriptor fixture."""

from __future__ import annotations

import hashlib
import importlib.util
import sys
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("generate_data_descriptor_docx.py")
SPEC = importlib.util.spec_from_file_location("generate_data_descriptor_docx", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)

REPO_ROOT = Path(__file__).resolve().parents[2]
FIXTURE_DIR = REPO_ROOT / "scripts/screenshot/fixtures/v0-22-38-multi-format"
SOURCE = FIXTURE_DIR / "representative.docx"
EXPECTED = FIXTURE_DIR / "data-descriptor.docx"
EXPECTED_SHA256 = "a1b7e22021218d314bc2d90c526d6d682981828b67cef6e61d8cb2a71ef5742a"


class DataDescriptorDocxTests(unittest.TestCase):
    def test_fixture_is_deterministic_and_all_entries_use_descriptors(self) -> None:
        first = MODULE.generate(SOURCE)
        second = MODULE.generate(SOURCE)

        self.assertEqual(first, second)
        self.assertEqual(first, EXPECTED.read_bytes())
        self.assertEqual(hashlib.sha256(first).hexdigest(), EXPECTED_SHA256)
        entries = MODULE.descriptor_entries(first)
        self.assertEqual(len(entries), 20)
        self.assertIn("word/document.xml", entries)


if __name__ == "__main__":
    unittest.main()
