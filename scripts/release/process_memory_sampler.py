#!/usr/bin/env python3
"""Read RSS and, where supported, the physical footprint of an external PID."""

from __future__ import annotations

import argparse
import ctypes
import ctypes.util
import json
import os
import platform
import re
import struct
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

DARWIN_RUSAGE_INFO_V4 = 4
DARWIN_RESIDENT_SIZE_OFFSET = 64
DARWIN_PHYSICAL_FOOTPRINT_OFFSET = 72
DARWIN_START_ABSTIME_OFFSET = 80
DARWIN_PROC_PATH_MAX = 4096
POSIX_PID_MAX = 2**31 - 1


def now() -> str:
    return datetime.now(timezone.utc).isoformat()

def validate_pid(pid: int) -> None:
    if not 1 <= pid <= POSIX_PID_MAX:
        raise ValueError(f"pid must be in POSIX signed-32 range: {pid}")

def _base(pid: int) -> dict[str, Any]:
    return {
        "schema_version": 1,
        "sampler": "external_process_memory_v2",
        "pid": pid,
        "observed_at": now(),
        "identity": None,
        "rss_bytes": None,
        "rss_source": None,
        "physical_footprint_bytes": None,
        "physical_footprint_source": None,
        "status": "error",
        "error": None,
    }

def _linux_starttime(pid: int) -> str:
    text = Path(f"/proc/{pid}/stat").read_text(encoding="ascii")
    closing = text.rfind(")")
    if closing < 0:
        raise ValueError("/proc stat has no process-name terminator")
    fields = text[closing + 2 :].split()
    if len(fields) <= 19 or not fields[19].isdigit():
        raise ValueError("/proc stat has no valid starttime")
    # WHY: starttime（field 22）で実行ファイル全体を毎回ハッシュせずPID再利用を区別する。
    return fields[19]

def _darwin_library() -> ctypes.CDLL:
    return ctypes.CDLL(ctypes.util.find_library("proc") or "/usr/lib/libproc.dylib", use_errno=True)

def _darwin_identity(pid: int) -> dict[str, str]:
    library = _darwin_library()
    proc_pidpath = library.proc_pidpath
    proc_pidpath.argtypes = [ctypes.c_int, ctypes.c_void_p, ctypes.c_uint32]
    proc_pidpath.restype = ctypes.c_int
    path_buffer = ctypes.create_string_buffer(DARWIN_PROC_PATH_MAX)
    path_length = proc_pidpath(pid, path_buffer, DARWIN_PROC_PATH_MAX)
    if path_length <= 0:
        error_number = ctypes.get_errno()
        raise OSError(error_number, os.strerror(error_number))
    executable = path_buffer.raw[:path_length].decode("utf-8", errors="strict")
    rusage = library.proc_pid_rusage
    rusage.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_void_p]
    rusage.restype = ctypes.c_int
    buffer = ctypes.create_string_buffer(1024)
    if rusage(pid, DARWIN_RUSAGE_INFO_V4, ctypes.cast(buffer, ctypes.c_void_p)) != 0:
        error_number = ctypes.get_errno()
        raise OSError(error_number, os.strerror(error_number))
    start_abstime = struct.unpack_from("<Q", buffer.raw, DARWIN_START_ABSTIME_OFFSET)[0]
    if not start_abstime:
        raise ValueError("proc_pid_rusage returned no process start time")
    return {"executable": executable, "start_identity": str(start_abstime)}

def process_identity(pid: int, system: str | None = None) -> dict[str, str]:
    validate_pid(pid)
    current_system = system or platform.system()
    if current_system == "Linux":
        executable = os.readlink(f"/proc/{pid}/exe")
        return {"executable": executable, "start_identity": _linux_starttime(pid)}
    if current_system == "Darwin":
        return _darwin_identity(pid)
    raise RuntimeError(f"unsupported platform: {current_system}")

def _identity_mismatch(expected: dict[str, str] | None, actual: dict[str, str]) -> str | None:
    if expected is None:
        return None
    for key in ("start_identity", "executable"):
        value = expected.get(key)
        if value is not None and value != actual.get(key):
            return f"process identity mismatch for {key}"
    return None

