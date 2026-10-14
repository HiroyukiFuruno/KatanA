"""Verify that every declared upstream release is publicly downloadable."""

from __future__ import annotations

from contextlib import closing
import hashlib
import json
import tarfile
import time
import tomllib
from io import BytesIO
from pathlib import PurePosixPath
from urllib.error import HTTPError, URLError
from urllib.parse import quote, unquote, urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener

from downstream_evidence_fields import _upstream
from bounded_archive import ArchiveLimitError, BoundedArchiveReader
from github_publication import fetch_and_check

DEADLINE_SECONDS = 30.0
MAX_RECORDS = 32
MAX_INDEX_BYTES = 4 * 1024 * 1024
MAX_CRATE_BYTES = 64 * 1024 * 1024
MAX_TAR_MEMBERS = 4096
MAX_TAR_METADATA_BYTES = 128 * 1024
MAX_TAR_EXPANDED_BYTES = 256 * 1024 * 1024
class PublicationError(ValueError):
    """An upstream publication is missing or fails its identity contract."""


class _NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, new):
        raise PublicationError("redirect is forbidden")
OPENER = build_opener(_NoRedirect())
def _deadline() -> float:
    return time.monotonic() + DEADLINE_SECONDS
def _read(response, limit: int, end: float) -> bytes:
    if time.monotonic() >= end:
        raise PublicationError("publication deadline exceeded")
    chunks: list[bytes] = []
    total = 0
    while True:
        chunk = response.read(min(64 * 1024, limit - total + 1))
        if not chunk:
            return b"".join(chunks)
        chunks.append(chunk)
        total += len(chunk)
        if total > limit:
            raise PublicationError("publication response exceeds size limit")
        if time.monotonic() >= end:
            raise PublicationError("publication deadline exceeded")
def _request(url: str, limit: int, end: float) -> bytes:
    parsed = urlsplit(url)
    if parsed.scheme != "https" or parsed.netloc.casefold() not in {"api.github.com", "index.crates.io", "static.crates.io"}:
        raise PublicationError("publication host is not fixed")
    if time.monotonic() >= end:
        raise PublicationError("publication deadline exceeded")
    try:
        with OPENER.open(Request(url, headers={"Accept": "application/json"}), timeout=max(0.1, end - time.monotonic())) as response:
            return _read(response, limit, end)
    except (HTTPError, URLError, TimeoutError, OSError) as error:
        raise PublicationError("publication request failed") from error
def _verify_release(record: dict[str, object], end: float) -> None:
    repository = str(record["repository"])
    version = str(record["version"])
    tag = unquote(urlsplit(str(record["release_url"])).path.rsplit("/", 1)[-1])
    try:
        fetch_and_check(repository, tag, version, str(record["release_url"]), end)
    except ValueError as error:
        raise PublicationError("GitHub publication identity check failed") from error
def _index_path(package: str) -> str:
    name = package.casefold()
    if len(name) == 1:
        return f"1/{name}"
    if len(name) == 2:
        return f"2/{name}"
    if len(name) == 3:
        return f"3/{name[0]}/{name}"
    return f"{name[:2]}/{name[2:4]}/{name}"
def _verify_registry(record: dict[str, object], end: float) -> None:
    package, version = str(record["package"]), str(record["version"])
    index_url = f"https://index.crates.io/{_index_path(package)}"
    lines = _request(index_url, MAX_INDEX_BYTES, end).splitlines()
    entry = None
    try:
        for line in lines:
            candidate = json.loads(line)
            if isinstance(candidate, dict) and candidate.get("vers") == version:
                entry = candidate
                break
    except json.JSONDecodeError as error:
        raise PublicationError("registry index is malformed") from error
    if not isinstance(entry, dict) or entry.get("name") != package or entry.get("yanked") is not False:
        raise PublicationError("registry release is missing or yanked")
    checksum = entry.get("cksum")
    if checksum != record["registry_checksum"]:
        raise PublicationError("registry checksum mismatch")
    crate_url = f"https://static.crates.io/crates/{quote(package)}/{quote(package + '-' + version)}.crate"
    crate = _request(crate_url, MAX_CRATE_BYTES, end)
    if hashlib.sha256(crate).hexdigest() != checksum:
        raise PublicationError("crate checksum mismatch")
    _verify_crate(crate, str(record["repository"]), package, version, end)
def _scan_crate(archive, expected: str, end: float | None) -> bytes:
    metadata = expanded = count = 0
    cargo_raw = None
    for member in archive:
        count += 1
        if count > MAX_TAR_MEMBERS:
            raise PublicationError("crate has too many members")
        path = PurePosixPath(member.name)
        if path.is_absolute() or ".." in path.parts or member.issym() or member.islnk():
            raise PublicationError("crate has unsafe members")
        metadata += len(member.name.encode())
        if metadata > MAX_TAR_METADATA_BYTES:
            raise PublicationError("crate metadata exceeds size limit")
        expanded += max(0, member.size)
        if expanded > MAX_TAR_EXPANDED_BYTES:
            raise PublicationError("crate expanded size exceeds limit")
        if end is not None and time.monotonic() >= end:
            raise PublicationError("publication deadline exceeded")
        if member.name == expected:
            if cargo_raw is not None or not member.isfile() or member.size > MAX_TAR_METADATA_BYTES:
                raise PublicationError("crate Cargo.toml is unsafe or duplicated")
            cargo_raw = archive.extractfile(member).read(MAX_TAR_METADATA_BYTES + 1)
    if cargo_raw is None:
        raise PublicationError("crate Cargo.toml is unsafe or missing")
    return cargo_raw
def _verify_crate(data: bytes, repository: str, package: str, version: str, end: float | None = None) -> None:
    expected = f"{package}-{version}/Cargo.toml"
    try:
        with closing(BoundedArchiveReader(data, MAX_TAR_EXPANDED_BYTES, end)) as reader:
            with tarfile.open(fileobj=reader, mode="r|") as archive:
                raw = _scan_crate(archive, expected, end)
        if len(raw) > MAX_TAR_METADATA_BYTES:
            raise PublicationError("crate metadata exceeds size limit")
        package_data = tomllib.loads(raw.decode("utf-8")).get("package", {})
    except (ArchiveLimitError, OSError, tarfile.TarError, UnicodeError, tomllib.TOMLDecodeError, AttributeError) as error:
        raise PublicationError("crate archive is malformed") from error
    if not isinstance(package_data, dict) or package_data.get("name") != package or package_data.get("version") != version:
        raise PublicationError("crate package identity mismatch")
    declared = package_data.get("repository")
    if not isinstance(declared, str) or urlsplit(declared).netloc.casefold() != "github.com" or urlsplit(declared).path.strip("/").removesuffix(".git") != repository:
        raise PublicationError("crate repository identity mismatch")
def _validated_unique(records: list[dict]) -> list[dict]:
    if not isinstance(records, list) or not records or len(records) > MAX_RECORDS:
        raise PublicationError("upstream records must contain 1..32 entries")
    checked: dict[tuple[str, str, str], str] = {}
    unique: list[dict] = []
    for record in records:
        try:
            key = _upstream(record)
        except ValueError as error:
            raise PublicationError("invalid upstream record") from error
        checksum = record["registry_checksum"]
        if key in checked and checked[key] != checksum:
            raise PublicationError("duplicate upstream has different checksum")
        if key in checked:
            continue
        checked[key] = checksum
        unique.append(record)
    return unique


def verify_upstreams(records: list[dict]) -> None:
    unique = _validated_unique(records)
    end = _deadline()
    for record in unique:
        _verify_release(record, end)
        _verify_registry(record, end)
