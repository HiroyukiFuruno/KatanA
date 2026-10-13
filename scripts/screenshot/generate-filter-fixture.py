#!/usr/bin/env python3
"""Create the deterministic XLSX fixture used by the host filter bridge tests."""

from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "scripts/screenshot/fixtures/v0-22-38-multi-format/representative.xlsx"
TARGET = ROOT / "scripts/screenshot/fixtures/v0-22-42-spreadsheet-filter/representative-filter.xlsx"
FILTER = (
    '<autoFilter ref="A3:F7"><filterColumn colId="0">'
    '<filters><filter val="North"/></filters></filterColumn></autoFilter>'
)


def main() -> None:
    TARGET.parent.mkdir(parents=True, exist_ok=True)
    with ZipFile(SOURCE) as source, ZipFile(TARGET, "w", ZIP_DEFLATED) as target:
        for entry in source.infolist():
            content = source.read(entry.filename)
            if entry.filename == "xl/worksheets/sheet1.xml":
                text = content.decode("utf-8")
                text = text.replace("</worksheet>", f"{FILTER}</worksheet>", 1)
                content = text.encode("utf-8")
            target.writestr(entry, content)


if __name__ == "__main__":
    main()
