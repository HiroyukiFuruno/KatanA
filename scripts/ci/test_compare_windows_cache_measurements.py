"""Reject incompatible observations without claiming reusable results."""
import copy
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

from compare_windows_cache_measurements import compare


def observations():
    key = "windows-target-v1-" + "a" * 64
    source = {"RUNNER_OS": "Windows", "RUNNER_ARCH": "X64", "GITHUB_SHA": "b" * 40,
              "GITHUB_RUN_ID": "123", "GITHUB_RUN_ATTEMPT": "1"}
    record = {"schema": 1, "executed_once": True, "exit_code": 0, "reuse_authorized": False,
              "compiled_target_cache": {"eligible": True, "key": key, "hit": False},
              "elapsed_seconds": 30.0, "source": source, "command_argv": ["bash", "test"],
              "cargo_lock_sha256": {"Cargo.lock": "c" * 64, "scripts/screenshot/Cargo.lock": "d" * 64},
              "lock_observation_phase": "after_command"}
    warm = copy.deepcopy(record)
    warm["compiled_target_cache"]["hit"] = True
    identity = {"schema": 1, "cache_eligible": True, "key": key,
                "environment_encoding": "sha256-per-value",
                "inputs": {"source": {"head": source["GITHUB_SHA"]},
                           "environment": {"RUSTFLAGS": "e" * 64}}}
    return [record, warm, *[copy.deepcopy(identity) for _ in range(4)]]


class MeasurementTests(unittest.TestCase):
    def test_matched_records_are_diagnostic_only(self):
        result = compare(*observations())
        self.assertFalse(result["reuse_authorized"])
        self.assertFalse(result["full_environment_equivalence_proven"])
        self.assertEqual(len(result["inputs_sha256"]), 64)

    def test_miss_mismatch_or_failed_invocation_is_rejected(self):
        for mutation in ("miss", "identity", "source", "lock", "failure", "unknown", "nan"):
            rows = observations()
            if mutation == "miss":
                rows[1]["compiled_target_cache"]["hit"] = False
            elif mutation == "identity":
                rows[4]["key"] = "windows-target-v1-" + "e" * 64
            elif mutation == "source":
                rows[1]["source"]["GITHUB_SHA"] = "e" * 40
            elif mutation == "lock":
                rows[1]["cargo_lock_sha256"]["Cargo.lock"] = "e" * 64
            elif mutation == "failure":
                rows[0]["exit_code"] = 1
            elif mutation == "unknown":
                rows[0]["compiled_target_cache"]["hit"] = None
            else:
                rows[0]["elapsed_seconds"] = float("nan")
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                compare(*rows)

    def test_real_workflow_shell_fails_when_comparison_fails(self):
        root = Path(__file__).resolve().parents[2]
        workflow = (root / ".github/workflows/windows-cache-measurement.yml").read_text()
        block = workflow.split("      - name: Compare matched measurements\n", 1)[1].split("\n      - ", 1)[0]
        self.assertIn("        shell: bash\n", block)
        shell = "\n".join(line[10:] for line in block.split("        run: |\n", 1)[1].splitlines())
        self.assertIn("set -o pipefail", shell)
        with tempfile.TemporaryDirectory() as directory:
            fixture = Path(directory)
            (fixture / "scripts").mkdir()
            (fixture / "scripts/ci").symlink_to(root / "scripts/ci", target_is_directory=True)
            (fixture / "cold").mkdir()
            (fixture / "warm").mkdir()
            for name in ("cold/cold.json", "cold/cold-before.json", "cold/cold-after.json",
                         "warm/warm.json", "warm/warm-before.json", "warm/warm-after.json"):
                (fixture / name).write_text("{}")
            result = subprocess.run(["bash", "--noprofile", "--norc", "-e", "-o", "pipefail", "-c", shell],
                                    cwd=fixture, env=os.environ | {"RUNNER_TEMP": directory},
                                    capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertTrue((fixture / "windows-cache-comparison.json").exists())


if __name__ == "__main__":
    unittest.main()
