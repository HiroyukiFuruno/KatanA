"""Build a fail-closed identity for Windows compiled-target cache entries."""

import hashlib
import argparse
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import tempfile
import time

try:
    from native_test_run_identity import INPUT_ENVIRONMENT, source_identity
except ModuleNotFoundError:
    sys.path.insert(0, str(Path(__file__).parents[1] / "release"))
    from native_test_run_identity import INPUT_ENVIRONMENT, source_identity

MAX_VERSION_BYTES = 65536
KNOWN_ENVIRONMENT = frozenset((*INPUT_ENVIRONMENT, "CARGO_TERM_COLOR", "CARGO_HOME",
                               "RUSTUP_HOME", "ImageOS", "ImageVersion", "RUNNER_OS",
                               "RUNNER_ARCH"))
ENVIRONMENT_PREFIXES = ("CARGO_", "RUST", "LLVM_", "KATANA_", "JOBS", "DEBUG",
                        "Image")
IGNORED_ENVIRONMENT_NAMES = frozenset((
    "KATANA_TEST_INVENTORY_ROOT", "KATANA_TEST_INVENTORY_OUTPUT",
    "KATANA_TEST_INVENTORY_DIRECTORY", "GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT",
))


def _run_version_process(command: list[str], root: Path, environment: dict[str, str],
                         stdout: Path, stderr: Path) -> None:
    with stdout.open("wb") as output, stderr.open("wb") as errors:
        try:
            process = subprocess.Popen(command, cwd=root, env=environment,
                                       stdout=output, stderr=errors)
        except OSError as error:
            raise ValueError("toolchain version is unavailable") from error
        deadline = time.monotonic() + 10
        while process.poll() is None and time.monotonic() < deadline:
            if stdout.stat().st_size > MAX_VERSION_BYTES or stderr.stat().st_size > MAX_VERSION_BYTES:
                process.kill()
                process.wait(timeout=2)
                raise ValueError("toolchain version output is too large")
            time.sleep(0.02)
        if process.poll() is None:
            process.kill()
            process.wait(timeout=2)
            raise ValueError("toolchain version timed out")
        if process.returncode != 0:
            raise ValueError("toolchain version is unavailable")
        if stdout.stat().st_size > MAX_VERSION_BYTES or stderr.stat().st_size > MAX_VERSION_BYTES:
            raise ValueError("toolchain version output is too large")


def _version(command: list[str], root: Path, environment: dict[str, str]) -> str:
    with tempfile.TemporaryDirectory() as directory:
        stdout = Path(directory) / "stdout"
        stderr = Path(directory) / "stderr"
        _run_version_process(command, root, environment, stdout, stderr)
        return _read_version_output(stdout)


def _read_version_output(stdout: Path) -> str:
    with stdout.open("rb") as stream:
        raw = stream.read(MAX_VERSION_BYTES + 1)
    if len(raw) > MAX_VERSION_BYTES:
        raise ValueError("toolchain version output is too large")
    try:
        return raw.decode("utf-8").strip()
    except (OSError, UnicodeError) as error:
        raise ValueError("toolchain version is unavailable") from error


def _reject_unknown_environment(environment: dict[str, str]) -> None:
    unknown = [key for key in environment if key not in KNOWN_ENVIRONMENT
               and key not in IGNORED_ENVIRONMENT_NAMES
               and not key.startswith("RUNNER_") and key.startswith(ENVIRONMENT_PREFIXES)]
    if unknown:
        raise ValueError("unknown execution environment option")


def _host_version(value: str) -> str:
    for line in value.splitlines():
        if line.startswith("host: "):
            return line[6:]
    raise ValueError("rustc host is unavailable")


def _source_stable(before: dict, root: Path) -> bool:
    return source_identity(root) == before


def _payload(root: Path, environment: dict[str, str], profile: str,
             features: tuple[str, ...], test_selection: tuple[str, ...]) -> dict:
    source = source_identity(root)
    locks = source.get("locks", {})
    if (source.get("clean") is not True or "head" not in source or "tree" not in source
            or set(locks) != {"Cargo.lock", "scripts/screenshot/Cargo.lock"}
            or any(not isinstance(value, str) for value in locks.values())):
        raise ValueError("source is dirty or unavailable")
    _reject_unknown_environment(environment)
    if (environment.get("RUSTC") or environment.get("RUSTDOC")
            or environment.get("RUSTC_WRAPPER") or environment.get("RUSTC_WORKSPACE_WRAPPER")):
        raise ValueError("compiler wrapper is not identity-bound")
    rustc = _version([environment.get("RUSTC", "rustc"), "-vV"], root, environment)
    cargo = _version(["cargo", "--version"], root, environment)
    host = _host_version(rustc)
    target = environment.get("CARGO_BUILD_TARGET") or host
    values = {key: environment.get(key) for key in sorted(KNOWN_ENVIRONMENT)}
    return {"source": source, "toolchain": {"rustc": rustc, "cargo": cargo},
            "platform": {"os": platform.system(), "arch": platform.machine(), "host": host,
                          "target": target}, "profile": profile, "features": features,
            "test_selection": test_selection, "environment": values}


def _cache_key(payload: dict) -> str:
    encoded = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()
    return "windows-target-v1-" + hashlib.sha256(encoded).hexdigest()


def cache_identity(root: Path, environment: dict[str, str], profile: str,
                   features: tuple[str, ...], test_selection: tuple[str, ...]) -> dict:
    try:
        if (not isinstance(profile, str) or not profile or "\x00" in profile
                or any(not isinstance(value, str) or not value or "\x00" in value
                       for value in (*features, *test_selection))):
            raise ValueError("cache identity option is invalid")
        payload = _payload(root, environment, profile, tuple(features), tuple(test_selection))
        before = payload["source"]
        if not _source_stable(before, root):
            raise ValueError("source changed during identity collection")
        return {"schema": 1, "cache_eligible": True, "key": _cache_key(payload)}
    except (OSError, ValueError, UnicodeError):
        return {"schema": 1, "cache_eligible": False, "key": "",
                "reason": "identity_unavailable"}


def build_cache_key(root: Path, environment: dict[str, str], profile: str,
                    features: tuple[str, ...], test_selection: tuple[str, ...]) -> str:
    result = cache_identity(root, environment, profile, features, test_selection)
    if not result["cache_eligible"]:
        raise ValueError(result["reason"])
    return result["key"]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--profile", required=True)
    parser.add_argument("--feature", action="append", default=[])
    parser.add_argument("--selection", action="append", default=[])
    parser.add_argument("--github-output", type=Path)
    arguments = parser.parse_args()
    try:
        result = cache_identity(arguments.repo.resolve(), dict(os.environ), arguments.profile,
                                tuple(arguments.feature), tuple(arguments.selection))
    except (OSError, ValueError, UnicodeError):
        result = {"schema": 1, "cache_eligible": False, "key": "",
                  "reason": "identity_unavailable"}
    print(json.dumps(result, sort_keys=True))
    if arguments.github_output:
        eligible = "true" if result["cache_eligible"] else "false"
        with arguments.github_output.open("a", encoding="utf-8") as output:
            output.write(f"cache_eligible={eligible}\ncache_key={result['key']}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
