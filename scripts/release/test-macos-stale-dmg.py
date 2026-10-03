#!/usr/bin/env python3
"""Run the macOS stale-DMG workflow step against cold and warm checkouts."""

from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/build-and-release.yml"
STEP_NAME = "      - name: Clean stale DMG files"


def extract_step_script(workflow: str) -> str:
    lines = workflow.splitlines()
    matches = [index for index, line in enumerate(lines) if line == STEP_NAME]
    if len(matches) != 1:
        raise AssertionError(
            f"expected exactly one {STEP_NAME.strip()!r} step, found {len(matches)}"
        )

    step_start = matches[0]
    step_end = next(
        (
            index
            for index in range(step_start + 1, len(lines))
            if lines[index].startswith("      - ")
        ),
        len(lines),
    )
    return extract_run_block(lines[step_start + 1 : step_end])


def extract_run_block(lines: list[str]) -> str:
    run_lines = [
        index
        for index, line in enumerate(lines)
        if line == "        run: |"
    ]
    if len(run_lines) != 1:
        raise AssertionError(
            f"expected one literal run block in {STEP_NAME.strip()!r}, found {len(run_lines)}"
        )

    script_lines: list[str] = []
    for line in lines[run_lines[0] + 1 :]:
        if line and len(line) - len(line.lstrip()) < 10:
            break
        script_lines.append(line[10:] if line else "")
    script = "\n".join(script_lines).rstrip()
    if not script:
        raise AssertionError(f"{STEP_NAME.strip()!r} run block is empty")
    return script


class StaleDmgCleanupTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.script = extract_step_script(WORKFLOW.read_text(encoding="utf-8"))

    def run_step(self, checkout: Path, runner_temp: Path) -> subprocess.CompletedProcess[str]:
        environment = os.environ.copy()
        environment["RUNNER_TEMP"] = str(runner_temp)
        return subprocess.run(
            ["bash", "-e", "-u", "-o", "pipefail", "-c", self.script],
            cwd=checkout,
            env=environment,
            capture_output=True,
            text=True,
            check=False,
        )

    def test_cold_checkout_without_release_directory_succeeds(self) -> None:
        with tempfile.TemporaryDirectory() as temp_directory:
            root = Path(temp_directory)
            checkout = root / "checkout"
            runner_temp = root / "runner-temp"
            checkout.mkdir()
            runner_temp.mkdir()

            self.assertFalse((checkout / "target/release").exists())
            result = self.run_step(checkout, runner_temp)

            self.assertEqual(0, result.returncode, result.stderr)
            self.assertTrue(
                (runner_temp / "katana-stale-release-assets").is_dir()
            )

    def test_warm_checkout_moves_only_matching_top_level_dmgs(self) -> None:
        with tempfile.TemporaryDirectory() as temp_directory:
            root = Path(temp_directory)
            checkout = root / "checkout"
            release = checkout / "target/release"
            nested = release / "nested"
            runner_temp = root / "runner-temp"
            nested.mkdir(parents=True)
            runner_temp.mkdir()

            stale = release / "KatanA-Desktop-0.22.41.dmg"
            unrelated = release / "Other-Desktop-0.22.41.dmg"
            nested_stale = nested / "KatanA-Desktop-nested.dmg"
            for path in (stale, unrelated, nested_stale):
                path.write_text(path.name, encoding="utf-8")
            stale_contents = stale.read_text(encoding="utf-8")

            result = self.run_step(checkout, runner_temp)

            self.assertEqual(0, result.returncode, result.stderr)
            stale_directory = runner_temp / "katana-stale-release-assets"
            self.assertEqual(
                stale_contents,
                (stale_directory / stale.name).read_text(encoding="utf-8"),
            )
            self.assertFalse(stale.exists())
            self.assertTrue(unrelated.is_file())
            self.assertTrue(nested_stale.is_file())


if __name__ == "__main__":
    unittest.main()
