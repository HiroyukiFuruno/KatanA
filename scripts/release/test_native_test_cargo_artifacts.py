import json
import unittest

from native_test_cargo_artifacts import CargoArtifactInventory


def artifact(executable="/tmp/test", fresh=True, **changes):
    event = {"reason": "compiler-artifact", "package_id": "pkg 1.0.0",
             "target": {"name": "unit", "kind": ["bin"],
                        "src_path": "src/main.rs", "crate_types": ["bin"]},
             "profile": {"test": True}, "features": ["default"],
             "executable": executable, "fresh": fresh,
             "manifest_path": "Cargo.toml"}
    event.update(changes)
    return event


def finished(success=True):
    return {"reason": "build-finished", "success": success}


class CargoArtifactInventoryTests(unittest.TestCase):
    def feed(self, *events):
        inventory = CargoArtifactInventory()
        for event in events:
            if isinstance(event, bytes):
                raw = event
            elif isinstance(event, str):
                raw = event.encode()
            else:
                raw = json.dumps(event).encode()
            inventory.consume(raw)
        return inventory.result()

    def test_records_test_artifact_and_fresh(self):
        event = artifact()
        event["target"]["doctest"] = False
        result = self.feed(event, finished(), artifact("/tmp/after-finished"))
        self.assertTrue(result["native_artifacts_complete"])
        self.assertEqual(result["artifacts"][0]["fresh"], True)
        self.assertEqual(result["artifacts"][0]["manifest_path"], "Cargo.toml")
        self.assertFalse(result["artifacts"][0]["target"]["doctest"])

        result["artifacts"][0]["features"].append("mutated")
        inventory = CargoArtifactInventory()
        inventory.consume(json.dumps(event).encode())
        event["target"]["kind"].append("mutated")
        self.assertEqual(inventory.result()["artifacts"][0]["target"]["kind"], ["bin"])
        self.assertEqual(inventory.result()["artifacts"][0]["features"], ["default"])

    def test_exact_duplicates_are_deduplicated(self):
        result = self.feed(artifact(), artifact(), finished())
        self.assertEqual(len(result["artifacts"]), 1)

    def test_conflicting_executable_or_metadata_is_incomplete(self):
        result = self.feed(artifact(), artifact("/tmp/other"), finished())
        self.assertFalse(result["native_artifacts_complete"])
        self.assertIn("conflicting_compiler_artifact", result["reasons"])

    def test_malformed_and_non_cargo_output(self):
        inventory = CargoArtifactInventory()
        inventory.consume(b"running test output\n")
        inventory.consume(b"{not-json}\n")
        inventory.consume(json.dumps({"message": "hello"}).encode())
        result = inventory.result()
        self.assertIn("invalid_cargo_json", result["reasons"])
        self.assertIn("missing_build_finished", result["reasons"])

    def test_missing_nonbool_and_failed_finished(self):
        self.assertIn("missing_build_finished", self.feed()["reasons"])
        self.assertIn("invalid_build_finished", self.feed({"reason": "build-finished",
                                                             "success": 1})["reasons"])
        self.assertIn("build_failed", self.feed(finished(False))["reasons"])

        for change in ({"fresh": 1}, {"manifest_path": ""},
                       {"target": {"name": "", "kind": [], "src_path": "",
                                   "crate_types": []}, "features": [""]}):
            result = self.feed({**artifact(), **change}, finished())
            self.assertFalse(result["native_artifacts_complete"])

        custom = artifact()
        custom["profile"] = {"test": False}
        self.assertEqual(self.feed(custom, finished())["artifacts"], [])

    def test_line_and_artifact_limits(self):
        inventory = CargoArtifactInventory()
        inventory.consume(b"x" * (64 * 1024 + 1))
        for index in range(4097):
            event = artifact(f"/tmp/{index}")
            event["target"]["name"] = f"unit_{index}"
            inventory.consume(json.dumps(event).encode())
        result = inventory.result()
        self.assertIn("line_too_long", result["reasons"])
        self.assertIn("artifact_limit_exceeded", result["reasons"])
        self.assertEqual(len(result["artifacts"]), 4096)


if __name__ == "__main__":
    unittest.main()
