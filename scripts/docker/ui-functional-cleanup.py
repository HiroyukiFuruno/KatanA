#!/usr/bin/env python3
"""Collect and clean one owned Docker UI run after the producer exits."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
from typing import BinaryIO

sys.path.insert(0, str(Path(__file__).resolve().parent))
import importlib.util

_MANAGER_SPEC = importlib.util.spec_from_file_location(
    "volume_lifecycle", Path(__file__).with_name("volume-lifecycle.py")
)
_MANAGER = importlib.util.module_from_spec(_MANAGER_SPEC)
_MANAGER_SPEC.loader.exec_module(_MANAGER)

OWNER = _MANAGER.OWNER
OWNER_LABEL = _MANAGER.OWNER_LABEL
RUN_LABEL = _MANAGER.RUN_LABEL
ID_RE = re.compile(r"[0-9a-f]{64}\Z")
ERROR_TAIL_BYTES = 16 * 1024


def command(*args: str, check: bool = True, input_bytes: bytes | None = None) -> subprocess.CompletedProcess:
    result = subprocess.run(["docker", *args], input=input_bytes, capture_output=True, check=False)
    if check and result.returncode:
        raise RuntimeError(result.stderr.decode(errors="replace").strip() or f"docker {' '.join(args)} failed")
    return result


def owned_container(run_id: str, name: str, id_file: Path, volumes: tuple[str, str]) -> tuple[str | None, dict | None]:
    if not id_file.exists():
        return None, None
    container_id = id_file.read_text(encoding="utf-8").strip()
    if not ID_RE.fullmatch(container_id):
        raise RuntimeError(f"invalid container ID in {id_file}")
    raw = json.loads(command("inspect", container_id).stdout)
    if len(raw) != 1:
        raise RuntimeError(f"expected one container inspection for {container_id}")
    container = raw[0]
    labels = container.get("Config", {}).get("Labels") or {}
    mounted = {entry.get("Name") for entry in container.get("Mounts", []) if entry.get("Type") == "volume"}
    if (container.get("Id") != container_id or container.get("Name") != f"/{name}"
            or labels.get(OWNER_LABEL) != OWNER or labels.get(RUN_LABEL) != run_id
            or not set(volumes).issubset(mounted)):
        raise RuntimeError(f"container ownership or mount mismatch for {name}")
    return container_id, container


def owned_volume(run_id: str, name: str, purpose: str) -> bool:
    volume = _MANAGER.inspect_volume(name)
    labels = volume.get("Labels") or {}
    return (volume.get("Name") == name and labels.get(OWNER_LABEL) == OWNER
            and labels.get(_MANAGER.PURPOSE_LABEL) == purpose
            and labels.get(RUN_LABEL) == run_id)


def export_evidence(observer_id: str, evidence_dir: Path) -> None:
    evidence_dir.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryFile() as producer_error, tempfile.TemporaryFile() as extractor_error:
        producer = start_archive_producer(observer_id, producer_error)
        try:
            extractor = start_host_extractor(evidence_dir, producer.stdout, extractor_error)
        except OSError:
            producer.stdout.close()
            producer.terminate()
            producer.wait()
            raise
        producer.stdout.close()
        extractor_status = extractor.wait()
        producer_status = producer.wait()
        errors = collect_export_errors(producer_status, producer_error, extractor_status, extractor_error)
        if errors:
            raise RuntimeError("; ".join(errors))


def start_archive_producer(observer_id: str, error_log: BinaryIO) -> subprocess.Popen[bytes]:
    return subprocess.Popen([
        "docker", "exec", observer_id, "tar", "--exclude=./package", "-C", "/evidence", "-cf", "-", "."
    ], stdout=subprocess.PIPE, stderr=error_log)


def start_host_extractor(evidence_dir: Path, archive: BinaryIO,
                         error_log: BinaryIO) -> subprocess.Popen[bytes]:
    return subprocess.Popen(["tar", "-xf", "-", "-C", str(evidence_dir)], stdin=archive,
                            stdout=subprocess.DEVNULL, stderr=error_log)


def read_error_tail(error_log: BinaryIO) -> str:
    error_log.seek(0, 2)
    size = error_log.tell()
    error_log.seek(max(0, size - ERROR_TAIL_BYTES))
    excerpt = error_log.read(ERROR_TAIL_BYTES).decode(errors="replace").strip()
    if size > ERROR_TAIL_BYTES:
        return f"[earlier error output omitted] {excerpt}"
    return excerpt


def collect_export_errors(producer_status: int, producer_error: BinaryIO,
                          extractor_status: int, extractor_error: BinaryIO) -> list[str]:
    errors = []
    producer_message = read_error_tail(producer_error)
    extractor_message = read_error_tail(extractor_error)
    if producer_status:
        message = producer_message or "no diagnostic output"
        errors.append(f"Docker archive producer exited with status {producer_status}: {message}")
    if extractor_status:
        message = extractor_message or "no diagnostic output"
        errors.append(f"host tar extraction exited with status {extractor_status}: {message}")
    return errors


def run(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--run-dir", type=Path, required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--app-name", required=True)
    parser.add_argument("--observer-name", required=True)
    parser.add_argument("--app-id-file", type=Path, required=True)
    parser.add_argument("--observer-id-file", type=Path, required=True)
    parser.add_argument("--x11-volume", required=True)
    parser.add_argument("--evidence-volume", required=True)
    parser.add_argument("--initial-status", type=int, required=True)
    args = parser.parse_args(argv)

    run_id = args.run_id
    x11, evidence = args.x11_volume, args.evidence_volume
    expected_x11, expected_evidence = f"katana-{run_id}-x11", f"katana-{run_id}-evidence"
    if (not _MANAGER.RUN_ID_RE.fullmatch(run_id) or x11 != expected_x11 or evidence != expected_evidence
            or args.app_name != f"katana-{run_id}-app" or args.observer_name != f"katana-{run_id}-observer"
            or args.run_dir.resolve() != (args.root.resolve() / "tmp" / run_id)):
        raise RuntimeError("run identity or resource names do not match the producer contract")

    status = args.initial_status
    cleanup_failed = False
    volumes = (x11, evidence)
    try:
        app_id, app = owned_container(run_id, args.app_name, args.app_id_file, volumes)
        observer_id, observer = owned_container(run_id, args.observer_name, args.observer_id_file, volumes)
    except (RuntimeError, OSError, json.JSONDecodeError) as error:
        print(f"FAIL: refusing cleanup for unverified run resources: {error}", file=sys.stderr)
        cleanup_failed = True
        app_id = observer_id = None
        app = observer = None

    for container_id, target in ((app_id, args.run_dir / "application.log"),):
        if container_id:
            logs = command("logs", container_id, check=False)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(logs.stdout + logs.stderr)
    inspect_ids = [item for item in (observer_id, app_id) if item]
    if inspect_ids:
        inspected = command("inspect", *inspect_ids, check=False)
        (args.run_dir / "containers.json").write_bytes(inspected.stdout)

    if status in (0, 2) and (app is None or observer is None):
        print("FAIL: a normal or blocked producer exit requires both owned containers", file=sys.stderr)
        cleanup_failed = True
    if observer_id is None:
        print("FAIL: no verified observer is available to export evidence", file=sys.stderr)
        cleanup_failed = True

    if app and app.get("State", {}).get("Running"):
        (args.run_dir / "forced-cleanup.txt").write_text("Forced cleanup, not normal exit\n", encoding="utf-8")
        cleanup_failed = True
        stopped = command("stop", "-t", "5", app_id, check=False)
        if stopped.returncode:
            print("FAIL: application did not stop during forced cleanup", file=sys.stderr)

    if observer_id:
        try:
            export_evidence(observer_id, args.run_dir / "evidence")
        except (RuntimeError, OSError) as error:
            print(f"FAIL: functional evidence export failed: {error}", file=sys.stderr)
            cleanup_failed = True
        else:
            record = subprocess.run([
                sys.executable, str(Path(__file__).with_name("volume-lifecycle.py")),
                "--root", str(args.root.resolve()), "record-export", run_id,
                "--x11-volume", x11, "--evidence-volume", evidence,
            ], capture_output=True, text=True, check=False)
            (args.run_dir / "evidence-export-receipt.log").write_text(
                record.stdout + record.stderr, encoding="utf-8"
            )
            if record.returncode:
                print("FAIL: evidence export receipt could not be verified", file=sys.stderr)
                cleanup_failed = True

    if observer and observer.get("State", {}).get("Running"):
        stopped = command("stop", "-t", "5", observer_id, check=False)
        if stopped.returncode:
            print("FAIL: observer did not stop", file=sys.stderr)
            cleanup_failed = True

    if status in (0, 2) and not cleanup_failed:
        for container_id, name in ((app_id, args.app_name), (observer_id, args.observer_name)):
            if not container_id:
                continue
            try:
                current_id, _ = owned_container(run_id, name,
                    args.app_id_file if name == args.app_name else args.observer_id_file, volumes)
                if current_id != container_id:
                    raise RuntimeError(f"container identity changed for {name}")
                command("rm", container_id)
            except (RuntimeError, OSError, json.JSONDecodeError) as error:
                print(f"FAIL: owned container cleanup failed for {name}: {error}", file=sys.stderr)
                cleanup_failed = True
                break
        if not cleanup_failed:
            for name, purpose in ((x11, "ui-x11"), (evidence, "ui-evidence")):
                try:
                    if not owned_volume(run_id, name, purpose):
                        raise RuntimeError(f"volume ownership mismatch for {name}")
                    refs = _MANAGER.volume_refs(name)
                    if refs:
                        raise RuntimeError(f"volume {name} is referenced by containers: {refs}")
                    command("volume", "rm", name)
                except (RuntimeError, OSError, json.JSONDecodeError) as error:
                    print(f"FAIL: owned volume cleanup failed for {name}: {error}", file=sys.stderr)
                    cleanup_failed = True
                    break

    if cleanup_failed:
        status = 1
    (args.run_dir / "driver-exit.txt").write_text(f"{status}\n", encoding="utf-8")
    print(f"Functional evidence retained: {args.run_dir}")
    return status


if __name__ == "__main__":
    try:
        raise SystemExit(run())
    except (RuntimeError, OSError, json.JSONDecodeError) as error:
        print(f"docker-ui-functional-cleanup: {error}", file=sys.stderr)
        raise SystemExit(1)
