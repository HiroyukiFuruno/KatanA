"""Pure metadata and real-file boundaries for future publication collection."""

import copy
import os
from pathlib import Path
import tempfile
import unittest

from document_host_io import HostEvidenceError
from future_publication_download import download, selected_assets, verify_unchanged
from future_publication_identity import collect, controller_identity, publication_candidate
from generated_asset_identity import assets_for
from generated_identity_fixtures import bundle


def release(version="v0.22.45"):
    return {"id": 456, "tag_name": version, "draft": False, "prerelease": False,
            "published_at": "2026-10-30T01:00:00Z",
            "html_url": f"https://github.com/HiroyukiFuruno/KatanA/releases/tag/{version}",
            "assets": [{"id": index + 1, "name": name, "size": 10, "digest": "sha256:" + "a" * 64,
                        "updated_at": "2026-10-30T01:00:00Z"}
                       for index, name in enumerate(sorted(set().union(*assets_for(version).values()) | {"checksums.txt"}))]}


class FuturePublicationTests(unittest.TestCase):
    def test_actual_candidate_cannot_change_version_source_run_or_attempt(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            commit, _, _, records, _ = bundle(root)
            candidate = records["macos"]
            self.assertEqual(("v0.22.45", commit), publication_candidate(candidate, "123", "2"))
            for run, attempt, version in (("124", "2", None), ("123", "1", None),
                                           ("123", "2", "v0.22.44"), ("-1", "2", None)):
                with self.subTest(run=run, attempt=attempt), self.assertRaises(ValueError):
                    publication_candidate(candidate, run, attempt, version)
            for field, value in (("version", "v0.22.45-extra"), ("provenance", {}), ("platform", "linux")):
                changed = copy.deepcopy(candidate)
                changed[field] = value
                with self.subTest(field=field), self.assertRaises(ValueError):
                    publication_candidate(changed, "123", "2")

    def test_every_public_asset_and_stable_metadata_are_required(self):
        value = release()
        self.assertEqual(6, len(selected_assets(value, "v0.22.45")))
        verify_unchanged(value, copy.deepcopy(value), "v0.22.45")
        for index in range(6):
            changed = copy.deepcopy(value)
            changed["assets"].pop(index)
            with self.subTest(missing=index), self.assertRaises(HostEvidenceError):
                selected_assets(changed, "v0.22.45")
            for field, replacement in (("id", 999), ("size", 11), ("digest", "b" * 64), ("updated_at", "later")):
                changed = copy.deepcopy(value)
                changed["assets"][index][field] = replacement
                with self.subTest(asset=index, field=field), self.assertRaises(HostEvidenceError):
                    verify_unchanged(value, changed, "v0.22.45")
        for field, replacement in (("tag_name", "v0.22.44"), ("draft", True), ("prerelease", True),
                                    ("id", 789), ("published_at", "2026-10-30T02:00:00Z")):
            changed = copy.deepcopy(value)
            changed[field] = replacement
            with self.subTest(field=field), self.assertRaises(ValueError):
                verify_unchanged(value, changed, "v0.22.45")

    def test_no_existing_or_linked_candidate_directory_is_downloaded_or_overwritten(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            work = root / "existing"
            work.mkdir()
            marker = work / "keep"
            marker.write_text("preserved")
            with self.assertRaises(HostEvidenceError):
                collect(root, work, "123", "2")
            with self.assertRaises(HostEvidenceError):
                download(work, "v0.22.45", release())
            self.assertEqual("preserved", marker.read_text())
            if os.name == "posix":
                linked = root / "linked"
                linked.symlink_to(work, target_is_directory=True)
                with self.assertRaises(HostEvidenceError):
                    collect(root, linked, "123", "2")

    def test_actual_controller_must_be_committed_without_broadening_old_scope(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            commit, _, _, _, _ = bundle(root)
            self.assertEqual(commit, controller_identity(root))
            (root / "Cargo.toml").write_text("changed production source")
            with self.assertRaises(HostEvidenceError):
                controller_identity(root)


if __name__ == "__main__":
    unittest.main()
