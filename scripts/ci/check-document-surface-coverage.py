#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import sys
from pathlib import PurePath
from typing import Any


REQUIRED_FILES = (
    "crates/katana-ui/src/preview_pane/document_surface/painter.rs",
    "crates/katana-ui/src/preview_pane/document_surface/painter_grid.rs",
    "crates/katana-ui/src/preview_pane/document_surface/painter_grid_conditional.rs",
    "crates/katana-ui/src/preview_pane/document_surface/painter_grid_style.rs",
    "crates/katana-ui/src/preview_pane/document_surface/painter_grid_text.rs",
    "crates/katana-ui/src/preview_pane/document_surface/painter_page.rs",
    "crates/katana-ui/src/preview_pane/document_surface/source_io.rs",
)


def fail(message: str) -> None:
    raise ValueError(message)


def verify(report: Any) -> None:
    if not isinstance(report, dict):
        fail("coverage report must be a JSON object")
    data = report.get("data")
    if not isinstance(data, list) or len(data) != 1 or not isinstance(data[0], dict):
        fail("coverage report must contain exactly one data set")
    files = data[0].get("files")
    if not isinstance(files, list):
        fail("coverage report files are missing")

    by_suffix: dict[str, dict[str, Any]] = {}
    for entry in files:
        if not isinstance(entry, dict) or not isinstance(entry.get("filename"), str):
            continue
        normalized = PurePath(entry["filename"]).as_posix()
        for suffix in REQUIRED_FILES:
            if normalized.endswith(suffix):
                if suffix in by_suffix:
                    fail(f"coverage report contains duplicate file: {suffix}")
                by_suffix[suffix] = entry

    missing = [suffix for suffix in REQUIRED_FILES if suffix not in by_suffix]
    if missing:
        fail("coverage report is missing required files: " + ", ".join(missing))

    for suffix, entry in by_suffix.items():
        summary = entry.get("summary")
        lines = summary.get("lines") if isinstance(summary, dict) else None
        if not isinstance(lines, dict):
            fail(f"coverage line summary is missing: {suffix}")
        count = lines.get("count")
        covered = lines.get("covered")
        percent = lines.get("percent")
        if (
            not isinstance(count, int)
            or isinstance(count, bool)
            or count <= 0
            or not isinstance(covered, int)
            or isinstance(covered, bool)
            or covered != count
            or percent != 100.0
        ):
            fail(
                f"strict document coverage failed for {suffix}: "
                f"covered={covered!r}, count={count!r}, percent={percent!r}"
            )


def fixture(percent: float = 100.0) -> dict[str, Any]:
    return {
        "data": [
            {
                "files": [
                    {
                        "filename": f"/workspace/{suffix}",
                        "summary": {
                            "lines": {
                                "count": 1,
                                "covered": 1 if percent == 100.0 else 0,
                                "percent": percent,
                            }
                        },
                    }
                    for suffix in REQUIRED_FILES
                ]
            }
        ]
    }


def self_test() -> None:
    verify(fixture())
    for invalid in (
        fixture(99.0),
        {"data": [{"files": fixture()["data"][0]["files"][:-1]}]},
        {"data": []},
    ):
        try:
            verify(invalid)
        except ValueError:
            continue
        raise AssertionError("invalid coverage fixture was accepted")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("report", nargs="?", default="-")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        print("PASS: strict document surface coverage contract")
        return 0

    if args.report == "-":
        report = json.load(sys.stdin)
    else:
        with open(args.report, encoding="utf-8") as handle:
            report = json.load(handle)
    try:
        verify(report)
    except ValueError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    print("PASS: strict document surface coverage is 100% with 0 uncovered lines")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
