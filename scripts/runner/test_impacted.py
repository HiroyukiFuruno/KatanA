import unittest
from unittest import mock

import impacted


WORKSPACE_PACKAGES = [
    {"name": "katana-core", "dependencies": []},
    {"name": "katana-platform", "dependencies": [{"name": "katana-core"}]},
    {
        "name": "katana-ui",
        "dependencies": [{"name": "katana-core"}, {"name": "katana-platform"}],
    },
    {"name": "katana-linter", "dependencies": []},
]
ALL_PACKAGES = {package["name"] for package in WORKSPACE_PACKAGES}


class ImpactedPackageTests(unittest.TestCase):
    def determine_for(self, changed_paths):
        with (
            mock.patch.object(
                impacted,
                "get_workspace_members",
                return_value=(WORKSPACE_PACKAGES, {"nodes": []}),
            ),
            mock.patch.object(impacted, "get_git_diff", return_value=changed_paths),
        ):
            return set(impacted.determine_impacted_packages())

    def test_vendor_changes_impact_every_workspace_package(self):
        self.assertEqual(self.determine_for(["vendor/egui_commonmark_upstream/src/lib.rs"]), ALL_PACKAGES)

    def test_core_changes_include_reverse_dependencies(self):
        self.assertEqual(
            self.determine_for(["crates/katana-core/src/lib.rs"]),
            {"katana-core", "katana-platform", "katana-ui"},
        )

    def test_readme_only_changes_impact_no_packages(self):
        self.assertEqual(self.determine_for(["README.md"]), set())


if __name__ == "__main__":
    unittest.main()
