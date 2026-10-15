"""Keep one real invocation and its exit status; attach diagnostic metadata only."""

from datetime import datetime, timezone
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import uuid

from native_test_artifact_listing import list_artifacts
from native_test_cargo_artifacts import CargoArtifactInventory
from native_test_run_identity import environment_identity, source_identity

MAX_LINE_BYTES = 65536


def cargo_command(argv: list[str], kind: str) -> list[str]:
    expected = "test" if kind == "test" else "llvm-cov"
    if argv[:2] != ["cargo", expected]:
        raise ValueError("inventory requires the explicit Cargo test or llvm-cov command")
    boundary = argv.index("--") if "--" in argv else len(argv)
    cargo_arguments = argv[:boundary]
    if any(argument.startswith("--message-format") for argument in cargo_arguments):
        raise ValueError("inventory owns the Cargo JSON message format")
    if "--no-run" in cargo_arguments or "--list" in argv[boundary:]:
        raise ValueError("inventory must observe real tests, not replace them with listing")
    if kind == "coverage":
        return argv
    return [*argv[:boundary], "--message-format=json", *argv[boundary:]]


def _consume_output(process, inventory: CargoArtifactInventory | None) -> None:
    continued = False
    while chunk := process.stdout.readline(MAX_LINE_BYTES + 1):
        sys.stdout.buffer.write(chunk)
        sys.stdout.buffer.flush()
        if not continued and inventory is not None:
            inventory.consume(chunk)
        continued = not chunk.endswith(b"\n")


def _execute(command: list[str], root: Path, inventory: CargoArtifactInventory | None) -> int:
    with subprocess.Popen(command, cwd=root, stdout=subprocess.PIPE) as process:
        try:
            _consume_output(process, inventory)
        except BaseException:
            process.kill()
            process.wait()
            raise
        return process.wait()


def _save(output: Path, manifest: dict) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix=".inventory-", dir=output.parent)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            json.dump(manifest, stream, sort_keys=True, indent=2)
            stream.write("\n")
        os.replace(temporary, output)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def _manifest(root: Path, kind: str, argv: list[str], before: dict, environment: dict) -> dict:
    return {"schema": 1, "run_id": uuid.uuid4().hex, "kind": kind,
            "observed_at": datetime.now(timezone.utc).isoformat(), "cwd": str(root),
            "requested_argv": argv, "source_before": before, "inputs": environment,
            "complete": False, "reuse_authorized": False,
            "limitations": ["doctest inventory not captured", "author-writable diagnostic, not CI proof",
                            "native binary digest measured at listing, not at test execution",
                            "environment capture is selected inputs, not a hermetic environment"]}


def run_recording(root: Path, output: Path, argv: list[str], kind: str) -> int:
    command = cargo_command(argv, kind)
    before, environment = source_identity(root), environment_identity(root)
    inventory = CargoArtifactInventory()
    status = _execute(command, root, inventory if kind == "test" else None)
    manifest = _manifest(root, kind, argv, before, environment)
    artifacts = inventory.result()
    records, reasons = list_artifacts(artifacts["artifacts"], root, kind)
    after = source_identity(root)
    manifest.update({"executed_argv": command, "exit_code": status, "source_after": after,
                     "source_stable": before == after and before.get("clean") is True,
                     "cargo": artifacts, "native_listings": records, "listing_reasons": reasons})
    if kind == "coverage":
        manifest["limitations"].append("cargo llvm-cov does not expose Cargo artifact JSON here")
    _save(output, manifest)
    print(f"inventory diagnostic: {output}; complete=false; tests exit={status}", file=sys.stderr)
    return status
