from __future__ import annotations

import unittest

from github_publication import check_repository


class GitHubPublicationTests(unittest.TestCase):
    def test_public_repository_identity_is_required(self) -> None:
        check_repository({"full_name": "HiroyukiFuruno/KatanA", "private": False, "visibility": "public"}, "HiroyukiFuruno/KatanA")

    def test_encoded_release_tag_path_is_not_double_encoded(self) -> None:
        from github_publication import check_release
        release = {"published_at": "2026-01-01T00:00:00Z", "draft": False, "prerelease": False, "tag_name": "v0.5.12", "html_url": "https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.5.12"}
        check_release(release, "HiroyukiFuruno/KatanA", "v0.5.12", "0.5.12", "https://github.com/HiroyukiFuruno/KatanA/releases/tag/%760.5.12")

    def test_private_or_omitted_visibility_is_rejected(self) -> None:
        for metadata in (
            {"full_name": "HiroyukiFuruno/KatanA", "private": True, "visibility": "private"},
            {"full_name": "HiroyukiFuruno/KatanA", "private": False},
            {"full_name": "other/KatanA", "private": False, "visibility": "public"},
            {"full_name": None, "private": False, "visibility": "public"},
        ):
            with self.assertRaises(ValueError):
                check_repository(metadata, "HiroyukiFuruno/KatanA")


if __name__ == "__main__":
    unittest.main()
