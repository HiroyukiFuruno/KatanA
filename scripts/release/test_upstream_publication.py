from __future__ import annotations

import copy
import gzip
import tarfile
import unittest
from io import BytesIO

from github_publication import check_release
from upstream_publication import (
    PublicationError,
    _index_path,
    _release_tag_from_url,
    _validated_unique,
    _verify_crate,
    verify_upstreams,
)


def record() -> dict[str, str]:
    return {
        "repository": "HiroyukiFuruno/KatanA",
        "package": "katana-render-runtime",
        "version": "0.5.12",
        "release_url": "https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.5.12",
        "registry_checksum": "a" * 64,
    }


class PublicationTests(unittest.TestCase):
    def test_nested_release_tag_is_decoded_once_as_a_full_suffix(self) -> None:
        value = record()
        value["release_url"] = "https://github.com/HiroyukiFuruno/KatanA/releases/tag/release%2Fv0.5.12"
        self.assertEqual(_release_tag_from_url(value["release_url"], value["repository"]), "release/v0.5.12")

    def test_raw_nested_release_tag_keeps_all_components(self) -> None:
        value = record()
        value["release_url"] = "https://github.com/HiroyukiFuruno/KatanA/releases/tag/release/v0.5.12"
        self.assertEqual(_release_tag_from_url(value["release_url"], value["repository"]), "release/v0.5.12")

    def test_multiple_nested_release_tag_prefixes_are_preserved(self) -> None:
        value = record()
        value["release_url"] = "https://github.com/HiroyukiFuruno/KatanA/releases/tag/maintenance/candidate/v0.5.12"
        self.assertEqual(_release_tag_from_url(value["release_url"], value["repository"]), "maintenance/candidate/v0.5.12")

    def test_percent_escaped_percent_remains_literal_after_one_decode(self) -> None:
        value = record()
        value["release_url"] = "https://github.com/HiroyukiFuruno/KatanA/releases/tag/release%252Fv0.5.12"
        self.assertEqual(_release_tag_from_url(value["release_url"], value["repository"]), "release%2Fv0.5.12")

    def test_repository_named_releases_tag_uses_exact_prefix(self) -> None:
        release_url = "https://github.com/releases/tag/releases/tag/v0.5.12"
        self.assertEqual(_release_tag_from_url(release_url, "releases/tag"), "v0.5.12")

    def test_release_tag_url_with_unsafe_empty_component_is_rejected_by_schema(self) -> None:
        value = record()
        value["release_url"] = "https://github.com/HiroyukiFuruno/KatanA/releases/tag/release//v0.5.12"
        with self.assertRaises(PublicationError):
            verify_upstreams([value])

    def test_index_shards(self) -> None:
        self.assertEqual(_index_path("a"), "1/a")
        self.assertEqual(_index_path("ab"), "2/ab")
        self.assertEqual(_index_path("abc"), "3/a/abc")
        self.assertEqual(_index_path("abcd"), "ab/cd/abcd")

    def test_invalid_schema_is_rejected_before_network(self) -> None:
        value = record()
        value["release_url"] = "https://evil.example/releases/tag/v0.5.12"
        with self.assertRaises(PublicationError):
            verify_upstreams([value])

    def test_duplicate_conflicting_checksum_is_rejected(self) -> None:
        first = record()
        second = copy.deepcopy(first)
        second["registry_checksum"] = "b" * 64
        with self.assertRaises(PublicationError):
            verify_upstreams([first, second])

    def test_duplicate_same_checksum_is_collected_once(self) -> None:
        value = record()
        self.assertEqual(len(_validated_unique([value, copy.deepcopy(value)])), 1)

    def test_record_limit_is_rejected(self) -> None:
        with self.assertRaises(PublicationError):
            verify_upstreams([record()] * 33)

    def test_release_negative_states_are_rejected(self) -> None:
        release = {"published_at": "", "draft": False, "prerelease": False, "tag_name": "v0.5.12", "html_url": "https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.5.12"}
        with self.assertRaises(ValueError):
            check_release(release, "HiroyukiFuruno/KatanA", "v0.5.12", "0.5.12", record()["release_url"])
        release["published_at"] = "2026-01-01T00:00:00Z"
        release["draft"] = True
        with self.assertRaises(ValueError):
            check_release(release, "HiroyukiFuruno/KatanA", "v0.5.12", "0.5.12", record()["release_url"])

    def test_crate_tar_identity_and_missing_metadata(self) -> None:
        def make(name: str, version: str, path: str = "katana-render-runtime-0.5.12/Cargo.toml", extras: int = 0) -> bytes:
            out = BytesIO()
            with tarfile.open(fileobj=out, mode="w:gz") as archive:
                data = f'[package]\nname = "{name}"\nversion = "{version}"\nrepository = "https://github.com/HiroyukiFuruno/KatanA"\n'.encode()
                info = tarfile.TarInfo(path)
                info.size = len(data)
                archive.addfile(info, BytesIO(data))
                for index in range(extras):
                    extra = tarfile.TarInfo(f"katana-render-runtime-0.5.12/src/file-{index}.rs")
                    extra.size = 0
                    archive.addfile(extra)
            return out.getvalue()
        _verify_crate(make("katana-render-runtime", "0.5.12"), "HiroyukiFuruno/KatanA", "katana-render-runtime", "0.5.12")
        for data in (make("wrong", "0.5.12"), make("katana-render-runtime", "0.5.11"), make("katana-render-runtime", "0.5.12", "other/Cargo.toml")):
            with self.assertRaises(PublicationError):
                _verify_crate(data, "HiroyukiFuruno/KatanA", "katana-render-runtime", "0.5.12")

    def test_crate_member_capacity_matches_public_shape(self) -> None:
        def crate(extras: int) -> bytes:
            out = BytesIO()
            with tarfile.open(fileobj=out, mode="w:gz") as archive:
                data = b'[package]\nname="katana-render-runtime"\nversion="0.5.12"\nrepository="https://github.com/HiroyukiFuruno/KatanA"\n'
                cargo = tarfile.TarInfo("katana-render-runtime-0.5.12/Cargo.toml")
                cargo.size = len(data)
                archive.addfile(cargo, BytesIO(data))
                for index in range(extras):
                    member = tarfile.TarInfo(f"katana-render-runtime-0.5.12/src/{index}.rs")
                    archive.addfile(member)
            return out.getvalue()
        _verify_crate(crate(919), "HiroyukiFuruno/KatanA", "katana-render-runtime", "0.5.12")
        with self.assertRaises(PublicationError):
            _verify_crate(crate(4096), "HiroyukiFuruno/KatanA", "katana-render-runtime", "0.5.12")

    def test_tiny_archive_with_oversized_member_header_is_rejected(self) -> None:
        out = BytesIO()
        with tarfile.open(fileobj=out, mode="w:gz") as archive:
            member = tarfile.TarInfo("katana-render-runtime-0.5.12/blob")
            member.size = 256 * 1024 * 1024 + 1
            archive.close()
        out = BytesIO(gzip.compress(member.tobuf(format=tarfile.PAX_FORMAT) + b"\0" * 1024))
        with self.assertRaises(PublicationError):
            _verify_crate(out.getvalue(), "HiroyukiFuruno/KatanA", "katana-render-runtime", "0.5.12")


if __name__ == "__main__":
    unittest.main()
