"""OSが返した子孫processだけを観測する。sidecar未観測と未使用は同じ0にしない。"""

from __future__ import annotations

from datetime import datetime, timezone
import errno
import hashlib
from pathlib import Path
import subprocess

from document_host_io import number, require
from process_memory_sampler import process_identity, sample


def process_table(text: str) -> dict[int, tuple[int, float]]:
    records = {}
    for line in text.splitlines():
        if not line.strip():
            continue
        parts = line.split()
        require(len(parts) == 3, "OS process table fields differ")
        pid, parent, cpu = int(parts[0]), int(parts[1]), float(parts[2])
        require(pid > 0 and parent >= 0 and pid not in records, "OS process table identity invalid")
        number(cpu, "actual OS CPU percent")
        records[pid] = (parent, cpu)
    return records


def descendants(table: dict[int, tuple[int, float]], main_pid: int) -> set[int]:
    owned = {main_pid}
    while True:
        added = {pid for pid, (parent, _) in table.items() if parent in owned} - owned
        if not added:
            return owned - {main_pid}
        owned.update(added)


def signature(path: Path) -> tuple[int, int, int, int, int]:
    require(path.is_file() and not path.is_symlink(), "actual process executable must be regular")
    value = path.stat()
    return value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns, value.st_ctime_ns


