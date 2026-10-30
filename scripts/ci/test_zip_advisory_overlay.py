import datetime as dt
import importlib.util
import json
import sys
import unittest
import tomllib
from pathlib import Path
from unittest import mock

from test_zstd_advisory_policy import metadata as zstd_metadata, node as zstd_node
import zip_advisory_policy as zip_policy


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


def direct_workspace_zip_metadata(zip_version="9.1.2", parent="katana-core", parent_version="0.22.42"):
    repo_root = zip_policy.SAFE_ZIP_MANIFEST.parents[2]
    core_manifest = zip_policy.SAFE_ZIP_MANIFEST
    screenshot_manifest = repo_root / "scripts" / "screenshot" / "Cargo.toml"
    zip_id = f"registry+https://github.com/rust-lang/crates.io-index#zip@{zip_version}"
    parent_id = f"path+{core_manifest.parent.as_uri()}#{parent_version}"
    root_id = f"path+{screenshot_manifest.parent.as_uri()}#katana-screenshot@0.1.0"
    packages = [{"id": zip_id, "name": "zip", "version": zip_version,
                 "source": "registry+https://github.com/rust-lang/crates.io-index"},
                {"id": parent_id, "name": parent, "version": parent_version, "source": None,
                 "manifest_path": str(core_manifest)},
                {"id": root_id, "name": "katana-screenshot", "version": "0.1.0",
                 "source": None, "manifest_path": str(screenshot_manifest)}]
    nodes = [{"id": zip_id, "deps": []}, {"id": parent_id, "deps": [{"pkg": zip_id}]},
             {"id": root_id, "deps": [{"pkg": parent_id}]}]
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

    def test_exact_current_direct_zip_edge_is_accepted(self):
        for fixture in (
            direct_workspace_zip_metadata(zip_version="9.0.5", parent_version="0.22.42"),
            direct_workspace_zip_metadata(zip_version="9.1.2", parent_version="0.22.42"),
            direct_workspace_zip_metadata(zip_version="9.1.3", parent_version="0.22.43"),
            direct_workspace_zip_metadata(zip_version="9.1.3", parent_version="0.22.44"),
        ):
            with self.subTest(packages=fixture["packages"]):
                MODULE.validate_metadata(fixture)

    def test_unapproved_direct_zip_version_or_parent_is_rejected(self):
        for fixture in (
            direct_workspace_zip_metadata(zip_version="9.1.3", parent_version="0.22.42"),
            direct_workspace_zip_metadata(zip_version="9.1.4", parent_version="0.22.43"),
            direct_workspace_zip_metadata(zip_version="9.1.4", parent_version="0.22.44"),
            direct_workspace_zip_metadata(zip_version="9.1.3", parent_version="0.22.45"),
            direct_workspace_zip_metadata(
                zip_version="9.1.3", parent="other-root", parent_version="0.22.43"
            ),
        ):
            with self.subTest(packages=fixture["packages"]):
                with self.assertRaises(ValueError):
                    MODULE.validate_metadata(fixture)

    def test_safe_direct_zip_edge_requires_local_manifest_and_reachable_route(self):
        for approved in (
            {"zip_version": "9.1.2", "parent_version": "0.22.42"},
            {"zip_version": "9.1.3", "parent_version": "0.22.43"},
            {"zip_version": "9.1.3", "parent_version": "0.22.44"},
        ):
            with self.subTest(approved=approved):
                git_parent = direct_workspace_zip_metadata(**approved)
                next(package for package in git_parent["packages"] if package["name"] == "katana-core")["source"] = (
                    "git+https://example.invalid/katana-core")
                wrong_zip_source = direct_workspace_zip_metadata(**approved)
                next(package for package in wrong_zip_source["packages"] if package["name"] == "zip")["source"] = (
                    "git+https://example.invalid/zip")
                other_path = direct_workspace_zip_metadata(**approved)
                next(package for package in other_path["packages"] if package["name"] == "katana-core")["manifest_path"] = (
                    "/workspace/fork/crates/katana-core/Cargo.toml")
                orphaned = direct_workspace_zip_metadata(**approved)
                root_id = next(package["id"] for package in orphaned["packages"]
                               if package["name"] == "katana-screenshot")
                next(node for node in orphaned["resolve"]["nodes"]
                     if node["id"] == root_id)["deps"] = []
                for fixture in (git_parent, wrong_zip_source, other_path, orphaned):
                    with self.subTest(fixture=fixture["packages"]):
                        with self.assertRaises(ValueError):
                            MODULE.validate_metadata(fixture)

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

    def test_runner_uses_per_manifest_scoped_configs_and_removes_them(self):
        affected = zstd_metadata()
        modern = zstd_metadata()
        next(p for p in modern["packages"] if p["id"] == "safe")["version"] = "8.2.0"
        metadata_results = iter((affected, modern))
        configs, temporary_files = [], []
        manifests = [ROOT / "root.toml", ROOT / "modern.toml"]
        deny = ROOT / "../../deny.toml"
        original = deny.read_bytes()

        def command(args, **kwargs):
            if args[:2] == ["cargo", "metadata"]:
                self.assertIn("--locked", args)
                self.assertIn("--all-features", args)
                return mock.Mock(returncode=0, stdout=json.dumps(next(metadata_results)), stderr="")
            self.assertEqual(args[0], "cargo-deny")
            self.assertEqual(args[-2:], ["check", "--hide-inclusion-graph"])
            self.assertIn("--locked", args)
            path = Path(args[args.index("--config") + 1])
            temporary_files.append(path)
            config = tomllib.loads(path.read_text(encoding="utf-8"))
            configs.append(config)
            self.assertEqual(args[args.index("--manifest-path") + 1], str(manifests[len(configs) - 1]))
            for section in ("bans", "licenses", "sources"):
                self.assertEqual(config[section], tomllib.loads(original.decode())[section])
            return mock.Mock(returncode=0)

        with mock.patch.object(MODULE.subprocess, "run", side_effect=command):
            self.assertEqual(MODULE.run("cargo-deny", deny, manifests), 0)
        ids = [[entry["id"] if isinstance(entry, dict) else entry
                for entry in config["advisories"]["ignore"]] for config in configs]
        self.assertIn("RUSTSEC-2026-0403", ids[0])
        self.assertNotIn("RUSTSEC-2026-0403", ids[1])
        self.assertTrue(all(not path.exists() for path in temporary_files))
        self.assertEqual(deny.read_bytes(), original)

    def test_runner_rejects_unapproved_path_before_writing_or_deny(self):
        data = zstd_metadata()
        zstd_node(data, "root")["deps"].append({"pkg": "safe"})
        result = mock.Mock(returncode=0, stdout=json.dumps(data), stderr="")
        with mock.patch.object(MODULE.subprocess, "run", return_value=result) as command, \
                mock.patch.object(MODULE.tempfile, "NamedTemporaryFile") as temporary:
            self.assertEqual(MODULE.run("cargo-deny", ROOT / "../../deny.toml", [ROOT / "Cargo.toml"]), 78)
            self.assertEqual(command.call_count, 1)
            temporary.assert_not_called()

    def test_runner_preserves_metadata_failure_and_does_not_write_config(self):
        result = mock.Mock(returncode=23, stdout="", stderr="locked resolution failed\n")
        with mock.patch.object(MODULE.subprocess, "run", return_value=result) as command, \
                mock.patch.object(MODULE.tempfile, "NamedTemporaryFile") as temporary:
            self.assertEqual(MODULE.run("cargo-deny", ROOT / "../../deny.toml", [ROOT / "Cargo.toml"]), 23)
            self.assertEqual(command.call_count, 1)
            temporary.assert_not_called()

    def test_runner_rejects_malformed_metadata_before_config(self):
        for output in ("not-json", "null", '{"resolve": null}'):
            with self.subTest(output=output):
                result = mock.Mock(returncode=0, stdout=output, stderr="")
                with mock.patch.object(MODULE.subprocess, "run", return_value=result) as command, \
                        mock.patch.object(MODULE.tempfile, "NamedTemporaryFile") as temporary:
                    self.assertEqual(MODULE.run("cargo-deny", ROOT / "../../deny.toml", [ROOT / "Cargo.toml"]), 78)
                    self.assertEqual(command.call_count, 1)
                    temporary.assert_not_called()

    def test_runner_expiry_prevents_metadata_config_and_deny(self):
        with mock.patch.object(MODULE, "policy_expired", return_value=True), \
                mock.patch.object(MODULE.subprocess, "run") as command, \
                mock.patch.object(MODULE.tempfile, "NamedTemporaryFile") as temporary:
            self.assertEqual(MODULE.run("cargo-deny", ROOT / "../../deny.toml", [ROOT / "Cargo.toml"]), 78)
            command.assert_not_called()
            temporary.assert_not_called()

    def test_scoped_runner_preserves_deny_failure_and_stops_next_manifest(self):
        metadata_result = mock.Mock(returncode=0, stdout=json.dumps(zstd_metadata()), stderr="")
        deny_result = mock.Mock(returncode=19)
        with mock.patch.object(MODULE.subprocess, "run", side_effect=[metadata_result, deny_result]) as command:
            self.assertEqual(MODULE.run("cargo-deny", ROOT / "../../deny.toml", [ROOT / "one.toml", ROOT / "two.toml"]), 19)
            self.assertEqual(command.call_count, 2)


if __name__ == "__main__":
    unittest.main()
