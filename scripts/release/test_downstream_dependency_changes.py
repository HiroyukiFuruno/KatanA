"""Pure contracts for downstream dependency snapshots."""

from __future__ import annotations

import unittest

from downstream_dependency_changes import (
    DependencySnapshotError,
    classify_lock_changes,
    classify_manifest_changes,
    lock_snapshot,
    manifest_snapshot,
)


INTERNAL = frozenset({"katana-core", "katana-render-runtime"})


class ManifestSnapshotTests(unittest.TestCase):
    def test_formatting_and_order_do_not_change_snapshot(self) -> None:
        first = '[dependencies]\nkatana-render-runtime = { version = "1.0.0", features = ["a"] }\n'
        second = '[dependencies]\n\nkatana-render-runtime={features=["a"],version="1.0.0"}\n'
        self.assertEqual(manifest_snapshot(first, INTERNAL), manifest_snapshot(second, INTERNAL))

    def test_alias_and_all_dependency_scopes_are_classified(self) -> None:
        source = """
        [dependencies]
        runtime = { package = "katana-render-runtime", version = "1.0.0" }
        [dev-dependencies]
        katana-dev = "1.0.0"
        [build-dependencies]
        katana-build = { git = "https://example.invalid/build" }
        [workspace.dependencies]
        katana-workspace = "1.0.0"
        [target.'cfg(unix)'.dependencies]
        katana-target = "1.0.0"
        [patch.crates-io]
        katana-patched = "1.0.0"
        [replace]
        "katana-old:1.0.0" = { git = "https://example.invalid/old" }
        """
        self.assertEqual(len(manifest_snapshot(source, frozenset())), 7)

    def test_path_only_known_internal_is_omitted_but_release_path_change_is_not(self) -> None:
        old = '[dependencies]\nkatana-core = { path = "../core" }\n'
        new = '[dependencies]\nkatana-core = { path = "../core", version = "1.1.0" }\n'
        self.assertEqual(manifest_snapshot(old, INTERNAL), ())
        self.assertEqual(manifest_snapshot(new, INTERNAL), ())

    def test_workspace_alias_unions_features_and_inherits_root_package(self) -> None:
        root = ('[workspace.dependencies]\n'
                'runtime = { package = "katana-render-runtime", version = "1.0.0", features = ["root"] }\n')
        member = '[dependencies]\nruntime = { workspace = true, features = ["member"] }\n'
        result = manifest_snapshot(member, INTERNAL, root)
        self.assertEqual(result[0].package, "katana-render-runtime")
        self.assertEqual(result[0].features, ("member", "root"))

    def test_workspace_root_change_changes_member_snapshot(self) -> None:
        member = '[dependencies]\nruntime = { workspace = true }\n'
        old_root = '[workspace.dependencies]\nruntime = { package = "katana-render-runtime", version = "1.0.0" }\n'
        new_root = '[workspace.dependencies]\nruntime = { package = "katana-render-runtime", version = "1.1.0" }\n'
        changes = classify_manifest_changes(member, member, INTERNAL, old_root, new_root)
        self.assertEqual([change.kind for change in changes], ["changed"])

    def test_workspace_internal_path_is_omitted_even_when_root_version_changes(self) -> None:
        member = '[dependencies]\nkatana-core = { workspace = true }\n'
        old_root = '[workspace.dependencies]\nkatana-core = { path = "../core", version = "1.0.0" }\n'
        new_root = '[workspace.dependencies]\nkatana-core = { path = "../core", version = "1.1.0" }\n'
        self.assertEqual(classify_manifest_changes(member, member, INTERNAL, old_root, new_root), ())

    def test_workspace_missing_alias_and_bad_override_are_rejected(self) -> None:
        member = '[dependencies]\nruntime = { workspace = true }\n'
        with self.assertRaises(DependencySnapshotError):
            manifest_snapshot(member, INTERNAL, '[workspace.dependencies]\nother = "1.0.0"\n')
        bad = '[dependencies]\nruntime = { workspace = true, version = "1.0.0" }\n'
        with self.assertRaises(DependencySnapshotError):
            manifest_snapshot(bad, INTERNAL, '[workspace.dependencies]\nruntime = "1.0.0"\n')

    def test_changes_cover_add_remove_version_features_source_and_alias(self) -> None:
        old = '[dependencies]\nold = { package = "katana-old", version = "1.0.0", features = ["a"] }\n'
        new = '[dependencies]\nnew = { package = "katana-new", git = "https://example.invalid/new", features = ["b"] }\n'
        changes = classify_manifest_changes(old, new, frozenset())
        self.assertEqual([change.kind for change in changes], ["added", "removed"])

    def test_malformed_and_unexpected_manifest_values_are_rejected(self) -> None:
        with self.assertRaises(DependencySnapshotError):
            manifest_snapshot("[dependencies\n", frozenset())
        with self.assertRaises(DependencySnapshotError):
            manifest_snapshot("[dependencies]\nkatana-x = { features = [1] }\n", frozenset())
        with self.assertRaises(DependencySnapshotError):
            manifest_snapshot(42, frozenset())  # type: ignore[arg-type]
        with self.assertRaises(DependencySnapshotError):
            manifest_snapshot("[dependencies]\nkatana-x = \" \"\n", frozenset())
        with self.assertRaises(DependencySnapshotError):
            manifest_snapshot("[dependencies]\nkatana-x = { workspace = true }\n", frozenset())