class ProcessObserver:
    def __init__(self, main_pid: int, main: Path, worker: Path):
        require(main.is_absolute() and worker.is_absolute(), "actual main/worker paths required")
        self.main_pid, self.main, self.worker = main_pid, main, worker
        self.main_identity = process_identity(main_pid)
        require(self.main_identity["executable"] == str(main), "actual main process path differs")
        self.signatures = {path: signature(path) for path in (main, worker)}
        self.hashes = {}
        for path in (main, worker):
            with path.open("rb") as handle:
                self.hashes[path] = hashlib.file_digest(handle, "sha256").hexdigest()
        self.owned: dict[int, dict[str, str]] = {}

    def unchanged(self) -> None:
        for path, expected in self.signatures.items():
            require(signature(path) == expected, "actual main/worker inode/stat changed during observation")

    def snapshot(self, *, main_alive: bool = True) -> dict:
        # WHY: ownedには失敗sampleも入るため、完了済みの正しいworker観測だけを別に保持する。
        verified = getattr(self, "_verified_workers", {})
        terminated = []
        unknown_terminated = []
        retired_pids = set()

        def read_table():
            output = subprocess.run(["ps", "-axo", "pid=,ppid=,%cpu="], check=True,
                                    capture_output=True, text=True, timeout=10).stdout
            table = process_table(output)
            require(not (retired_pids & set(table)), "retired child PID reappeared during OS snapshot")
            return output, table, datetime.now(timezone.utc).isoformat()

        def check_main(table):
            if self.main_pid not in table and main_alive:
                raise ProcessLookupError(errno.ESRCH, "actual main process is absent from the OS table")
            alive = self.main_pid in table
            if alive:
                require(process_identity(self.main_pid) == self.main_identity, "actual main PID/start/path changed")
            require(alive == main_alive, "normal main exit was not observed by the OS")
            return alive

        def verified_worker(pid, identity=None):
            previous = verified.get(pid)
            return (previous is not None and previous["identity"]["executable"] == str(self.worker)
                    and previous["executable_sha256"] == self.hashes[self.worker]
                    and self.owned.get(pid) == previous["identity"] and pid in current_descendants
                    and (identity is None or identity == previous["identity"]))

        def retired_worker(pid, error, stage, identity=None):
            known = verified_worker(pid, identity)
            first_seen = (stage == "identity" and pid not in self.owned and pid not in verified
                          and pid in current_descendants)
            if not (isinstance(error, ProcessLookupError) and error.errno == errno.ESRCH
                    and (known or first_seen)):
                return None
            fresh = read_table()
            check_main(fresh[1])
            if pid in fresh[1]:
                return None
            observation = {"pid": pid, "ppid": table[pid][0],
                           "error": "ProcessLookupError", "errno": errno.ESRCH,
                           "observation_stage": stage,
                           "before_observed_at": observed_at, "before_ps": output,
                           "after_observed_at": fresh[2], "after_ps": fresh[0],
                           "main_identity": self.main_identity, "pid_absent": True}
            if known:
                terminated.append(observation | {"previous_verified": verified[pid]})
            else:
                # WHY: 消失した未同定PIDをworkerの正の観測やmemory=0へ読み替えない。
                unknown_terminated.append(observation | {"classification": "UNKNOWN-RETIRED"})
            retired_pids.add(pid)
            return fresh

        current = read_table()
        for attempt in range(3):
            self.unchanged()
            output, table, observed_at = current
            actual_main_alive = check_main(table)
            current_descendants = descendants(table, self.main_pid) if actual_main_alive else set()
            pids = current_descendants | (set(self.owned) & set(table))
            children, completed = [], {}
            refreshed = None
            for pid in sorted(pids):
                try:
                    identity = process_identity(pid)
                except (OSError, RuntimeError, ValueError) as error:
                    fresh = retired_worker(pid, error, "identity")
                    if fresh is not None:
                        refreshed = fresh
                        continue
                    children.append({"pid": pid, "ppid": table[pid][0], "status": "identity_unobserved",
                                     "error": type(error).__name__})
                    continue
                previous = self.owned.get(pid)
                if previous is not None and identity != previous:
                    # WHY: 読み直しでも、別start identityを旧workerの観測値へ読み替えない。
                    require(pid not in current_descendants, "actual owned child PID/start/path changed")
                    continue
                self.owned[pid] = identity
                try:
                    # WHY: 成功履歴と今回identityが一致するworkerだけ、文字列化前のESRCHを照合する。
                    measurement = (sample(pid, identity, raise_on_missing_process=True)
                                   if verified_worker(pid, identity) else sample(pid, identity))
                except ProcessLookupError as error:
                    fresh = retired_worker(pid, error, "memory", identity)
                    # WHY: childの例外をmain終了待ちへ流さず、不在未証明はその場で不合格にする。
                    require(fresh is not None, f"actual worker memory failure without proven exit: PID {pid}: {error}")
                    refreshed = fresh
                    continue
                role = "office_sidecar" if identity["executable"] == str(self.worker) else "unclassified_descendant"
                children.append({"pid": pid, "ppid": table[pid][0], "identity": identity,
                                 "cpu_percent": table[pid][1], "memory": measurement,
                                 "role": role, "executable_sha256": self.hashes[self.worker] if role == "office_sidecar" else None,
                                 "status": "observed" if measurement["status"] == "ok" else "memory_unobserved"})
                if (role == "office_sidecar" and measurement["status"] == "ok"
                        and measurement.get("identity") == identity and measurement.get("pid") == pid):
                    completed[pid] = {"identity": identity, "executable_sha256": self.hashes[self.worker],
                                      "observed_at": measurement["observed_at"]}
            if refreshed is not None:
                # WHY: 全initial PIDを調べ終える前に再取得表を採用せず、他childの異常を再取得で捨てない。
                require(all(child["status"] == "observed" and child["role"] == "office_sidecar"
                            and child["memory"].get("identity") == child["identity"]
                            and child["memory"].get("pid") == child["pid"] for child in children),
                        "invalid actual child observation cannot be discarded by OS table refresh")
                # WHY: 対象PIDを削るのでなく、main/新descendant/retained ownedを全て実OSから作り直す。
                current = refreshed
                continue
            if all(child["status"] == "observed" and child["role"] == "office_sidecar" for child in children):
                verified.update(completed)
                self._verified_workers = verified
            return {"observed_at": datetime.now(timezone.utc).isoformat(),
                    "main_cpu_percent": table[self.main_pid][1] if self.main_pid in table else None,
                    "main_alive": actual_main_alive, "children": children,
                    **({"terminated_children": terminated} if terminated else {}),
                    **({"unknown_retired_children": unknown_terminated} if unknown_terminated else {})}
        require(False, "OS child snapshot did not stabilize within three attempts")

    def finish(self) -> dict:
        result = self.snapshot(main_alive=False)
        for path, expected in self.hashes.items():
            with path.open("rb") as handle:
                require(hashlib.file_digest(handle, "sha256").hexdigest() == expected, "actual executable bytes changed during observation")
        require(not result["children"], "owned child remains after normal main exit")
        return result
