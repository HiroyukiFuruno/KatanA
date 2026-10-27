import copy
import datetime as dt
import tomllib
import unittest

import zstd_advisory_policy as policy
from zip_advisory_policy import ADVISORIES, EXPIRES_ON


TODAY = dt.date(2026, 11, 6)


def metadata():
    selected = {
        "root": ("katana-screenshot", "0.1.0"),
        "core": ("katana-core", "0.22.42"),
        "kdv": ("katana-document-viewer", "0.5.12"),
        "iron": ("ironcalc", "0.8.3"),
        "office": ("office2pdf", "0.8.1"),
        "zip": ("zip", "0.6.6"),
        "zstd": ("zstd", "0.11.2+zstd.1.5.2"),
        "safe": ("zstd-safe", "5.0.2+zstd.1.5.2"),
        "zipnew": ("zip", "9.1.2"),
        "zstdnew": ("zstd", "0.14.3"),
        "safenew": ("zstd-safe", "8.2.0"),
    }
    edges = {"root": ["core", "kdv"],
             "core": ["zipnew"], "kdv": ["iron", "office"],
             "iron": ["zip"], "office": ["zip"], "zip": ["zstd"],
             "zstd": ["safe"], "safe": [], "zipnew": ["zstdnew"],
             "zstdnew": ["safenew"], "safenew": []}
    return {"packages": [
        {"id": pid, "name": name, "version": version,
         "source": None if pid in {"root", "core"} else policy.REGISTRY,
         **({"manifest_path": str(policy.zip_policy.SAFE_ZIP_MANIFEST)} if pid == "core" else {})}
        for pid, (name, version) in selected.items()],
        "workspace_members": ["root"],
        "resolve": {"nodes": [
            {"id": pid, "deps": [{"pkg": child} for child in children]}
            for pid, children in edges.items()]}}


def package(data, pid):
    return next(p for p in data["packages"] if p["id"] == pid)


def node(data, pid):
    return next(n for n in data["resolve"]["nodes"] if n["id"] == pid)


