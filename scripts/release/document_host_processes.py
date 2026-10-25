"""OSが返した子孫processだけを観測する。sidecar未観測と未使用は同じ0にしない。"""

from __future__ import annotations

from datetime import datetime, timezone
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
        self.unchanged()
        output = subprocess.run(["ps", "-axo", "pid=,ppid=,%cpu="], check=True,
                                capture_output=True, text=True, timeout=10).stdout
        table = process_table(output)
        actual_main_alive = self.main_pid in table and process_identity(self.main_pid) == self.main_identity
        if main_alive:
            require(actual_main_alive,
                    "actual main PID/start/path changed")
        else:
            require(not actual_main_alive, "normal main exit was not observed by the OS")
        pids = (descendants(table, self.main_pid) if actual_main_alive else set()) | (set(self.owned) & set(table))
        children = []
        for pid in sorted(pids):
            try:
                identity = process_identity(pid)
            except (OSError, RuntimeError, ValueError) as error:
                children.append({"pid": pid, "ppid": table[pid][0], "status": "identity_unobserved",
                                 "error": type(error).__name__})
                continue
            previous = self.owned.get(pid)
            if previous is not None and identity != previous:
                # WHY: 旧PIDの新processをownedへ誤認せず、現main子孫でなければ無関係な再利用を除く。
                if pid not in descendants(table, self.main_pid):
                    continue
            self.owned[pid] = identity
            measurement = sample(pid, identity)
            role = "office_sidecar" if identity["executable"] == str(self.worker) else "unclassified_descendant"
            children.append({"pid": pid, "ppid": table[pid][0], "identity": identity,
                             "cpu_percent": table[pid][1], "memory": measurement,
                             "role": role, "executable_sha256": self.hashes[self.worker] if role == "office_sidecar" else None,
                             "status": "observed" if measurement["status"] == "ok" else "memory_unobserved"})
        return {"observed_at": datetime.now(timezone.utc).isoformat(),
                "main_cpu_percent": table[self.main_pid][1] if self.main_pid in table else None,
                "main_alive": actual_main_alive, "children": children}

    def finish(self) -> dict:
        result = self.snapshot(main_alive=False)
        for path, expected in self.hashes.items():
            with path.open("rb") as handle:
                require(hashlib.file_digest(handle, "sha256").hexdigest() == expected, "actual executable bytes changed during observation")
        require(not result["children"], "owned child remains after normal main exit")
        return result
