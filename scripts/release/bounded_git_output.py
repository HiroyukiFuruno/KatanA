"""Run Git reads with bounded pipes and a hard deadline."""

from __future__ import annotations

import math
import os
from pathlib import Path
import subprocess
import time

from downstream_git_binding import GitBindingError

_CLEANUP_TIMEOUT = 1.0


class GitOutputLimitError(GitBindingError):
    """A Git child exceeded an output bound."""


class GitTimeoutError(GitBindingError):
    """A Git child did not finish before its deadline."""


class GitInvocationError(GitBindingError):
    """The bounded Git invocation was invalid or failed."""


def _validate_limits(stdout_limit: int, timeout: float) -> None:
    if isinstance(stdout_limit, bool) or not isinstance(stdout_limit, int) or stdout_limit <= 0:
        raise GitInvocationError("stdout_limit must be a positive integer")
    if isinstance(timeout, bool) or not isinstance(timeout, (int, float)) \
            or not math.isfinite(timeout) or timeout <= 0:
        raise GitInvocationError("timeout must be a positive finite number")


def _kill_process(process: subprocess.Popen[bytes]) -> None:
    if process.poll() is None:
        process.kill()


def _drain(fd: int, output: bytearray, limit: int,
           deadline: float) -> tuple[bool, bool]:
    while True:
        if time.monotonic() >= deadline:
            raise GitTimeoutError("Git process exceeded deadline")
        try:
            chunk = os.read(fd, 65536)
        except BlockingIOError:
            return False, False
        except OSError as error:
            raise GitInvocationError("Git output could not be read") from error
        if not chunk:
            return True, False
        before = len(output)
        if before < limit:
            output.extend(chunk[:limit - before])
        if before + len(chunk) > limit:
            return False, True


def _watch_process(process: subprocess.Popen[bytes], streams: dict[int, bytearray],
                   limit: int, timeout: float) -> GitBindingError | None:
    deadline = time.monotonic() + timeout
    while streams or process.poll() is None:
        for fd, output in list(streams.items()):
            eof, exceeded = _drain(fd, output, limit, deadline)
            if exceeded:
                return GitOutputLimitError("Git output exceeded stdout/stderr limit")
            if eof:
                streams.pop(fd)
        if time.monotonic() >= deadline:
            return GitTimeoutError("Git process exceeded deadline")
        time.sleep(min(0.01, max(0.0, deadline - time.monotonic())))
    return None


def _cleanup_process(process: subprocess.Popen[bytes]) -> None:
    try:
        process.wait(timeout=_CLEANUP_TIMEOUT)
    except subprocess.TimeoutExpired:
        _kill_process(process)
        process.wait(timeout=_CLEANUP_TIMEOUT)
    for stream in (process.stdout, process.stderr):
        stream.close()


def _open_streams(process: subprocess.Popen[bytes]) -> tuple[dict[int, bytearray], bytearray]:
    output = bytearray()
    errors = bytearray()
    streams = {process.stdout.fileno(): output, process.stderr.fileno(): errors}
    try:
        for fd in streams:
            os.set_blocking(fd, False)
    except (AttributeError, OSError) as error:
        raise GitInvocationError("Git output does not support nonblocking reads") from error
    return streams, output


def _finish_process(process: subprocess.Popen[bytes],
                    failure: GitBindingError | None) -> GitBindingError | None:
    if failure is not None:
        _kill_process(process)
    try:
        _cleanup_process(process)
    except (OSError, subprocess.TimeoutExpired) as error:
        cleanup_failure = GitTimeoutError("Git process cleanup exceeded deadline")
        cleanup_failure.__cause__ = error
        return failure or cleanup_failure
    return failure


def _run_bounded(command: list[str], stdout_limit: int, timeout: float,
                 cwd: Path | None = None) -> bytes:
    _validate_limits(stdout_limit, timeout)
    environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
    environment["GIT_GRAFT_FILE"] = os.devnull
    try:
        process = subprocess.Popen(command, cwd=cwd, env=environment, stdin=subprocess.DEVNULL,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
    except (OSError, ValueError, TypeError) as error:
        raise GitInvocationError("Git process could not be started") from error
    failure: GitBindingError | None = None
    try:
        streams, output = _open_streams(process)
        failure = _watch_process(process, streams, stdout_limit, timeout)
    except GitBindingError as error:
        failure = error
    finally:
        failure = _finish_process(process, failure)
    if failure is not None:
        raise failure
    if process.returncode:
        raise GitInvocationError("Git process returned a non-zero status")
    return bytes(output)


def bounded_git(root: Path, args: list[str], *, stdout_limit: int = 2_097_152,
                timeout: float = 30) -> bytes:
    """Run a Git read without inheriting Git replacement or fetch settings."""
    if not isinstance(root, Path) or not isinstance(args, list) \
            or not all(isinstance(arg, str) for arg in args):
        raise GitInvocationError("Git root and arguments have invalid types")
    resolved_root = root.resolve()
    command = ["git", "--no-lazy-fetch", "--no-replace-objects", "--literal-pathspecs",
               "-C", str(resolved_root), *args]
    return _run_bounded(command, stdout_limit, timeout, resolved_root)
