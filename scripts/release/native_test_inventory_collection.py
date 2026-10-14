"""Bounded native libtest --list collection; no reuse or CI claims."""

from __future__ import annotations

from collections.abc import Sequence
import hashlib
import os
from pathlib import Path
import selectors
import shutil
import subprocess
import time

from native_test_inventory import InventoryFormatError, parse_inventory
from native_test_inventory_record import InventoryConditions, InventoryRecord


MAX_STREAM_BYTES = 1_048_576
TIMEOUT_SECONDS = 30


class InventoryExecutionError(RuntimeError):
    """The selected native listing command did not produce a valid inventory."""


def _resolve_command(argv: Sequence[str], cwd: Path, kind: str) -> list[str]:
    if kind not in {"test", "coverage"} or not argv:
        raise InventoryExecutionError("inventory kind or argv is invalid")
    if "--list" not in argv[1:]:
        raise InventoryExecutionError("native command must request --list")
    requested = Path(argv[0])
    local_candidate = cwd / requested
    if not requested.is_absolute() and local_candidate.exists():
        executable = str(local_candidate)
    elif not requested.is_absolute() and (
        os.sep in argv[0] or (os.altsep is not None and os.altsep in argv[0])
    ):
        executable = str((cwd / requested).resolve())
    else:
        executable = shutil.which(argv[0])
    if executable is None or not Path(executable).resolve().is_file():
        raise InventoryExecutionError("native command executable is not a regular file")
    if not cwd.is_dir():
        raise InventoryExecutionError("native command cwd is not a directory")
    return [str(Path(executable).resolve()), *argv[1:]]


def _stop(process: subprocess.Popen[bytes]) -> None:
    if process.poll() is None:
        process.kill()
    try:
        process.wait(timeout=1)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()


def _consume_event(key: selectors.SelectorKey, selector: selectors.BaseSelector,
                   streams: set[object], buffers: dict[object, bytearray]) -> None:
    chunk = os.read(key.fileobj.fileno(), 65536)
    if not chunk:
        selector.unregister(key.fileobj)
        streams.remove(key.fileobj)
        return
    buffers[key.fileobj].extend(chunk)
    if len(buffers[key.fileobj]) > MAX_STREAM_BYTES:
        raise InventoryExecutionError("native --list output exceeds the stream limit")


def _read_streams(process: subprocess.Popen[bytes]) -> tuple[int, bytes, bytes]:
    buffers = {process.stdout: bytearray(), process.stderr: bytearray()}
    streams = set(buffers)
    selector = selectors.DefaultSelector()
    try:
        try:
            for stream in streams:
                selector.register(stream, selectors.EVENT_READ)
        except (OSError, ValueError) as error:
            raise InventoryExecutionError("native --list streams could not be registered") from error
        deadline = time.monotonic() + TIMEOUT_SECONDS
        while streams:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise InventoryExecutionError("native --list command timed out")
            events = selector.select(remaining)
            if not events:
                raise InventoryExecutionError("native --list command timed out")
            for key, _ in events:
                _consume_event(key, selector, streams, buffers)
        return process.wait(timeout=1), bytes(buffers[process.stdout]), bytes(buffers[process.stderr])
    except (OSError, subprocess.TimeoutExpired) as error:
        raise InventoryExecutionError("native --list output could not be collected safely") from error
    finally:
        selector.close()
        for stream in (process.stdout, process.stderr):
            if stream is not None:
                stream.close()


def collect_inventory(argv: Sequence[str], cwd: Path, kind: str,
                      conditions: InventoryConditions) -> InventoryRecord:
    if os.name != "posix":
        raise InventoryExecutionError("native --list collector requires POSIX selector pipes")
    command = _resolve_command(argv, cwd, kind)
    try:
        process = subprocess.Popen(command, cwd=cwd, stdin=subprocess.DEVNULL,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    except OSError as error:
        raise InventoryExecutionError("native --list command could not start") from error
    try:
        return_code, stdout, _ = _read_streams(process)
    except InventoryExecutionError:
        _stop(process)
        raise
    if return_code != 0:
        raise InventoryExecutionError(f"native --list command failed: {return_code}")
    try:
        inventory = parse_inventory(stdout.decode("utf-8"))
    except (UnicodeError, InventoryFormatError) as error:
        raise InventoryExecutionError("native --list output is not a verified inventory") from error
    return InventoryRecord(kind, stdout, hashlib.sha256(stdout).hexdigest(),
                           tuple(command), inventory, conditions)
