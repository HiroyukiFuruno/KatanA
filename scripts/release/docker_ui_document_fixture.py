"""Native fresh-workspace fixture discovery and opening."""

import csv
import io
import time
from pathlib import Path


def fixture_row_position(tsv: str, basename: str, window_width: int) -> tuple[int, int]:
    rows = list(csv.DictReader(io.StringIO(tsv), delimiter="\t"))
    matches = [
        row for row in rows
        if row.get("text", "").strip() == basename
        and int(row.get("left", "-1")) < window_width * 0.25
    ]
    if len(matches) != 1:
        raise AssertionError("fixture basename must appear once in the left explorer pane")
    row = matches[0]
    return (int(row["left"]) + int(row["width"]) // 2,
            int(row["top"]) + int(row["height"]) // 2)


def open_fixture(controller, deadline: float) -> None:
    basename = Path(controller.document).name
    while time.monotonic() < deadline:
        controller.acceptance.capture("fixture-open-probe")
        probe = controller.acceptance.last_capture
        assert probe is not None, "fixture explorer capture is missing"
        tsv = controller.acceptance.base.command(
            "tesseract", str(probe), "stdout", "--psm", "3", "tsv"
        )
        try:
            width = int(controller.acceptance.geometry()["WIDTH"])
            x, y = fixture_row_position(tsv, basename, width)
        except (AssertionError, KeyError, ValueError):
            time.sleep(controller.acceptance.base.POLL_SECONDS)
            continue
        controller.acceptance.base.command(
            "xdotool", "mousemove", "--window", controller.acceptance.window_id,
            str(x), str(y), "click", "1", "click", "1",
        )
        controller.acceptance.record(
            "fresh-fixture-native-open", basename=basename, x=x, y=y,
            left_pane_max_fraction=0.25, native_ui=True,
        )
        return
    raise AssertionError("fresh fixture was not uniquely visible in the explorer")
