"""Bind host candidate metadata to its actual committed workspace version."""

import plistlib
import sys
import tomllib
from pathlib import Path

from document_host_io import require
from document_host_publication import allowed_publication_version


def cargo_version(path: Path) -> str:
    with path.open("rb") as stream:
        value = tomllib.load(stream)["workspace"]["package"]["version"]
    require(isinstance(value, str), "workspace version missing")
    allowed_publication_version("v" + value)
    return value


def check_plist(path: Path, version: str) -> None:
    with path.open("rb") as stream:
        plist = plistlib.load(stream)
    require(plist.get("CFBundleShortVersionString") == "v" + version,
            "actual bundle version differs from Cargo version")


if __name__ == "__main__":
    if len(sys.argv) == 2:
        print(cargo_version(Path(sys.argv[1])))
    elif len(sys.argv) == 3:
        check_plist(Path(sys.argv[1]), sys.argv[2])
    else:
        raise SystemExit("usage: document_host_version.py <Cargo.toml> | <Info.plist> <version>")