def _darwin_memory(pid: int) -> tuple[int, int]:
    library = _darwin_library()
    proc_pid_rusage = library.proc_pid_rusage
    proc_pid_rusage.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_void_p]
    proc_pid_rusage.restype = ctypes.c_int
    buffer = ctypes.create_string_buffer(1024)
    if proc_pid_rusage(pid, DARWIN_RUSAGE_INFO_V4, ctypes.cast(buffer, ctypes.c_void_p)) != 0:
        error_number = ctypes.get_errno()
        raise OSError(error_number, os.strerror(error_number))
    resident = struct.unpack_from("<Q", buffer.raw, DARWIN_RESIDENT_SIZE_OFFSET)[0]
    physical = struct.unpack_from("<Q", buffer.raw, DARWIN_PHYSICAL_FOOTPRINT_OFFSET)[0]
    if resident <= 0 or physical <= 0:
        raise ValueError("proc_pid_rusage returned non-positive memory values")
    return resident, physical

def _linux_memory(pid: int) -> int:
    text = Path(f"/proc/{pid}/status").read_text(encoding="ascii")
    match = re.search(r"^VmRSS:\s+(\d+)\s+kB$", text, re.MULTILINE)
    if match is None:
        raise ValueError("/proc status has no VmRSS field")
    resident = int(match.group(1)) * 1024
    if resident <= 0:
        raise ValueError("/proc VmRSS is non-positive")
    return resident

def _read_memory(result: dict[str, Any], pid: int, system: str) -> None:
    if system == "Darwin":
        resident, physical = _darwin_memory(pid)
        result.update(rss_bytes=resident, rss_source="proc_pid_rusage:RUSAGE_INFO_V4.ri_resident_size",
                      physical_footprint_bytes=physical,
                      physical_footprint_source="proc_pid_rusage:RUSAGE_INFO_V4.ri_phys_footprint")
    else:
        result.update(rss_bytes=_linux_memory(pid), rss_source=f"/proc/{pid}/status:VmRSS",
                      physical_footprint_source="unsupported on Linux sampler")

def sample_for_platform(pid: int, system: str, expected_identity: dict[str, str] | None = None) -> dict[str, Any]:
    validate_pid(pid)
    result = _base(pid)
    if system not in ("Darwin", "Linux"):
        result.update(status="unsupported", error=f"unsupported platform: {system}")
        return result
    try:
        before = process_identity(pid, system)
        result["identity"] = before
        mismatch = _identity_mismatch(expected_identity, before)
        if mismatch:
            raise RuntimeError(mismatch)
        _read_memory(result, pid, system)
        after = process_identity(pid, system)
        if after != before:
            raise RuntimeError("process identity changed during memory sample")
        result["status"] = "ok"
    except (OSError, RuntimeError, ValueError, UnicodeError, ctypes.ArgumentError) as error:
        # WHY: 識別に失敗した値は不採用とし、部分的な古い値を公開しない。
        result["rss_bytes"] = None
        result["rss_source"] = None
        result["physical_footprint_bytes"] = None
        result["physical_footprint_source"] = None
        result["error"] = f"{type(error).__name__}: {error}"
    return result

def sample(pid: int, expected_identity: dict[str, str] | None = None) -> dict[str, Any]:
    return sample_for_platform(pid, platform.system(), expected_identity)

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pid", type=int)
    parser.add_argument("output", type=Path, nargs="?")
    parser.add_argument("--expected-start-identity")
    parser.add_argument("--expected-executable")
    args = parser.parse_args()
    expected = {
        key: value
        for key, value in (
            ("start_identity", args.expected_start_identity),
            ("executable", args.expected_executable),
        )
        if value is not None
    }
    try:
        validate_pid(args.pid)
        result = sample(args.pid, expected or None)
    except ValueError as error:
        parser.error(str(error))
    encoded = json.dumps(result, sort_keys=True) + "\n"
    if args.output is not None:
        try:
            args.output.write_text(encoded, encoding="utf-8")
        except OSError as error:
            print(f"FAIL: sampler output write failed: {error}", file=sys.stderr)
            return 2
    print(encoded, end="")
    return 0 if result["status"] == "ok" else 2

if __name__ == "__main__":
    raise SystemExit(main())