class ZstdPolicyTests(unittest.TestCase):
    def reject(self, data):
        with self.assertRaises(ValueError):
            policy.validate_metadata(data, TODAY)

    def test_exact_legacy_and_modern_coexistence_needs_exception(self):
        self.assertTrue(policy.validate_metadata(metadata(), TODAY))

    def test_modern_only_and_graph_without_kdv_need_no_exception(self):
        data = metadata()
        data["packages"] = [p for p in data["packages"] if p["id"] in {"root", "core", "zipnew", "zstdnew", "safenew"}]
        data["resolve"]["nodes"] = [n for n in data["resolve"]["nodes"] if n["id"] in {"root", "core", "zipnew", "zstdnew", "safenew"}]
        node(data, "root")["deps"] = [{"pkg": "core"}]
        node(data, "core")["deps"] = [{"pkg": "zipnew"}]
        self.assertFalse(policy.validate_metadata(data, TODAY))
        node(data, "root")["deps"] = []
        data["packages"] = [package(data, "root")]
        data["resolve"]["nodes"] = [node(data, "root")]
        self.assertFalse(policy.validate_metadata(data, TODAY))

    def test_affected_version_and_build_are_exact(self):
        for version in ("4.0.0", "5.0.2", "5.0.2+zstd.1.5.3", "7.2.4", "8.1.9"):
            with self.subTest(version=version):
                data = metadata()
                package(data, "safe")["version"] = version
                self.reject(data)

    def test_patched_versions_need_no_exception(self):
        for version in ("8.2.0", "8.2.0+zstd.1.5.7", "8.2.1", "9.0.0"):
            with self.subTest(version=version):
                data = metadata()
                package(data, "safe")["version"] = version
                self.assertFalse(policy.validate_metadata(data, TODAY))

    def test_malformed_versions_are_rejected(self):
        for version in ("8.2", "8.2.0-rc.1", "5.0.2+", "nonsense", None):
            with self.subTest(version=version):
                data = metadata()
                package(data, "safe")["version"] = version
                self.reject(data)

    def test_expiry_is_shared_and_has_closed_boundary(self):
        self.assertEqual(policy.EXPIRES_ON, EXPIRES_ON)
        for today in (EXPIRES_ON, EXPIRES_ON + dt.timedelta(days=1)):
            with self.subTest(today=today), self.assertRaisesRegex(ValueError, "expired"):
                policy.validate_metadata(metadata(), today)

    def test_selected_versions_and_registry_sources_are_fixed(self):
        for pid in ("kdv", "iron", "office", "zip", "zstd", "safe"):
            with self.subTest(pid=pid):
                data = metadata()
                package(data, pid)["version"] = "1.0.0"
                if pid == "safe":
                    package(data, pid)["version"] = "5.0.3"
                self.reject(data)
                data = metadata()
                package(data, pid)["source"] = "git+https://example.invalid/replacement"
                self.reject(data)

    def test_unknown_zip_path_is_rejected_even_without_legacy_zstd(self):
        data = metadata()
        package(data, "safe")["version"] = "8.2.0"
        package(data, "zip")["version"] = "0.6.7"
        self.reject(data)

    def test_additional_affected_version_is_rejected(self):
        data = metadata()
        data["packages"].append({"id": "other", "name": "zstd-safe", "version": "7.2.4", "source": policy.REGISTRY})
        data["resolve"]["nodes"].append({"id": "other", "deps": []})
        self.reject(data)

    def test_new_direct_callers_are_rejected(self):
        for target in ("zip", "zstd", "safe", "iron", "office"):
            with self.subTest(target=target):
                data = metadata()
                data["packages"].append({"id": "other", "name": "new-caller", "version": "1.0.0", "source": policy.REGISTRY})
                data["resolve"]["nodes"].append({"id": "other", "deps": [{"pkg": target}]})
                node(data, "kdv")["deps"].append({"pkg": "other"})
                self.reject(data)

    def test_direct_roots_and_kdv_bypasses_are_rejected(self):
        for target in ("zip", "zstd", "safe", "iron", "office"):
            with self.subTest(target=target):
                data = metadata()
                data["workspace_members"].append(target)
                self.reject(data)
                data = metadata()
                node(data, "root")["deps"].append({"pkg": target})
                self.reject(data)

    def test_missing_or_orphaned_legacy_nodes_are_rejected(self):
        for pid in ("zstd", "safe", "kdv"):
            with self.subTest(pid=pid):
                data = metadata()
                data["packages"] = [p for p in data["packages"] if p["id"] != pid]
                self.reject(data)
        data = metadata()
        node(data, "zip")["deps"] = []
        self.reject(data)
        data = metadata()
        node(data, "zstd")["deps"] = []
        self.reject(data)

    def test_malformed_graphs_are_rejected(self):
        changes = (
            lambda data: data.update(resolve=None),
            lambda data: data.update(workspace_members=[[]]),
            lambda data: data["resolve"]["nodes"].append(copy.deepcopy(node(data, "root"))),
            lambda data: data["resolve"]["nodes"].remove(node(data, "safe")),
            lambda data: node(data, "root").update(deps=[{"pkg": []}]),
        )
        for change in changes:
            with self.subTest(change=change):
                data = metadata()
                change(data)
                self.reject(data)

    def test_overlay_adds_only_0403_and_preserves_existing_config(self):
        source = '[advisories]\nignore = [\n  { id = "RUSTSEC-2020-0001", reason = "existing" },\n]\n[bans]\nmultiple-versions = "deny"\n[licenses]\nallow = ["MIT"]\n[sources]\nunknown-registry = "deny"\n'
        output = policy.overlay_config(source, metadata(), TODAY)
        config = tomllib.loads(output)
        self.assertEqual([entry["id"] for entry in config["advisories"]["ignore"]],
                         ["RUSTSEC-2020-0001", *ADVISORIES, policy.ADVISORY_ID])
        self.assertEqual(config["bans"], {"multiple-versions": "deny"})
        self.assertEqual(config["licenses"], {"allow": ["MIT"]})
        self.assertEqual(config["sources"], {"unknown-registry": "deny"})
        self.assertIn(EXPIRES_ON.isoformat(), config["advisories"]["ignore"][-1]["reason"])
        data = metadata()
        package(data, "safe")["version"] = "8.2.0"
        self.assertNotIn(policy.ADVISORY_ID, policy.overlay_config(source, data, TODAY))


if __name__ == "__main__":
    unittest.main()
