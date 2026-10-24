#!/usr/bin/env python3
import importlib.util
import json
import os
import platform
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch


SAMPLER = Path(__file__).with_name("process_memory_sampler.py")
SPEC = importlib.util.spec_from_file_location("process_memory_sampler", SAMPLER)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class ProcessMemorySamplerTests(unittest.TestCase):
    def _run_cli(self, *arguments):
        return subprocess.run(
            [sys.executable, str(SAMPLER), *arguments],
            check=False, capture_output=True, text=True,
        )

    def test_external_sampler_binds_identity_and_keeps_physical_field_separate(self):
        if platform.system() not in ("Darwin", "Linux"):
            completed = self._run_cli(str(os.getpid()))
            self.assertEqual(completed.returncode, 2, completed.stderr)
            result = json.loads(completed.stdout)
            self.assertEqual(result["status"], "unsupported")
            self.assertIsNone(result["rss_bytes"])
            self.assertIsNone(result["physical_footprint_bytes"])
            return
        identity = MODULE.process_identity(os.getpid())
        completed = self._run_cli(
            str(os.getpid()), "--expected-start-identity", identity["start_identity"],
            "--expected-executable", identity["executable"],
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        result = json.loads(completed.stdout)
        self.assertEqual(result["status"], "ok")
        self.assertGreater(result["rss_bytes"], 0)
        self.assertEqual(result["identity"], identity)
        self._assert_supported_memory(result)

    def _assert_supported_memory(self, result):
        if platform.system() == "Darwin":
            self.assertGreater(result["physical_footprint_bytes"], 0)
            self.assertIn("ri_phys_footprint", result["physical_footprint_source"])
        elif platform.system() == "Linux":
            self.assertIsNone(result["physical_footprint_bytes"])
            self.assertEqual(result["physical_footprint_source"], "unsupported on Linux sampler")

    def test_identity_mismatch_rejects_sample(self):
        identity = {"start_identity": "current-process", "executable": "/bin/current"}
        expected = dict(identity, start_identity="not-the-current-process")
        with patch.object(MODULE, "process_identity", return_value=identity):
            result = MODULE.sample_for_platform(123, "Linux", expected)
        self.assertEqual(result["status"], "error")
        self.assertIsNone(result["rss_bytes"])
        self.assertIn("identity mismatch", result["error"])

    def test_identity_change_after_memory_read_rejects_partial_sample(self):
        before = {"start_identity": "one", "executable": "/bin/first"}
        after = {"start_identity": "two", "executable": "/bin/second"}

        def populated(result, _pid, _system):
            result.update(rss_bytes=7, rss_source="test:rss",
                          physical_footprint_bytes=11, physical_footprint_source="test:physical")

        with patch.object(MODULE, "process_identity", side_effect=[before, after]), patch.object(
            MODULE, "_read_memory", side_effect=populated
        ):
            result = MODULE.sample_for_platform(123, "Linux")
        self.assertEqual(result["status"], "error")
        self.assertIsNone(result["rss_bytes"])
        self.assertIsNone(result["rss_source"])
        self.assertIsNone(result["physical_footprint_bytes"])
        self.assertIsNone(result["physical_footprint_source"])
        self.assertIn("identity changed", result["error"])

    def test_invalid_pid_does_not_sample(self):
        completed = self._run_cli("0")
        self.assertNotEqual(completed.returncode, 0)

    def test_unsupported_platform_keeps_memory_fields_missing(self):
        for system in ("Windows", "TestOS"):
            with self.subTest(system=system):
                result = MODULE.sample_for_platform(os.getpid(), system)
                self.assertEqual(result["status"], "unsupported")
                self.assertIsNone(result["rss_bytes"])
                self.assertIsNone(result["physical_footprint_bytes"])

    def test_pid_above_posix_signed_32_bit_range_is_rejected(self):
        with self.assertRaises(ValueError):
            MODULE.validate_pid(2**32 + 1)


if __name__ == "__main__":
    unittest.main()
