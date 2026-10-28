#!/usr/bin/env python3
"""Validate the executable activation contract for the real Win32 subclass tests."""

from pathlib import Path
import unittest
import xml.etree.ElementTree as ET


ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "crates/katana-ui/windows.manifest"
NAMESPACE = "{urn:schemas-microsoft-com:asm.v1}"


class WindowsCommonControlsManifestTests(unittest.TestCase):
    def test_executable_identity_precedes_dependencies(self):
        assembly = ET.parse(MANIFEST).getroot()
        self.assertEqual(assembly.tag, f"{NAMESPACE}assembly")
        self.assertEqual(assembly.attrib["manifestVersion"], "1.0")
        identity = list(assembly)[0]
        self.assertEqual(identity.tag, f"{NAMESPACE}assemblyIdentity")
        self.assertEqual(identity.attrib["name"], "HiroyukiFuruno.KatanA")
        self.assertEqual(identity.attrib["type"], "win32")
        version = identity.attrib["version"].split(".")
        self.assertEqual(len(version), 4)
        self.assertTrue(all(part.isdigit() and int(part) <= 65535 for part in version))

    def test_subclass_functions_activate_the_v6_shared_assembly(self):
        assembly = ET.parse(MANIFEST).getroot()
        path = f"{NAMESPACE}dependency/{NAMESPACE}dependentAssembly/{NAMESPACE}assemblyIdentity"
        identities = assembly.findall(path)
        self.assertEqual(len(identities), 1)
        self.assertEqual(identities[0].attrib, {
            "type": "win32",
            "name": "Microsoft.Windows.Common-Controls",
            "version": "6.0.0.0",
            "processorArchitecture": "*",
            "publicKeyToken": "6595b64144ccf1df",
            "language": "*",
        })


if __name__ == "__main__":
    unittest.main()
