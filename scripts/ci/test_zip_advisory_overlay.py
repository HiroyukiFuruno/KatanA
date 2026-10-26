import datetime as dt
import importlib.util
import json
import sys
import unittest
from pathlib import Path
from unittest import mock


ROOT = Path(__file__).parent
sys.path.insert(0, str(ROOT))
SPEC = importlib.util.spec_from_file_location("zip_overlay", ROOT / "zip_advisory_overlay.py")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def metadata(zip_version="0.6.6", parent="ironcalc", parent_version="0.8.3"):
    zip_id = f"registry+https://github.com/rust-lang/crates.io-index#zip@{zip_version}"
    parent_id = f"registry+https://github.com/rust-lang/crates.io-index#{parent}@{parent_version}"
    kdv_id = "registry+https://github.com/rust-lang/crates.io-index#katana-document-viewer@0.5.12"
    root_id = "path+file:///workspace#root@1.0.0"
    packages = [{"id": zip_id, "name": "zip", "version": zip_version,
                 "source": "registry+https://github.com/rust-lang/crates.io-index"},
                {"id": parent_id, "name": parent, "version": parent_version,
                 "source": "registry+https://github.com/rust-lang/crates.io-index"},
                {"id": kdv_id, "name": "katana-document-viewer", "version": "0.5.12",
                 "source": "registry+https://github.com/rust-lang/crates.io-index"},
                {"id": root_id, "name": "root", "version": "1.0.0", "source": None}]
    nodes = [{"id": zip_id, "deps": []},
             {"id": parent_id, "deps": [{"pkg": zip_id}]},
             {"id": kdv_id, "deps": [{"pkg": parent_id}]},
             {"id": root_id, "deps": [{"pkg": kdv_id}]}]
    return {"packages": packages, "workspace_members": [root_id], "resolve": {"nodes": nodes}}


class ZipOverlayTests(unittest.TestCase):
    def test_expiry_boundary_is_closed(self):
        self.assertFalse(MODULE.policy_expired(dt.date(2026, 11, 6)))
        self.assertTrue(MODULE.policy_expired(dt.date(2026, 11, 7)))

    def test_overlay_contains_exact_three_ids(self):
        result = MODULE.overlay_config("[advisories]\nignore = [\n]\n")
        self.assertEqual(result.count("RUSTSEC-"), 3)
        self.assertNotIn("allow =", result)

    def test_wrong_advisory_id_is_rejected(self):
        with self.assertRaises(ValueError):
            MODULE.validate_policy_ids(("RUSTSEC-2026-0001",))

    def test_new_zip_version_is_rejected(self):
        with self.assertRaises(ValueError):
            MODULE.validate_metadata(metadata("0.6.7"))

    def test_new_direct_parent_is_rejected(self):
        with self.assertRaises(ValueError):
            MODULE.validate_metadata(metadata(parent="new-parent"))

    def test_missing_metadata_is_rejected(self):
        with self.assertRaises(ValueError):
            MODULE.validate_metadata({"packages": [], "resolve": {"nodes": []}})

    def test_cargo_deny_failure_exit_is_preserved(self):
        metadata_result = mock.Mock(returncode=0, stdout=json.dumps(metadata()), stderr="")
        deny_result = mock.Mock(returncode=17)
        with mock.patch.object(MODULE.subprocess, "run", side_effect=[metadata_result, deny_result]):
            self.assertEqual(MODULE.run("cargo-deny", ROOT / "../../deny.toml", [ROOT / "Cargo.toml"]), 17)

    def test_unknown_dependency_node_is_rejected(self):
        fixture = metadata()
        fixture["resolve"]["nodes"][1]["deps"][0]["pkg"] = "missing"
        with self.assertRaises(ValueError):
            MODULE.validate_metadata(fixture)

    def test_duplicate_package_id_is_rejected(self):
        fixture = metadata()
        fixture["packages"].append(dict(fixture["packages"][0]))
        with self.assertRaises(ValueError):
            MODULE.validate_metadata(fixture)

    def test_wrong_zip_source_is_rejected(self):
        fixture = metadata()
        fixture["packages"][0]["source"] = "git+https://example.invalid/zip"
        with self.assertRaises(ValueError):
            MODULE.validate_metadata(fixture)

    def test_spoof_parent_source_is_rejected(self):
        fixture = metadata()
        fixture["packages"][1]["source"] = "git+https://example.invalid/parent"
        with self.assertRaises(ValueError):
            MODULE.validate_metadata(fixture)

    def test_missing_dependency_node_is_rejected(self):
        fixture = metadata()
        fixture["packages"].append({"id": "registry+https://example/unused", "name": "unused", "version": "1.0.0"})
        fixture["resolve"]["nodes"][2]["deps"].append({"pkg": "registry+https://example/unused"})
        with self.assertRaises(ValueError):
            MODULE.validate_metadata(fixture)

    def test_rogue_workspace_route_bypassing_kdv_is_rejected(self):
        fixture = metadata()
        rogue_id = "path+file:///workspace#rogue@1.0.0"
        fixture["packages"].append({"id": rogue_id, "name": "rogue", "version": "1.0.0", "source": None})
        fixture["workspace_members"].append(rogue_id)
        fixture["resolve"]["nodes"].append({"id": rogue_id, "deps": [{"pkg": fixture["resolve"]["nodes"][0]["id"]}]})
        with self.assertRaises(ValueError):
            MODULE.validate_metadata(fixture)

    def test_old_zip_direct_workspace_root_is_rejected(self):
        fixture = metadata()
        fixture["workspace_members"].append(fixture["resolve"]["nodes"][0]["id"])
        with self.assertRaises(ValueError):
            MODULE.validate_metadata(fixture)

    def test_orphaned_old_zip_parent_chain_is_rejected(self):
        fixture = metadata()
        fixture["resolve"]["nodes"][2]["deps"] = []
        with self.assertRaises(ValueError):
            MODULE.validate_metadata(fixture)

    def test_wrong_section_is_rejected(self):
        with self.assertRaises(ValueError):
            MODULE.overlay_config("[graph]\nignore = []\n[advisories]\nunused = true\n")

    def test_overlay_preserves_other_sections_and_source(self):
        source = "[graph]\ntargets = []\n[advisories]\nignore = [\n]\n[licenses]\nallow = [\"MIT\"]\n"
        result = MODULE.overlay_config(source)
        self.assertIn("[graph]\ntargets = []", result)
        self.assertIn("[licenses]\nallow = [\"MIT\"]", result)
        self.assertEqual(source, "[graph]\ntargets = []\n[advisories]\nignore = [\n]\n[licenses]\nallow = [\"MIT\"]\n")

    def test_checked_in_deny_source_is_unchanged(self):
        deny = ROOT / "../../deny.toml"
        before = deny.read_bytes()
        MODULE.overlay_config(deny.read_text(encoding="utf-8"))
        self.assertEqual(before, deny.read_bytes())


if __name__ == "__main__":
    unittest.main()
