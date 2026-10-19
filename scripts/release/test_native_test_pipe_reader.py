"""Exercise bounded pipe reading with real subprocesses."""

from __future__ import annotations

import subprocess
import sys
import unittest

from native_test_pipe_reader import read_pipes


MAX_BYTES = 1_048_576


class NativeTestPipeReaderTests(unittest.TestCase):
    def process(self, source: str) -> subprocess.Popen[bytes]:
        return subprocess.Popen([sys.executable, "-c", source], stdin=subprocess.PIPE,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    def test_reads_both_streams_and_preserves_nonzero_exit(self) -> None:
        process = self.process(
            "import sys; sys.stdout.buffer.write(b'out'); sys.stderr.buffer.write(b'err'); "
            "sys.stdout.flush(); sys.stderr.flush(); sys.exit(7)")
        try:
            status, stdout, stderr = read_pipes(process, MAX_BYTES, 2.0)
            self.assertEqual((status, stdout, stderr), (7, b"out", b"err"))
        finally:
            process.stdin.close()

    def test_captures_stderr_after_stdout_eof(self) -> None:
        process = self.process(
            "import sys; sys.stdout.write('head'); sys.stdout.close(); "
            "sys.stderr.write('tail'); sys.stderr.flush()")
        try:
            status, stdout, stderr = read_pipes(process, MAX_BYTES, 2.0)
            self.assertEqual((status, stdout, stderr), (0, b"head", b"tail"))
        finally:
            process.stdin.close()

    def test_accepts_exact_limit_and_detects_stderr_overflow(self) -> None:
        process = self.process(f"import sys; sys.stdout.buffer.write(b'x' * {MAX_BYTES})")
        try:
            status, stdout, stderr = read_pipes(process, MAX_BYTES, 2.0)
            self.assertEqual((status, len(stdout), stderr), (0, MAX_BYTES, b""))
        finally:
            process.stdin.close()
        process = self.process(f"import sys; sys.stderr.buffer.write(b'x' * {MAX_BYTES + 1})")
        try:
            with self.assertRaises(ValueError):
                read_pipes(process, MAX_BYTES, 2.0)
        finally:
            process.kill()
            process.wait()
            process.stdin.close()
            self.assertIsNotNone(process.poll())

    def test_stdout_eof_does_not_hide_open_stderr_and_timeout_reaps(self) -> None:
        process = self.process(
            "import sys; sys.stdout.write('done'); sys.stdout.close(); sys.stdin.buffer.read()")
        try:
            with self.assertRaises(TimeoutError):
                read_pipes(process, MAX_BYTES, 0.05)
        finally:
            process.kill()
            process.wait()
            process.stdin.close()
            self.assertIsNotNone(process.poll())


if __name__ == "__main__":
    unittest.main()
