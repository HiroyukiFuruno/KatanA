"""Write and read phase-owned Nextest JUnit files without following aliases."""

import hashlib
import json
import os
from pathlib import Path
import re
import stat


MAX_REPORT_BYTES = 16 * 1024 * 1024
PHASE_RE = re.compile(r"[a-z][a-z0-9-]*")


def write_phase_config(output: Path, phase: str, namespace: str, template: Path) -> dict:
    """Create a phase-specific repository config before Nextest writes its report."""
    report_path, config_path = _phase_paths(output, phase)
    source = _template_text(template)
    if not isinstance(namespace, str) or not namespace:
        raise ValueError("shadow JUnit namespace is invalid")
    config_path.parent.mkdir(mode=0o700, exist_ok=True)
    _reject_alias(config_path.parent)
    config = source.rstrip() + _junit_section(report_path, phase, namespace)
    _write_new(config_path, config)
    return {"report_path": str(report_path), "config_path": str(config_path),
            "config_sha256": hashlib.sha256(config.encode()).hexdigest()}


def read_report(path: Path) -> bytes:
    """Read a bounded regular report only if its filesystem identity is stable."""
    _validate_report_path(path)
    descriptor, path_before = _open_report(path)
    try:
        before = os.fstat(descriptor)
        if (not stat.S_ISREG(before.st_mode) or before.st_size > MAX_REPORT_BYTES
                or _signature(path_before) != _signature(before)):
            raise ValueError("shadow JUnit report is invalid or oversized")
        with os.fdopen(descriptor, "rb") as stream:
            descriptor = -1
            content = stream.read(MAX_REPORT_BYTES + 1)
            after = os.fstat(stream.fileno())
        path_after = os.stat(path, follow_symlinks=False)
    except OSError as error:
        raise ValueError("shadow JUnit report cannot be read") from error
    finally:
        if descriptor >= 0:
            os.close(descriptor)
    if (len(content) > MAX_REPORT_BYTES or _signature(before) != _signature(after)
            or _signature(after) != _signature(path_after)):
        raise ValueError("shadow JUnit report changed while reading")
    return content


def _phase_paths(output: Path, phase: str) -> tuple[Path, Path]:
    root = Path(output)
    if (not root.is_absolute() or root != root.resolve() or not root.is_dir()
            or not PHASE_RE.fullmatch(phase)):
        raise ValueError("shadow JUnit phase output is invalid")
    directory = root / "junit"
    _reject_alias(directory)
    report, config = directory / f"{phase}.xml", directory / f"{phase}.toml"
    if any(path.exists() or path.is_symlink() for path in (report, config)):
        raise ValueError("shadow JUnit phase output already exists")
    return report, config


def _template_text(template: Path) -> str:
    source = Path(template)
    if source.is_symlink() or not source.is_file():
        raise ValueError("shadow JUnit config template is invalid")
    text = source.read_text(encoding="utf-8")
    if "[profile.default.junit]" in text:
        raise ValueError("shadow JUnit config template already selects a report")
    return text


def _junit_section(report: Path, phase: str, namespace: str) -> str:
    # WHY: 同一targetを共有する4工程が相互にreportを上書きしないようにする。
    return ("\n\n[profile.default.junit]\n"
            f"path = {json.dumps(str(report))}\n"
            f"report-name = {json.dumps(f'katana-shadow-{namespace}-{phase}')}\n"
            'report-skipped = "all"\n')


def _reject_alias(path: Path) -> None:
    current = path
    while current != current.parent:
        if current.is_symlink() or (current.exists() and not current.is_dir()):
            raise ValueError("shadow JUnit path has an unsafe ancestor")
        current = current.parent


def _write_new(path: Path, text: str) -> None:
    try:
        with path.open("x", encoding="utf-8") as stream:
            stream.write(text)
    except OSError as error:
        raise ValueError("shadow JUnit config cannot be created") from error


def _validate_report_path(path: Path) -> None:
    if (not path.is_absolute() or path != path.resolve() or not hasattr(os, "O_NOFOLLOW")
            or any(parent.is_symlink() for parent in path.parents)):
        raise ValueError("shadow JUnit report is unsafe")


def _open_report(path: Path) -> tuple[int, os.stat_result]:
    try:
        before = os.stat(path, follow_symlinks=False)
        if not stat.S_ISREG(before.st_mode):
            raise ValueError("shadow JUnit report must be a regular file")
        return os.open(path, os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW), before
    except OSError as error:
        raise ValueError("shadow JUnit report is unsafe") from error


def _signature(result: os.stat_result) -> tuple[int, int, int, int, int]:
    return (result.st_dev, result.st_ino, result.st_size, result.st_mtime_ns, result.st_ctime_ns)
