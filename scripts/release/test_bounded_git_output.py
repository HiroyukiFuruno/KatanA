from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import time
import unittest

from bounded_git_output import (GitInvocationError, GitOutputLimitError, GitTimeoutError,
                                _CLEANUP_TIMEOUT, _run_bounded, bounded_git)


class BoundedGitOutputTests(unittest.TestCase):
    def _repo(self) -> Path:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        environment["GIT_GRAFT_FILE"] = os.devnull
        subprocess.run(["git", "-C", str(root), "init", "-q"], check=True, env=environment)
        return root

    def _terminate_pidfile(self, pidfile: Path) -> None:
        deadline = time.monotonic() + 1
        while not pidfile.exists() and time.monotonic() < deadline:
            time.sleep(.01)
        if pidfile.exists():
            pid = int(pidfile.read_text())
            try:
                os.kill(pid, 15)
            except ProcessLookupError:
                pass

    def test_normal_read_and_git_environment_isolation(self) -> None:
        root = self._repo()
        previous = os.environ.get("GIT_DIR")
        os.environ["GIT_DIR"] = "/invalid/git-dir"
        try:
            result = bounded_git(root, ["rev-parse", "--is-inside-work-tree"])
        finally:
            if previous is None:
                os.environ.pop("GIT_DIR", None)
            else:
                os.environ["GIT_DIR"] = previous
        self.assertEqual(result.strip(), b"true")

    def test_relative_root_is_resolved_once(self) -> None:
        root = self._repo()
        relative = Path(os.path.relpath(root, Path.cwd()))
        self.assertEqual(bounded_git(relative, ["rev-parse", "--is-inside-work-tree"]).strip(), b"true")

    def test_nonzero_git_is_typed_failure(self) -> None:
        with self.assertRaises(GitInvocationError):
            bounded_git(self._repo(), ["rev-parse", "--verify", "missing"])

    def test_stdout_limit_covers_stderr_too(self) -> None:
        command = [sys.executable, "-c", "import sys; sys.stderr.write('x' * 20)"]
        with self.assertRaises(GitOutputLimitError):
            _run_bounded(command, 8, 2)

    def test_stdout_below_and_at_limit_is_accepted(self) -> None:
        for size in (99, 100):
            command = [sys.executable, "-c", f"import sys; sys.stdout.write('x' * {size})"]
            self.assertEqual(len(_run_bounded(command, 100, 2)), size)

    def test_stdout_above_limit_is_rejected(self) -> None:
        command = [sys.executable, "-c", "import sys; sys.stdout.write('x' * 101)"]
        with self.assertRaises(GitOutputLimitError):
            _run_bounded(command, 100, 2)

    def test_silent_child_before_deadline_succeeds(self) -> None:
        command = [sys.executable, "-c", "import time; time.sleep(.2)"]
        self.assertEqual(_run_bounded(command, 1024, 2), b"")

    def test_eof_before_deadline_waits_for_child_exit(self) -> None:
        command = [sys.executable, "-c", "import os,time; os.close(1); os.close(2); time.sleep(1.2)"]
        self.assertEqual(_run_bounded(command, 100, 3), b"")

    def test_eof_child_exceeding_deadline_is_rejected(self) -> None:
        command = [sys.executable, "-c", "import os,time; os.close(1); os.close(2); time.sleep(1.2)"]
        with self.assertRaises(GitTimeoutError):
            _run_bounded(command, 100, .2)

    def test_finite_descendant_pipe_inheritance_is_cleaned(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        pidfile = Path(temporary.name) / "descendant.pid"
        self.addCleanup(self._terminate_pidfile, pidfile)
        command = [sys.executable, "-c",
                   "import pathlib,subprocess,sys; child=subprocess.Popen([sys.executable,'-c',"
                   "'import time; time.sleep(4)']); pathlib.Path(sys.argv[1]).write_text(str(child.pid))",
                   str(pidfile)]
        before = {thread.ident for thread in threading.enumerate()}
        started = time.monotonic()
        with self.assertRaises(GitTimeoutError):
            _run_bounded(command, 100, .1)
        elapsed = time.monotonic() - started
        self.assertLess(elapsed, .1 + (2 * _CLEANUP_TIMEOUT) + .5)
        after = {thread.ident for thread in threading.enumerate()}
        self.assertEqual(after - before, set())

    def test_timeout_kills_real_child(self) -> None:
        command = [sys.executable, "-c", "import time; time.sleep(2)"]
        with self.assertRaises(GitTimeoutError):
            _run_bounded(command, 1024, 0.05)

    def test_invalid_limits_are_rejected(self) -> None:
        root = self._repo()
        with self.assertRaises(GitInvocationError):
            bounded_git(root, ["status"], stdout_limit=0)
        with self.assertRaises(GitInvocationError):
            bounded_git(root, ["status"], timeout=0)
        with self.assertRaises(GitInvocationError):
            bounded_git(root, ["status"], timeout=float("inf"))
        with self.assertRaises(GitInvocationError):
            bounded_git(root, ["status"], timeout=True)
        with self.assertRaises(GitInvocationError):
            bounded_git(root, ("status",))
        with self.assertRaises(GitInvocationError):
            bounded_git(root, [None])


if __name__ == "__main__":
    unittest.main()
