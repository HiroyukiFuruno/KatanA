#!/usr/bin/env python3
"""Exercise an isolated X11 desktop without asserting native performance."""

import argparse
import csv
import hashlib
import io
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time

from PIL import Image, ImageStat


FUNCTIONAL_DEADLINE_SECONDS = 90
POLL_SECONDS = 0.1


def command(*arguments):
    return subprocess.run(arguments, text=True, capture_output=True, check=True, timeout=15).stdout


def until(label, predicate):
    deadline = time.monotonic() + FUNCTIONAL_DEADLINE_SECONDS
    while time.monotonic() < deadline:
        value = predicate()
        if value:
            return value
        time.sleep(POLL_SECONDS)
    raise AssertionError(f"Functional observation timed out: {label}")


class DesktopAcceptance:
    def __init__(self, evidence, expected_sha256):
        self.root = evidence
        self.output = Path(tempfile.mkdtemp(prefix="functional.", dir=evidence))
        self.window_id = None
        self.report = {"native_performance": False, "terms_accepted": False, "steps": []}
        self.last_capture = None
        self.expected_sha256 = expected_sha256

    def record(self, event, **fields):
        self.report["steps"].append({"event": event, **fields})
        print(json.dumps(self.report["steps"][-1]), flush=True)
        (self.output / "result.json").write_text(json.dumps(self.report, indent=2) + "\n")

    def frame(self):
        heartbeat = self.root / "heartbeat.txt"
        if not heartbeat.is_file():
            return 0
        match = re.search(r"^frame=(\d+)$", heartbeat.read_text(), re.MULTILINE)
        return int(match.group(1)) if match else 0

    def find_window(self):
        result = subprocess.run(
            ["xdotool", "search", "--onlyvisible", "--name", "KatanA"],
            text=True, capture_output=True, timeout=10,
        )
        return result.stdout.splitlines()[0] if result.returncode == 0 else None

    def geometry(self):
        output = command("xdotool", "getwindowgeometry", "--shell", self.window_id)
        return dict(line.split("=", 1) for line in output.splitlines() if "=" in line)

    def capture(self, name):
        path = self.output / f"{name}.png"
        command("import", "-window", self.window_id, str(path))
        with Image.open(path) as image:
            spread = max(ImageStat.Stat(image.convert("RGB")).stddev)
            dimensions = image.size
        text = command("tesseract", str(path), "stdout", "--psm", "6") if spread > 1 else ""
        (self.output / f"{name}.txt").write_text(text)
        self.record(name, size=dimensions, pixel_spread=spread, frame=self.frame(),
                    sha256=hashlib.sha256(path.read_bytes()).hexdigest())
        self.last_capture = path
        return text

    def rendered(self, name, accepts):
        attempts = 0

        def observed():
            nonlocal attempts
            attempts += 1
            text = self.capture(f"{name}-{attempts}")
            return text if text and accepts(text) else None

        return until(f"painted response: {name}", observed)

    def provenance(self):
        pid = int((self.root / "app.pid").read_text())
        binary = self.root / "package" / "KatanA"
        mappings = Path(f"/proc/{pid}/maps").read_text().splitlines()
        paths = {row.split(maxsplit=5)[-1] for row in mappings if len(row.split(maxsplit=5)) == 6}
        assert str(binary) in paths, "Guest ELF is not mapped by the launched process"
        cmdline = Path(f"/proc/{pid}/cmdline").read_bytes().split(b"\0")
        assert os.fsencode(binary) in cmdline, "Guest ELF is not in the launch command"
        digest = hashlib.sha256(binary.read_bytes()).hexdigest()
        assert digest == self.expected_sha256, "Mapped guest ELF differs from the input archive"
        self.record("guest-provenance", pid=pid, image=os.readlink(f"/proc/{pid}/exe"),
                    guest_sha256=digest)

    def startup(self):
        self.window_id = until("visible KatanA window", self.find_window)
        until("completed UI frame", self.frame)
        self.provenance()
        self.rendered("startup", lambda text: "terms" in text.lower())
        self.record("startup-rendered", window=self.window_id)

    def resize(self):
        before = self.frame()
        command("xdotool", "windowsize", self.window_id, "900", "700")
        until("900x700 window geometry", lambda: self.geometry()["WIDTH"] == "900"
              and self.geometry()["HEIGHT"] == "700")
        until("UI completed after resize", lambda: self.frame() > before)
        self.rendered("resized", lambda text: "terms" in text.lower())
        self.record("resize-acknowledged", geometry=self.geometry())

    def scroll(self):
        command("xdotool", "mousemove", "--window", self.window_id, "450", "400")
        command("xdotool", "click", "--repeat", "20", "4")
        self.rendered("before-scroll", lambda text: "thank you for using katana" in text.lower())
        before = self.frame()
        command("xdotool", "click", "--repeat", "8", "5")
        until("UI completed after wheel input", lambda: self.frame() > before)
        self.rendered("after-scroll", lambda text: "anonymous crash reports" in text.lower()
                      and "by clicking" in text.lower())
        self.record("wheel-scroll-acknowledged")

    def popup(self):
        path = self.last_capture
        tsv = command("tesseract", str(path), "stdout", "--psm", "6", "tsv")
        words = list(csv.DictReader(io.StringIO(tsv), delimiter="\t"))
        word = next(row for row in words if row["text"].strip() == "English"
                    and int(row["top"]) < 300)
        x = int(word["left"]) + int(word["width"]) // 2
        y = int(word["top"]) + int(word["height"]) // 2
        before = self.frame()
        command("xdotool", "mousemove", "--window", self.window_id, str(x), str(y), "click", "1")
        until("UI completed after language popup", lambda: self.frame() > before)
        self.rendered("language-popup", lambda text: "Deutsch" in text)
        command("xdotool", "key", "--window", self.window_id, "Escape")
        self.rendered("popup-dismissed", lambda text: "Deutsch" not in text
                      and "terms" in text.lower())
        self.record("language-popup-opened-and-dismissed")

    def close(self):
        command("wmctrl", "-i", "-c", hex(int(self.window_id)))
        until("window closed through WM request", lambda: not self.find_window())
        settings = json.loads((self.root / "config" / "settings.json").read_text())
        assert settings.get("terms_accepted_version") is None, "Terms acceptance changed"
        self.record("wm-close-acknowledged", final_frame=self.frame())


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--evidence", required=True, type=Path)
    parser.add_argument("--expected-sha256", required=True)
    options = parser.parse_args()
    desktop = DesktopAcceptance(options.evidence.resolve(), options.expected_sha256)
    try:
        desktop.startup()
        desktop.resize()
        desktop.scroll()
        desktop.popup()
        desktop.close()
    except Exception as error:
        desktop.record("failed", error=repr(error), stderr=getattr(error, "stderr", None))
        raise


if __name__ == "__main__":
    main()
