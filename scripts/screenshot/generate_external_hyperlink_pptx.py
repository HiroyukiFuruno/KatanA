#!/usr/bin/env python3
"""Generate the test-only PPTX used to exercise external hyperlink preflight."""

from __future__ import annotations

import argparse
import hashlib
import tempfile
import zipfile
from pathlib import Path


RELATIONSHIPS_NAMESPACE = "http://schemas.openxmlformats.org/package/2006/relationships"
HYPERLINK_RELATIONSHIP_TYPE = (
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink"
)
RELATIONSHIP_MEMBER = "ppt/slides/_rels/slide1.xml.rels"
RELATIONSHIP_ID = "rIdKatanaExternalHyperlink"
EXTERNAL_TARGET = "https://example.invalid/katana-external-hyperlink"
RELATIONSHIPS_END_TAG = b"</Relationships>"
EXTERNAL_HYPERLINK_RELATIONSHIP = (
    b'<Relationship Id="rIdKatanaExternalHyperlink" '
    b'Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" '
    b'Target="https://example.invalid/katana-external-hyperlink" TargetMode="External"/>'
)


def add_external_hyperlink_relationship(relationships: bytes) -> bytes:
    if RELATIONSHIPS_NAMESPACE.encode() not in relationships:
        raise ValueError("PPTX relationship member has an unexpected namespace")
    if RELATIONSHIP_ID.encode() in relationships:
        raise ValueError(f"PPTX already contains {RELATIONSHIP_ID}")
    if RELATIONSHIPS_END_TAG not in relationships:
        raise ValueError("PPTX relationship member is missing its closing tag")
    # This is a fixed byte insertion for test data, not an OOXML parser.
    return relationships.replace(
        RELATIONSHIPS_END_TAG,
        EXTERNAL_HYPERLINK_RELATIONSHIP + RELATIONSHIPS_END_TAG,
        1,
    )


def generate(source: Path, output: Path) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(source) as archive:
        try:
            relationships = archive.read(RELATIONSHIP_MEMBER)
        except KeyError as error:
            raise ValueError(f"PPTX does not contain {RELATIONSHIP_MEMBER}") from error
        members = [
            (entry, archive.read(entry.filename))
            for entry in archive.infolist()
            if entry.filename != RELATIONSHIP_MEMBER
        ]

    patched_relationships = add_external_hyperlink_relationship(relationships)
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for entry, content in members:
            archive.writestr(entry, content)
        archive.writestr(RELATIONSHIP_MEMBER, patched_relationships)


def file_digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def self_test() -> None:
    original_relationships = (
        b'<?xml version="1.0" encoding="UTF-8"?>'
        b'<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
        b'<Relationship Id="rId1" '
        b'Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" '
        b'Target="../slideLayouts/slideLayout1.xml"/>'
        b"</Relationships>"
    )
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        source = root / "source.pptx"
        output = root / "output.pptx"
        with zipfile.ZipFile(source, "w", compression=zipfile.ZIP_DEFLATED) as archive:
            archive.writestr(RELATIONSHIP_MEMBER, original_relationships)
            archive.writestr("ppt/slides/slide1.xml", b"<p:sld/>")
        source_digest = file_digest(source)
        generate(source, output)
        if file_digest(source) != source_digest:
            raise AssertionError("fixture generator modified its source PPTX")
        with zipfile.ZipFile(output) as archive:
            relationships = archive.read(RELATIONSHIP_MEMBER)
        if relationships.count(RELATIONSHIP_ID.encode()) != 1:
            raise AssertionError("fixture generator did not add exactly one hyperlink relationship")
        if EXTERNAL_HYPERLINK_RELATIONSHIP not in relationships:
            raise AssertionError("fixture generator emitted an unexpected hyperlink relationship")
    print("OK: external hyperlink PPTX fixture generator self-test passed.")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    if args.source is None or args.output is None:
        parser.error("--source and --output are required unless --self-test is used")
    generate(args.source, args.output)


if __name__ == "__main__":
    main()
