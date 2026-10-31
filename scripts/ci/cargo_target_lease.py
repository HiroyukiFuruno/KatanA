"""Coordinate managed Cargo checks with target cleanup through a stable sidecar lock."""

from __future__ import annotations

from contextlib import contextmanager
import errno
import hashlib
import os
from pathlib import Path
import shutil
import signal
import stat
import subprocess
import sys
import time

try:
    import fcntl
except ImportError:  # WHY: Windowsでは後段のbyte-range lockを使う。
    fcntl = None

try:
    import msvcrt
except ImportError:
    msvcrt = None


class LeaseUnavailable(RuntimeError):
    """The lease cannot be safely acquired or its path is not trusted."""


class LeaseBusy(LeaseUnavailable):
    """Another managed operation currently owns an incompatible lease."""


def _runtime_location(override: Path | None = None) -> tuple[Path, Path]:
    if override is not None:
        directory = override
        anchor = Path(os.path.abspath(directory.parent))
    elif os.name == "nt":
        anchor = _windows_user_profile()
        directory = anchor / "AppData" / "Local" / "KatanA" / "target-leases"
    else:
        # WHY: XDG_RUNTIME_DIRやTMPDIRの差でcheckとcleanupのlease keyが分かれないよう、user homeに固定する。
        import pwd

        anchor = Path(pwd.getpwuid(os.getuid()).pw_dir).resolve()
        directory = anchor / ".cache" / "katana" / "target-leases"
    return directory, anchor


def _windows_user_profile() -> Path:
    import ctypes

    buffer = ctypes.create_unicode_buffer(32768)
    shell32 = ctypes.windll.shell32
    shell32.SHGetFolderPathW.argtypes = [
        ctypes.c_void_p, ctypes.c_int, ctypes.c_void_p, ctypes.c_uint, ctypes.c_wchar_p,
    ]
    shell32.SHGetFolderPathW.restype = ctypes.c_long
    result = shell32.SHGetFolderPathW(None, 0x0028, None, 0, buffer)
    if result != 0 or not buffer.value:
        raise LeaseUnavailable(f"could not resolve the Windows user profile: HRESULT {result}")
    return Path(buffer.value).resolve(strict=True)


def _runtime_directory(override: Path | None = None) -> Path:
    directory, anchor = _runtime_location(override)
    _create_and_validate_directory_chain(directory, anchor)
    return directory


def _directory_chain(directory: Path, anchor: Path) -> list[Path]:
    anchor = Path(os.path.abspath(anchor))
    directory = Path(os.path.abspath(directory))
    try:
        relative = directory.relative_to(anchor)
    except ValueError as error:
        raise LeaseUnavailable("target lease directory escaped its user runtime root") from error
    paths = [anchor]
    for component in relative.parts:
        paths.append(paths[-1] / component)
    return paths


def _validate_directory_metadata(path: Path, final: bool) -> None:
    metadata = path.lstat()
    if not stat.S_ISDIR(metadata.st_mode) or _is_reparse_point(metadata):
        raise LeaseUnavailable("target lease directory contains a symlink or non-directory")
    if os.name != "nt":
        if metadata.st_uid != os.getuid():
            raise LeaseUnavailable("target lease directory has another owner")
        mode = stat.S_IMODE(metadata.st_mode)
        if final:
            if mode != 0o700:
                raise LeaseUnavailable("target lease directory must have mode 0700")
        elif mode & 0o022:
            raise LeaseUnavailable("target lease directory parent is writable by others")


def _validate_directory_chain(directory: Path, anchor: Path) -> None:
    paths = _directory_chain(directory, anchor)
    for index, path in enumerate(paths):
        _validate_directory_metadata(path, index == len(paths) - 1)


def _create_and_validate_directory_chain(directory: Path, anchor: Path) -> None:
    paths = _directory_chain(directory, anchor)
    for index, path in enumerate(paths):
        try:
            path.lstat()
        except FileNotFoundError:
            try:
                path.mkdir(mode=0o700)
            except FileExistsError:
                pass
        _validate_directory_metadata(path, index == len(paths) - 1)


def _is_reparse_point(metadata: os.stat_result) -> bool:
    attributes = getattr(metadata, "st_file_attributes", 0)
    reparse_flag = getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0x400)
    return bool(attributes & reparse_flag)


def lease_path(target: Path, runtime_directory: Path | None = None) -> Path:
    canonical = target.expanduser().resolve(strict=False)
    planned_directory, _ = _runtime_location(runtime_directory)
    planned_directory = planned_directory.resolve(strict=False)
    try:
        planned_directory.relative_to(canonical)
    except ValueError:
        pass
    else:
        raise LeaseUnavailable("target lease directory must be outside the Cargo target")
    directory = _runtime_directory(runtime_directory).resolve(strict=True)
    identity_path = os.path.normcase(str(canonical))
    identity = hashlib.sha256(os.fsencode(identity_path)).hexdigest()
    return directory / f"{identity}.lock"


