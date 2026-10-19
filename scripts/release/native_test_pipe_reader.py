"""Read bounded subprocess pipes without threads or temporary spools."""

from __future__ import annotations

import os
import subprocess
import time


READ_CHUNK_BYTES = 65_536
POLL_INTERVAL_SECONDS = 0.01


def _read_stream(stream, buffer: bytearray, max_stream_bytes: int) -> tuple[bool, bool]:
    try:
        chunk = os.read(stream.fileno(), READ_CHUNK_BYTES)
    except BlockingIOError:
        return False, False
    if not chunk:
        return True, False
    buffer.extend(chunk)
    if len(buffer) > max_stream_bytes:
        raise ValueError("native --list output exceeds the stream limit")
    return False, True


def _drain_streams(streams, buffers: list[bytearray], max_stream_bytes: int,
                   deadline: float) -> None:
    open_streams = [True, True]
    while any(open_streams):
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError("native --list command timed out")
        progress = False
        for index, stream in enumerate(streams):
            if not open_streams[index]:
                continue
            eof, read_progress = _read_stream(stream, buffers[index], max_stream_bytes)
            if eof:
                open_streams[index] = False
            if read_progress:
                progress = True
        if not progress and any(open_streams):
            time.sleep(min(POLL_INTERVAL_SECONDS, remaining))


def _wait_for_process(process: subprocess.Popen[bytes], deadline: float) -> int:
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise TimeoutError("native --list command timed out")
    try:
        return process.wait(timeout=remaining)
    except subprocess.TimeoutExpired as error:
        raise TimeoutError("native --list command timed out") from error


def read_pipes(process: subprocess.Popen[bytes], max_stream_bytes: int,
               timeout_seconds: float) -> tuple[int, bytes, bytes]:
    streams = (process.stdout, process.stderr)
    if (any(stream is None for stream in streams) or max_stream_bytes <= 0
            or timeout_seconds <= 0):
        raise ValueError("bounded pipe reader arguments are invalid")
    buffers = [bytearray(), bytearray()]
    try:
        for stream in streams:
            os.set_blocking(stream.fileno(), False)
        deadline = time.monotonic() + timeout_seconds
        _drain_streams(streams, buffers, max_stream_bytes, deadline)
        return _wait_for_process(process, deadline), bytes(buffers[0]), bytes(buffers[1])
    finally:
        for stream in streams:
            stream.close()
