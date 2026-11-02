#!/usr/bin/env python3
"""Synthetic unit tests for candidate version binding."""

import plistlib
import tempfile
import unittest
from pathlib import Path

from document_host_io import HostEvidenceError
from document_host_version import cargo_version, check_plist


class HostVersionTests(unittest.TestCase):
    def test_exact_cargo_and_bundle_version(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            cargo = root / "Cargo.toml"
            plist = root / "Info.plist"
            cargo.write_text('[workspace.package]\nversion = "0.22.47"\n')
            plist.write_bytes(plistlib.dumps({"CFBundleShortVersionString": "v0.22.47"}))
            self.assertEqual(cargo_version(cargo), "0.22.47")
            check_plist(plist, "0.22.47")
            with self.assertRaises(HostEvidenceError):
                check_plist(plist, "0.22.42")
            cargo.write_text('[workspace.package]\nversion = "0.22.48"\n')
            with self.assertRaises(HostEvidenceError):
                cargo_version(cargo)


if __name__ == "__main__":
    unittest.main()