class LockSnapshotTests(unittest.TestCase):
    def _lock(self, *, version="1.0.0", source="registry", checksum=None, dependencies=None):
        lines = ["[[package]]", 'name="katana-x"', f'version="{version}"']
        if source is not None:
            lines.append(f'source="{source}"')
        if checksum is not None:
            lines.append(f'checksum="{checksum}"')
        if dependencies is not None:
            lines.append("dependencies=[" + ",".join(f'\"{item}\"' for item in dependencies) + "]")
        return "\n".join(lines) + "\n"

    def test_lock_keeps_unknown_local_sibling_and_omits_known_internal(self) -> None:
        content = """
        [[package]]
        name = "katana-core"
        version = "1.0.0"
        [[package]]
        name = "katana-sibling"
        version = "1.0.0"
        dependencies = ["katana-core"]
        """
        result = lock_snapshot(content, INTERNAL)
        self.assertEqual([item.name for item in result], ["katana-sibling"])

    def test_lock_changes_include_version_source_checksum_and_dependencies(self) -> None:
        old = '[[package]]\nname="katana-x"\nversion="1.0.0"\nsource="registry"\nchecksum="a"\n'
        new = '[[package]]\nname="katana-x"\nversion="1.1.0"\nsource="registry"\nchecksum="b"\ndependencies=["katana-y"]\n'
        changes = classify_lock_changes(old, new, frozenset())
        self.assertEqual([change.kind for change in changes], ["changed"])

    def test_lock_checksum_only_change_is_visible(self) -> None:
        changes = classify_lock_changes(self._lock(checksum="a"), self._lock(checksum="b"), frozenset())
        self.assertEqual([change.kind for change in changes], ["changed"])

    def test_lock_dependencies_only_change_is_visible(self) -> None:
        old = self._lock(dependencies=["katana-a"])
        new = self._lock(dependencies=["katana-b"])
        changes = classify_lock_changes(old, new, frozenset())
        self.assertEqual([change.kind for change in changes], ["changed"])

    def test_lock_version_only_change_is_visible(self) -> None:
        changes = classify_lock_changes(self._lock(), self._lock(version="1.1.0"), frozenset())
        self.assertEqual([change.kind for change in changes], ["changed"])

    def test_lock_source_only_change_is_visible(self) -> None:
        changes = classify_lock_changes(self._lock(source="registry"), self._lock(source="git+new"), frozenset())
        self.assertEqual([change.kind for change in changes], ["changed"])

    def test_lock_keeps_removed_version_with_same_name(self) -> None:
        old = ('[[package]]\nname="katana-x"\nversion="1.0.0"\n'
               '[[package]]\nname="katana-x"\nversion="2.0.0"\n')
        new = '[[package]]\nname="katana-x"\nversion="2.0.0"\n'
        changes = classify_lock_changes(old, new, frozenset())
        self.assertEqual([change.kind for change in changes], ["removed"])

    def test_lock_source_selectors_and_none_order_are_preserved(self) -> None:
        content = ('[[package]]\nname="katana-x"\nversion="1.0.0"\n'
                   '[[package]]\nname="katana-x"\nversion="1.0.0"\n'
                   'source="git+https://example.invalid/x?rev=old#old"\n')
        result = lock_snapshot(content, frozenset())
        self.assertIsNone(result[0].source)
        self.assertTrue(result[1].source.startswith("git+"))

    def test_lock_common_keys_sort_source_none_and_text_safely(self) -> None:
        content = ('[[package]]\nname="katana-x"\nversion="1.0.0"\n'
                   '[[package]]\nname="katana-x"\nversion="1.0.0"\nsource="git+same"\n')
        self.assertEqual(classify_lock_changes(content, content, frozenset()), ())

    def test_lock_common_keys_mixed_sources_report_checksum_change(self) -> None:
        old = ('[[package]]\nname="katana-x"\nversion="1.0.0"\nchecksum="old"\n'
               '[[package]]\nname="katana-x"\nversion="1.0.0"\nsource="git+same"\nchecksum="old"\n')
        new = old.replace('checksum="old"', 'checksum="new"', 1)
        changes = classify_lock_changes(old, new, frozenset())
        self.assertEqual([change.kind for change in changes], ["changed"])

    def test_manifest_git_selectors_and_source_kind_are_semantic(self) -> None:
        old = '[dependencies]\nkatana-x = { git = "https://example.invalid/x", rev = "old" }\n'
        new = '[dependencies]\nkatana-x = { git = "https://example.invalid/x", rev = "new" }\n'
        changes = classify_manifest_changes(old, new, frozenset())
        self.assertEqual([change.kind for change in changes], ["changed"])
        path = manifest_snapshot('[dependencies]\nkatana-x = { path = "same" }\n', frozenset())
        git = manifest_snapshot('[dependencies]\nkatana-x = { git = "same" }\n', frozenset())
        self.assertNotEqual(path[0].source, git[0].source)

    def test_malformed_and_unexpected_lock_values_are_rejected(self) -> None:
        with self.assertRaises(DependencySnapshotError):
            lock_snapshot("package = {}\n", frozenset())
        with self.assertRaises(DependencySnapshotError):
            lock_snapshot('[[package]]\nname="katana-x"\nversion=1\n', frozenset())
        duplicate = ('[[package]]\nname="katana-x"\nversion="1.0.0"\n'
                     '[[package]]\nname="katana-x"\nversion="1.0.0"\n')
        with self.assertRaises(DependencySnapshotError):
            classify_lock_changes(duplicate, duplicate, frozenset())


if __name__ == "__main__":
    unittest.main()
