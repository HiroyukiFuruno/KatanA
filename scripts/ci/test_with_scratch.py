"""Exercise scratch cleanup with real processes and open files."""

import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

WRAPPER = Path(__file__).with_name("with-scratch.py")
WRITE_SCRATCH = (
    "import os, pathlib, sys;"
    "p = pathlib.Path(os.environ['KATANA_SCRATCH_DIR']);"
    "(p / 'generated').write_text('temporary');"
    "print(p, flush=True);"
)


@unittest.skipUnless(os.name == "posix", "POSIX process-group lifetime")
class ScratchTest(unittest.TestCase):
    def command(self, source, *args):
        return [sys.executable, str(WRAPPER), "--", sys.executable, "-c", source, *args]

    def test_cleanup_after_success_and_failure_preserves_external_output(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "result with spaces.txt"
            for status in (0, 7):
                with self.subTest(status=status):
                    source = WRITE_SCRATCH + (
                        "pathlib.Path(sys.argv[1]).write_text('result'); sys.exit(int(sys.argv[2]))"
                    )
                    result = subprocess.run(
                        self.command(source, str(output), str(status)), capture_output=True,
                        text=True, timeout=15,
                    )
                    self.assertEqual(status, result.returncode, result.stderr)
                    self.assertFalse(Path(result.stdout.strip()).exists())
                    self.assertEqual("result", output.read_text())

    def test_descendant_uses_scratch_after_parent_exits(self):
        descendant = (
            "import os, sys, time\n"
            "while os.getppid() == int(sys.argv[1]):\n"
            "    time.sleep(0.01)\n"
            + WRITE_SCRATCH
            + "sys.stdin.readline(); print((p / 'generated').read_text(), flush=True)"
        )
        parent = (
            "import os, subprocess, sys;"
            "subprocess.Popen([sys.executable, '-c', sys.argv[1], str(os.getpid())])"
        )
        result = subprocess.run(
            self.command(parent, descendant), input="continue\n",
            capture_output=True, text=True, timeout=15,
        )
        self.assertEqual(0, result.returncode, result.stderr)
        directory, contents = result.stdout.splitlines()
        self.assertEqual("temporary", contents)
        self.assertFalse(Path(directory).exists())

    def test_unrelated_process_open_file_prevents_cleanup(self):
        with tempfile.TemporaryDirectory() as parent:
            env = os.environ.copy() | {"TMPDIR": parent}
            with subprocess.Popen(
                self.command(WRITE_SCRATCH + "sys.stdin.readline()"), env=env,
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
            ) as process:
                directory = Path(process.stdout.readline().strip())
                with (directory / "generated").open() as stream:
                    _, error = process.communicate(input="continue\n", timeout=15)
                    self.assertEqual(1, process.returncode, error)
                    self.assertIn("may still be in use", error)
                    self.assertEqual("temporary", stream.read())
                    self.assertTrue(directory.is_dir())

    @unittest.skipIf(sys.platform == "darwin", "macOS uses the system lsof path")
    def test_missing_lsof_retains_scratch(self):
        with tempfile.TemporaryDirectory() as parent:
            tools = Path(parent, "bin")
            tools.mkdir()
            Path(tools, "ps").symlink_to(shutil.which("ps"))
            result = subprocess.run(
                self.command(WRITE_SCRATCH), env=os.environ | {"TMPDIR": parent, "PATH": str(tools)},
                capture_output=True, text=True, timeout=15,
            )
            self.assertEqual(1, result.returncode, result.stderr)
            self.assertIn("lsof is required", result.stderr)
            self.assertTrue(Path(result.stdout.strip()).is_dir())


if __name__ == "__main__":
    unittest.main()
