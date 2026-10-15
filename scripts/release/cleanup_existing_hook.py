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


def owner_environment() -> dict[str, str]:
    # 既存owner内の別Git操作に、単一削除用の入力を伝播させない。
    return {key: value for key, value in os.environ.items() if key not in (INTENT_ENV, DIGEST_ENV)}


def exit_like_owner(status: int) -> None:
    if status < 0:
        number = -status
        if number not in (signal.SIGKILL, signal.SIGSTOP):
            signal.signal(number, signal.SIG_DFL)
        signal.raise_signal(number)
        raise SystemExit(1)
    raise SystemExit(status)


def owner_command(path: Path, phase: str) -> list[str]:
    path = path.absolute()
    if path.is_symlink() or not path.is_file() or not os.access(path, os.X_OK):
        raise CleanupRefused("existing reference hook is not a regular executable")
    return [str(path), phase]


def _forward(stream: BinaryIO, owner: BinaryIO | None) -> bytes:
    captured = bytearray()
    while chunk := stream.read(STREAM_CHUNK_BYTES):
        remaining = MAX_TRANSACTION_BYTES + 1 - len(captured)
        captured.extend(chunk[:remaining])
        if owner is not None:
            try:
                owner.write(chunk)
            except BrokenPipeError:
                owner = None
    return bytes(captured)


def capture_owner_input(stream: BinaryIO, path: Path | None, phase: str) -> tuple[bytes, int]:
    if path is None:
        return _forward(stream, None), 0
    # 全入力は既存ownerへ渡し、追加検査用コピーだけを有界に保つ。
    with subprocess.Popen(owner_command(path, phase), stdin=subprocess.PIPE, env=owner_environment()) as process:
        payload = _forward(stream, process.stdin)
        if process.stdin is not None:
            try:
                process.stdin.close()
            except BrokenPipeError:
                pass
        return payload, process.wait()
