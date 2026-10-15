"""Forward complete native transaction input before applying cleanup policy."""

import os
from pathlib import Path
import signal
import subprocess
from typing import BinaryIO

from release_cleanup_audit import CleanupRefused
from release_cleanup_transaction import MAX_TRANSACTION_BYTES

STREAM_CHUNK_BYTES = 65536
INTENT_ENV = "KATANA_CLEANUP_INTENT"
DIGEST_ENV = "KATANA_CLEANUP_INTENT_SHA256"


def owner_environment(environment: dict[str, str] | None = None) -> dict[str, str]:
    # 既存owner内の別Git操作に、単一削除用の入力を伝播させない。
    values = os.environ if environment is None else environment
    return {key: value for key, value in values.items() if key not in (INTENT_ENV, DIGEST_ENV)}


def exit_like_owner(status: int) -> None:
    if status < 0:
        number = -status
        if number not in (signal.SIGKILL, signal.SIGSTOP):
            signal.signal(number, signal.SIG_DFL)
        signal.raise_signal(number)
        raise SystemExit(1)
    raise SystemExit(status)


def owner_command(path: Path, phase: str | list[str]) -> list[str]:
    path = path.absolute()
    if path.is_symlink() or not path.is_file() or not os.access(path, os.X_OK):
        raise CleanupRefused("existing hook is not a regular executable")
    arguments = [phase] if isinstance(phase, str) else phase
    return [str(path), *arguments]


def _forward(stream: BinaryIO, owner: BinaryIO | None, limit: int) -> bytes:
    captured = bytearray()
    while chunk := stream.read(STREAM_CHUNK_BYTES):
        remaining = limit + 1 - len(captured)
        captured.extend(chunk[:remaining])
        if owner is not None:
            try:
                owner.write(chunk)
            except BrokenPipeError:
                owner = None
    return bytes(captured)


def capture_owner_input(stream: BinaryIO, path: Path | None, phase: str | list[str], *,
                        limit: int = MAX_TRANSACTION_BYTES, cwd: Path | None = None,
                        environment: dict[str, str] | None = None) -> tuple[bytes, int]:
    if path is None:
        return _forward(stream, None, limit), 0
    # 全入力は既存ownerへ渡し、追加検査用コピーだけを有界に保つ。
    with subprocess.Popen(owner_command(path, phase), stdin=subprocess.PIPE, cwd=cwd,
                          env=owner_environment(environment)) as process:
        payload = _forward(stream, process.stdin, limit)
        if process.stdin is not None:
            try:
                process.stdin.close()
            except BrokenPipeError:
                pass
        return payload, process.wait()