def _open_lock(path: Path) -> int:
    flags = (
        os.O_CREAT | os.O_RDWR | getattr(os, "O_NOFOLLOW", 0)
        | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NONBLOCK", 0)
    )
    descriptor = None
    try:
        try:
            existing = path.lstat()
        except FileNotFoundError:
            existing = None
        if existing is not None and (
            not stat.S_ISREG(existing.st_mode) or _is_reparse_point(existing)
        ):
            raise LeaseUnavailable("target lease path is not a regular non-symlink file")
        descriptor = os.open(path, flags, 0o600)
        metadata = os.fstat(descriptor)
    except OSError as error:
        if descriptor is not None:
            os.close(descriptor)
        raise LeaseUnavailable(f"could not open target lease ({path}): {error}") from error
    if (not stat.S_ISREG(metadata.st_mode) or _is_reparse_point(metadata)
            or metadata.st_nlink != 1):
        os.close(descriptor)
        raise LeaseUnavailable("target lease is not a private regular file")
    if existing is not None and (existing.st_dev, existing.st_ino) != (
        metadata.st_dev, metadata.st_ino
    ):
        os.close(descriptor)
        raise LeaseUnavailable("target lease changed while opening")
    if os.name != "nt":
        if metadata.st_uid != os.getuid() or stat.S_IMODE(metadata.st_mode) != 0o600:
            os.close(descriptor)
            raise LeaseUnavailable("target lease file owner or mode is unsafe")
    elif os.fstat(descriptor).st_size == 0:
        os.write(descriptor, b"\0")
    return descriptor


def _lock(descriptor: int, exclusive: bool, nonblocking: bool) -> None:
    if fcntl is not None and os.name == "posix":
        operation = fcntl.LOCK_EX if exclusive else fcntl.LOCK_SH
        if nonblocking:
            operation |= fcntl.LOCK_NB
        try:
            fcntl.flock(descriptor, operation)
        except BlockingIOError as error:
            raise LeaseBusy("target lease is held by another managed operation") from error
        except OSError as error:
            raise LeaseUnavailable(f"could not lock target lease: {error}") from error
        return

    if msvcrt is None:
        raise LeaseUnavailable("no supported target lease lock is available")
    if not exclusive:
        # WHY: Windowsのbyte-range lockはexclusiveのみなのでmanaged checkも直列化する。
        exclusive = True
    mode = msvcrt.LK_NBLCK
    deadline = time.monotonic() + 60
    while True:
        try:
            os.lseek(descriptor, 0, os.SEEK_SET)
            msvcrt.locking(descriptor, mode, 1)
            return
        except OSError as error:
            contention_errors = {errno.EACCES, errno.EAGAIN, errno.EDEADLK}
            if error.errno not in contention_errors:
                raise LeaseUnavailable(f"target lease locking is unsupported: {error}") from error
            if nonblocking:
                raise LeaseBusy("target lease is held by another managed operation") from error
            if time.monotonic() >= deadline:
                raise LeaseUnavailable("timed out waiting for target lease") from error
            time.sleep(0.05)


def _unlock(descriptor: int) -> None:
    if fcntl is not None and os.name == "posix":
        fcntl.flock(descriptor, fcntl.LOCK_UN)
        return
    if msvcrt is None:
        raise LeaseUnavailable("no supported target lease lock is available")
    os.lseek(descriptor, 0, os.SEEK_SET)
    msvcrt.locking(descriptor, msvcrt.LK_UNLCK, 1)


@contextmanager
def target_lease(
    target: Path,
    *,
    exclusive: bool,
    nonblocking: bool = False,
    runtime_directory: Path | None = None,
):
    """Hold a stable target lease; its file lives outside the cleaned target."""
    descriptor = _open_lock(lease_path(target, runtime_directory))
    locked = False
    try:
        _lock(descriptor, exclusive, nonblocking)
        locked = True
        yield descriptor
    finally:
        try:
            if locked:
                _unlock(descriptor)
        finally:
            os.close(descriptor)


