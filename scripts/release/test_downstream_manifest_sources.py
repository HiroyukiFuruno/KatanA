"""Source identity regression tests for manifest dependency snapshots."""

from __future__ import annotations

import unittest

from downstream_dependency_changes import DependencySource, manifest_snapshot


class ManifestSourceTests(unittest.TestCase):
    def test_git_locator_and_selector_are_structurally_distinct(self) -> None:
        embedded = '[dependencies]\nkatana-x = { git = "file:///tmp/source|rev=main" }\n'
        separate = '[dependencies]\nkatana-x = { git = "file:///tmp/source", rev = "main" }\n'
        embedded_source = manifest_snapshot(embedded, frozenset())[0].source
        separate_source = manifest_snapshot(separate, frozenset())[0].source
        self.assertNotEqual(embedded_source, separate_source)
        self.assertEqual(embedded_source, DependencySource("git", "file:///tmp/source|rev=main"))
        self.assertEqual(separate_source, DependencySource("git", "file:///tmp/source", (("rev", "main"),)))

    def test_path_registry_and_git_sources_have_distinct_kinds(self) -> None:
        sources = []
        for field in ("path", "registry", "git"):
            content = f'[dependencies]\nkatana-x = {{ {field} = "same" }}\n'
            sources.append(manifest_snapshot(content, frozenset())[0].source)
        self.assertEqual([source.kind for source in sources], ["path", "registry", "git"])


if __name__ == "__main__":
    unittest.main()
