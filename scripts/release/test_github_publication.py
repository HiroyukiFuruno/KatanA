from __future__ import annotations

import unittest

from github_publication import check_release, check_repository


class GitHubPublicationTests(unittest.TestCase):
    def test_public_repository_identity_is_required(self) -> None:
        check_repository({"full_name": "HiroyukiFuruno/KatanA", "private": False, "visibility": "public"}, "HiroyukiFuruno/KatanA")

    def test_encoded_release_tag_path_is_not_double_encoded(self) -> None:
        from github_publication import check_release
        release = {"published_at": "2026-01-01T00:00:00Z", "draft": False, "prerelease": False, "tag_name": "v0.5.12", "html_url": "https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.5.12"}
        check_release(release, "HiroyukiFuruno/KatanA", "v0.5.12", "0.5.12", "https://github.com/HiroyukiFuruno/KatanA/releases/tag/%760.5.12")

    def test_nested_release_tag_preserves_full_identity_and_version_suffix(self) -> None:
        from github_publication import check_release
        release = {"published_at": "2026-01-01T00:00:00Z", "draft": False, "prerelease": False,
                   "tag_name": "release/v0.5.12",
                   "html_url": "https://github.com/HiroyukiFuruno/KatanA/releases/tag/release/v0.5.12"}
        check_release(release, "HiroyukiFuruno/KatanA", "release/v0.5.12", "0.5.12",
                      "https://github.com/HiroyukiFuruno/KatanA/releases/tag/release%2Fv0.5.12")

    def test_nested_release_tag_must_end_in_requested_version(self) -> None:
        from github_publication import check_release
        release = {"published_at": "2026-01-01T00:00:00Z", "draft": False, "prerelease": False,
                   "tag_name": "release/v0.5.11",
                   "html_url": "https://github.com/HiroyukiFuruno/KatanA/releases/tag/release/v0.5.11"}
        with self.assertRaises(ValueError):
            check_release(release, "HiroyukiFuruno/KatanA", "release/v0.5.11", "0.5.12",
                          "https://github.com/HiroyukiFuruno/KatanA/releases/tag/release%2Fv0.5.11")

    def test_multiple_nested_prefixes_are_encoded_as_one_release_path(self) -> None:
        from github_publication import check_release
        release = {"published_at": "2026-01-01T00:00:00Z", "draft": False, "prerelease": False,
                   "tag_name": "maintenance/candidate/v0.5.12",
                   "html_url": "https://github.com/HiroyukiFuruno/KatanA/releases/tag/maintenance/candidate/v0.5.12"}
        check_release(release, "HiroyukiFuruno/KatanA", "maintenance/candidate/v0.5.12", "0.5.12",
                      "https://github.com/HiroyukiFuruno/KatanA/releases/tag/maintenance%2Fcandidate%2Fv0.5.12")

    def test_literal_percent_escape_is_not_normalized_into_a_slash(self) -> None:
        from github_publication import check_release
        release = {"published_at": "2026-01-01T00:00:00Z", "draft": False, "prerelease": False,
                   "tag_name": "release%2Fname/v0.5.12",
                   "html_url": "https://github.com/HiroyukiFuruno/KatanA/releases/tag/release%252Fname%2Fv0.5.12"}
        check_release(release, "HiroyukiFuruno/KatanA", "release%2Fname/v0.5.12", "0.5.12",
                      "https://github.com/HiroyukiFuruno/KatanA/releases/tag/release%252Fname%2Fv0.5.12")

    def test_literal_percent_escape_is_not_a_version_separator(self) -> None:
        release = {"published_at": "2026-01-01T00:00:00Z", "draft": False, "prerelease": False,
                   "tag_name": "release%2Fv0.5.12",
                   "html_url": "https://github.com/HiroyukiFuruno/KatanA/releases/tag/release%252Fv0.5.12"}
        with self.assertRaisesRegex(ValueError, "tag mismatch"):
            check_release(release, "HiroyukiFuruno/KatanA", "release%2Fv0.5.12", "0.5.12",
                          "https://github.com/HiroyukiFuruno/KatanA/releases/tag/release%252Fv0.5.12")

    def test_literal_percent_tag_cannot_match_normalized_slash_url(self) -> None:
        release = {"published_at": "2026-01-01T00:00:00Z", "draft": False, "prerelease": False,
                   "tag_name": "release%2Fname/v0.5.12",
                   "html_url": "https://github.com/HiroyukiFuruno/KatanA/releases/tag/release/name/v0.5.12"}
        with self.assertRaisesRegex(ValueError, "identity mismatch"):
            check_release(release, "HiroyukiFuruno/KatanA", "release%2Fname/v0.5.12", "0.5.12",
                          "https://github.com/HiroyukiFuruno/KatanA/releases/tag/release%252Fname%2Fv0.5.12")

    def test_nested_release_tag_url_must_match_exactly(self) -> None:
        from github_publication import check_release
        release = {"published_at": "2026-01-01T00:00:00Z", "draft": False, "prerelease": False,
                   "tag_name": "release/v0.5.12",
                   "html_url": "https://github.com/HiroyukiFuruno/KatanA/releases/tag/release/v0.5.12"}
        with self.assertRaises(ValueError):
            check_release(release, "HiroyukiFuruno/KatanA", "release/v0.5.12", "0.5.12",
                          "https://github.com/HiroyukiFuruno/KatanA/releases/tag/release%2Fv0.5.11")

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
