#!/usr/bin/env python3
"""HTML host runnerの所有プロセス群を記録し、安全に残存確認する。

CLI:
  launch RECEIPT OBSERVATIONS -- COMMAND [ARG ...]  # 同一PIDでrunnerへexec
  observe RECEIPT OBSERVATIONS                      # 現在の実プロセスを記録
  finish RECEIPT OBSERVATIONS                       # 残存を失敗扱いにして限定終了

finishは残存があれば終了処理後も非0を返す。専用プロセスグループから
離脱した子孫は対象外。PID、PGID、カーネル起動時刻を再照合できない場合は
signalせず失敗する。Darwinのps lstartは秒精度のため、同一秒内のPID再利用を
完全には識別できない。Linuxでは/procのstarttime tick値を使う。
"""

import argparse
import json
import os
import platform
import re
import signal
import subprocess
import sys
import time
from pathlib import Path


class InspectionError(RuntimeError):
    pass


def _validate_launch_parent(expected_parent_pid: int, actual_parent_pid: int) -> None:
    if actual_parent_pid != expected_parent_pid:
        raise InspectionError("caller is not the launch parent")


def _start_time(pid: int) -> str:
    if sys.platform.startswith("linux"):
        try:
            raw = Path(f"/proc/{pid}/stat").read_text()
            # commは空白や括弧を含められるため、最後の") "からstat項目を数える。
            fields = raw[raw.rfind(") ") + 2 :].split()
            value = fields[19]  # proc stat field 22: starttime
            if not value.isdecimal():
                raise ValueError("invalid proc starttime")
            return value
        except FileNotFoundError as error:
            raise InspectionError(f"process {pid} disappeared during identity lookup") from error
        except (OSError, IndexError, ValueError) as error:
            raise InspectionError(f"cannot read kernel starttime for pid {pid}") from error
    if platform.system() == "Darwin":
        try:
            result = subprocess.run(
                ["ps", "-p", str(pid), "-o", "lstart="],
                check=True, capture_output=True, text=True,
            )
        except (OSError, subprocess.CalledProcessError) as error:
            raise InspectionError(f"cannot read kernel starttime for pid {pid}") from error
        value = result.stdout.strip()
        if not value:
            raise InspectionError(f"missing kernel starttime for pid {pid}")
        return value
    raise InspectionError(f"unsupported platform: {platform.system()}")