def cargo_target_directory(manifest: Path, environment: dict[str, str] | None = None) -> Path:
    """Resolve the effective Cargo target directory using Cargo metadata."""
    import json
    import subprocess

    manifest = manifest.resolve()
    env = os.environ.copy() if environment is None else environment
    command = [
        "cargo", "metadata", "--no-deps", "--format-version", "1", "--locked",
        "--manifest-path", str(manifest),
    ]
    try:
        result = subprocess.run(command, env=env, capture_output=True, text=True)
    except OSError as error:
        raise LeaseUnavailable(f"could not run cargo metadata: {error}") from error
    if result.returncode:
        raise LeaseUnavailable(f"cargo metadata failed: {result.stderr.strip()}")
    try:
        target = Path(json.loads(result.stdout)["target_directory"])
    except (KeyError, TypeError, json.JSONDecodeError) as error:
        raise LeaseUnavailable(f"cargo metadata has no usable target directory: {error}") from error
    if not target.is_absolute() or target.is_symlink():
        raise LeaseUnavailable("Cargo target directory is not a safe absolute path")
    return target.resolve(strict=False)


def run_process_group(
    command: list[str],
    cwd: Path,
    environment: dict[str, str],
    inherited_fds: tuple[int, ...] = (),
) -> int:
    """Run an owned command group and wait for it after forwarding termination."""
    process = None
    handled_signals = [signal.SIGINT, signal.SIGTERM]
    if hasattr(signal, "SIGBREAK"):
        handled_signals.append(signal.SIGBREAK)
    previous_handlers = {signum: signal.getsignal(signum) for signum in handled_signals}
    pending_signals = []

    def forward_and_wait(signum, _frame):
        pending_signals.append(signum)
        if process is not None and (os.name == "posix" or process.poll() is None):
            _signal_group(process, signum)

    for signum in handled_signals:
        signal.signal(signum, forward_and_wait)
    try:
        options = {"cwd": cwd, "env": environment}
        if os.name == "posix":
            options["start_new_session"] = True
            if inherited_fds:
                options["pass_fds"] = inherited_fds
        elif os.name == "nt":
            options["creationflags"] = getattr(subprocess, "CREATE_NEW_PROCESS_GROUP", 0)
            if Path(command[0]).name == command[0]:
                # WHY: CreateProcessのSystem32優先検索を避け、指定されたPATHのproducerを実行する。
                executable = shutil.which(command[0], path=environment.get("PATH", ""))
                if executable is None:
                    raise FileNotFoundError(f"producer executable not found on PATH: {command[0]}")
                command = [str(Path(executable).absolute()), *command[1:]]
        process = subprocess.Popen(command, **options)
        if pending_signals:
            _signal_group(process, pending_signals[-1])
        return_code = process.wait()
        if os.name == "posix":
            _wait_for_process_group_exit(process.pid)
        if pending_signals:
            raise SystemExit(128 + pending_signals[0])
        return return_code
    except BaseException:
        if process is not None and process.poll() is None:
            _signal_group(process, signal.SIGTERM)
            process.wait()
        raise
    finally:
        for signum, handler in previous_handlers.items():
            signal.signal(signum, handler)


def _signal_group(process: subprocess.Popen, signum: int) -> None:
    if os.name == "posix":
        try:
            os.killpg(process.pid, signum)
        except ProcessLookupError:
            pass
        return
    if process.poll() is not None:
        return
    if os.name == "nt":
        control_break = getattr(signal, "CTRL_BREAK_EVENT", None)
        if control_break is not None:
            try:
                process.send_signal(control_break)
                return
            except OSError:
                # WHY: 制御イベントを送れない環境では子を殺さず完了までleaseを維持する。
                return
        return
    process.terminate()


def _wait_for_process_group_exit(process_group_id: int) -> None:
    while True:
        try:
            result = subprocess.run(
                ["ps", "-axo", "pgid=,stat="], capture_output=True, text=True, check=True
            )
            group_is_live = any(
                int(fields[0]) == process_group_id and "Z" not in fields[1]
                for line in result.stdout.splitlines()
                if (fields := line.split()) and len(fields) >= 2
            )
        except (OSError, subprocess.CalledProcessError, ValueError, IndexError):
            # WHY: 群の終了を確認できない場合はleaseを解放せず保護を継続する。
            group_is_live = True
        if not group_is_live:
            return
        time.sleep(0.05)


def _run_managed_check(target: Path, command: list[str], cwd: Path,
                       environment: dict[str, str]) -> int:
    try:
        with target_lease(target, exclusive=False, nonblocking=True) as lease_fd:
            inherited_fds = (lease_fd,) if os.name == "posix" else ()
            return run_process_group(command, cwd, environment, inherited_fds)
    except LeaseUnavailable as error:
        print(f"Cargo target check refused safely: {error}", file=sys.stderr)
        return 75


def main() -> int:
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target-dir", type=Path, required=True)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    arguments = parser.parse_args()
    command = arguments.command[1:] if arguments.command[:1] == ["--"] else arguments.command
    if not command:
        parser.error("a command is required after --")
    return _run_managed_check(arguments.target_dir, command, Path.cwd(), os.environ.copy())


if __name__ == "__main__":
    raise SystemExit(main())