def _ps_snapshot(pgid: int | None = None) -> list[dict]:
    if platform.system() not in {"Darwin", "Linux"}:
        raise InspectionError(f"unsupported platform: {platform.system()}")
    if sys.platform.startswith("linux") and not Path("/proc").is_dir():
        raise InspectionError("Linux /proc is unavailable for kernel process identity checks")
    try:
        result = subprocess.run(
            ["ps", "-axo", "pid=,pgid=,stat=,%cpu=,rss=,lstart="],
            check=True, capture_output=True, text=True,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        raise InspectionError("ps process inventory failed") from error
    processes = []
    for line in result.stdout.splitlines():
        match = re.match(r"^\s*(\d+)\s+(\d+)\s+(\S+)\s+([\d.]+)\s+(\d+)\s+(.+?)\s*$", line)
        if not match:
            raise InspectionError(f"unparseable ps row: {line!r}")
        pid_text, pgid_text, state, cpu_text, rss_text, ps_start = match.groups()
        try:
            pid = int(pid_text)
            row_pgid = int(pgid_text)
        except ValueError as error:
            raise InspectionError(f"invalid ps identity row: {line!r}") from error
        if pgid is not None and row_pgid != pgid:
            continue
        # Darwinのps行自体にカーネル起動時刻が含まれる。別コマンドでPIDを
        # 再照合すると短命なプロセスが消える競合が起きる。
        if platform.system() == "Darwin":
            start = ps_start
        else:
            try:
                start = _start_time(pid)
            except InspectionError:
                proc_path = Path(f"/proc/{pid}")
                if not proc_path.exists():
                    continue
                raise
        try:
            cpu_value = float(cpu_text)
            rss_value = int(rss_text)
            if not state or not ps_start.strip() or cpu_value < 0 or rss_value < 0:
                raise ValueError("invalid ps field")
        except ValueError as error:
            raise InspectionError(f"invalid ps values for pid {pid}") from error
        processes.append({
            "pid": pid, "pgid": row_pgid, "state": state,
            "cpu_percent": cpu_value, "rss_kib": rss_value, "starttime": start,
        })
    return processes


def _load_receipt(path: Path) -> dict:
    try:
        receipt = json.loads(path.read_text(encoding="utf-8"))
        if (receipt["platform"] != platform.system()
                or receipt["platform"] not in {"Darwin", "Linux"}
                or type(receipt["leader_pid"]) is not int
                or type(receipt["pgid"]) is not int
                or type(receipt["launch_parent_pid"]) is not int
                or not isinstance(receipt["leader_starttime"], str)
                or receipt["pgid"] != receipt["leader_pid"]
                or receipt["leader_pid"] <= 1
                or receipt["launch_parent_pid"] <= 0
                or not receipt["leader_starttime"]):
            raise ValueError("invalid leader process group")
        _validate_launch_parent(receipt["launch_parent_pid"], os.getppid())
        return receipt
    except (OSError, ValueError, KeyError, TypeError) as error:
        raise InspectionError(f"invalid process receipt: {error}") from error


def _has_group_anchor(receipt: dict, members: list[dict], witnessed: dict[int, tuple[int, str]]) -> bool:
    leader = next((p for p in members if p["pid"] == receipt["leader_pid"]), None)
    if leader is not None and leader["starttime"] != receipt["leader_starttime"]:
        raise InspectionError("leader PID was reused")
    if leader is not None:
        return True
    return any(witnessed.get(p["pid"]) == (p["pgid"], p["starttime"]) for p in members)


def observe(receipt_path: Path, observations_path: Path) -> list[dict]:
    receipt = _load_receipt(receipt_path)
    witnessed = _witnesses(observations_path)
    members = [p for p in _ps_snapshot(receipt["pgid"]) if not p["state"].startswith("Z")]
    if members and not _has_group_anchor(receipt, members, witnessed):
        raise InspectionError("process group has live members but no previously witnessed ownership anchor")
    record = {"observed_at": time.time(), "pgid": receipt["pgid"], "members": members}
    observations_path.parent.mkdir(parents=True, exist_ok=True)
    with observations_path.open("a", encoding="utf-8") as stream:
        stream.write(json.dumps(record, sort_keys=True) + "\n")
        stream.flush()
        os.fsync(stream.fileno())
    return members


def _witnesses(observations_path: Path) -> dict[int, tuple[int, str]]:
    witnessed = {}
    if not observations_path.exists():
        return witnessed
    try:
        for line in observations_path.read_text(encoding="utf-8").splitlines():
            for member in json.loads(line)["members"]:
                if (type(member["pid"]) is not int or type(member["pgid"]) is not int
                        or not isinstance(member["starttime"], str)
                        or member["pid"] <= 1 or member["pgid"] <= 0 or not member["starttime"]):
                    raise ValueError("invalid observed process identity")
                witnessed[member["pid"]] = (member["pgid"], member["starttime"])
    except (OSError, ValueError, KeyError, TypeError) as error:
        raise InspectionError(f"invalid process observations: {error}") from error
    return witnessed


def _still_matching(pid: int, expected: tuple[int, str], snapshot: list[dict]) -> bool:
    current = next((p for p in snapshot if p["pid"] == pid), None)
    return bool(current and not current["state"].startswith("Z")
                and (current["pgid"], current["starttime"]) == expected)


def finish(receipt_path: Path, observations_path: Path) -> tuple[list[dict], list[int]]:
    receipt = _load_receipt(receipt_path)
    # leaderか既観測メンバーを根拠にできるときだけ、終了直前の子も証跡に加える。
    observe(receipt_path, observations_path)
    witnessed = _witnesses(observations_path)
    initial = [p for p in _ps_snapshot(receipt["pgid"]) if not p["state"].startswith("Z")]
    if initial and not _has_group_anchor(receipt, initial, witnessed):
        raise InspectionError("process group lost its ownership anchor before cleanup")
    residual = initial
    targets = {
        p["pid"]: (p["pgid"], p["starttime"])
        for p in initial
        if witnessed.get(p["pid"]) == (p["pgid"], p["starttime"])
    }

    def signal_matching(sig: int) -> None:
        snapshot = _ps_snapshot(receipt["pgid"])
        for pid, identity in targets.items():
            if _still_matching(pid, identity, snapshot):
                try:
                    os.kill(pid, sig)
                except ProcessLookupError:
                    pass

    if targets:
        signal_matching(signal.SIGTERM)
        deadline = time.monotonic() + 1.0
        while time.monotonic() < deadline:
            snapshot = _ps_snapshot(receipt["pgid"])
            if not any(_still_matching(pid, identity, snapshot) for pid, identity in targets.items()):
                break
            time.sleep(0.05)
        signal_matching(signal.SIGKILL)
        deadline = time.monotonic() + 1.0
        while time.monotonic() < deadline:
            snapshot = _ps_snapshot(receipt["pgid"])
            if not any(_still_matching(pid, identity, snapshot) for pid, identity in targets.items()):
                break
            time.sleep(0.05)
    remaining = [p["pid"] for p in _ps_snapshot(receipt["pgid"])
                 if not p["state"].startswith("Z")]
    return residual, remaining


def launch(receipt_path: Path, observations_path: Path, command: list[str]) -> None:
    if not command:
        raise InspectionError("launch requires a command after --")
    if os.name != "posix":
        raise InspectionError(f"unsupported platform: {platform.system()}")
    try:
        os.setsid()
        pid = os.getpid()
        if os.getpgrp() != pid:
            raise InspectionError("setsid did not create the expected process group")
        receipt = {
            "leader_pid": pid, "pgid": os.getpgrp(), "launch_parent_pid": os.getppid(),
            "leader_starttime": _start_time(pid), "platform": platform.system(),
            "command": command, "created_at": time.time(),
        }
        receipt_path.parent.mkdir(parents=True, exist_ok=True)
        temp_path = receipt_path.with_suffix(receipt_path.suffix + ".tmp")
        temp_path.write_text(json.dumps(receipt, sort_keys=True) + "\n", encoding="utf-8")
        os.replace(temp_path, receipt_path)
        os.execvp(command[0], command)
    except OSError as error:
        raise InspectionError(f"cannot launch runner: {error}") from error


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="action", required=True)
    for action in ("launch", "observe", "finish"):
        sub = subparsers.add_parser(action)
        sub.add_argument("receipt", type=Path)
        sub.add_argument("observations", type=Path)
        if action == "launch":
            sub.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    try:
        if args.action == "launch":
            command = args.command[1:] if args.command[:1] == ["--"] else args.command
            launch(args.receipt, args.observations, command)
            return 0
        if args.action == "observe":
            members = observe(args.receipt, args.observations)
            print(json.dumps({"member_count": len(members), "members": members}, sort_keys=True))
            return 0
        residual, remaining = finish(args.receipt, args.observations)
        print(json.dumps({"residual_observed": [p["pid"] for p in residual], "remaining_after_cleanup": remaining}, sort_keys=True))
        return 1 if residual or remaining else 0
    except InspectionError as error:
        print(f"HTML host process-group check failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
