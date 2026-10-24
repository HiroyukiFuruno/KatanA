"""Process-local prepared Cargo fixture roots; never cache test results."""

from contextlib import contextmanager
import atexit
import hashlib
import os
import shutil
import stat
from pathlib import Path
import subprocess
import threading
import tempfile
from typing import Callable, Iterator
import unittest
from unittest.mock import patch

from native_test_run_identity import environment_identity, source_identity


MAX_ENTRIES = 8
MAX_SCAN_ENTRIES = 4096
MAX_SCAN_DEPTH = 32


def _clear_runtime_outputs(root: Path) -> None:
    """Remove only diagnostic/profile outputs, preserving Cargo build artifacts."""
    target = root / "target"
    if target.is_symlink():
        raise RuntimeError("fixture target directory is a symlink")
    deletions = []
    for path in (target / "inventory", root / "inventory", target / "report.json"):
        if path.is_symlink():
            raise RuntimeError("fixture runtime output is a symlink")
        if path.is_dir():
            deletions.append(("directory", path))
        elif path.exists():
            deletions.append(("file", path))
    profiles = _profile_outputs(target) if target.is_dir() else []
    deleted_directories = [path for kind, path in deletions if kind == "directory"]
    deletions.extend(("file", path) for path in profiles
                     if not any(parent in path.parents for parent in deleted_directories))
    _apply_runtime_deletions(deletions)


def _profile_outputs(target: Path) -> list[Path]:
    profiles = []
    scanned = 0
    for directory, child_directories, files in os.walk(target, topdown=True, followlinks=False):
        relative = Path(directory).relative_to(target)
        if len(relative.parts) > MAX_SCAN_DEPTH:
            raise RuntimeError("fixture target scan depth exceeded")
        scanned += len(child_directories) + len(files)
        if scanned > MAX_SCAN_ENTRIES:
            raise RuntimeError("fixture target scan entry limit exceeded")
        linked_directories = [Path(directory) / name for name in child_directories
                              if (Path(directory) / name).is_symlink()]
        if linked_directories:
            raise RuntimeError("fixture target contains a symlink directory")
        child_directories[:] = sorted(child_directories)
        for name in sorted(files):
            path = Path(directory) / name
            if path.suffix != ".profraw":
                continue
            if path.is_symlink() or not path.is_file():
                raise RuntimeError("fixture profile output is not a regular file")
            profiles.append(path)
    return profiles


def _apply_runtime_deletions(deletions: list[tuple[str, Path]]) -> None:
    for kind, path in deletions:
        if kind == "directory":
            shutil.rmtree(path)
        else:
            path.unlink()


def _remove_readonly_fixture_file(func, path, exc_info, root: Path) -> None:
    error = exc_info[1]
    candidate = Path(path)
    resolved = candidate.resolve(strict=False)
    if not (resolved == root or root in resolved.parents):
        raise error.with_traceback(exc_info[2])
    if (
        not isinstance(error, PermissionError)
        or getattr(error, "winerror", None) != 5
        or func not in (os.unlink, os.remove)
        or candidate.is_symlink()
        or not candidate.is_file()
        or candidate.stat().st_mode & stat.S_IWRITE
    ):
        raise error.with_traceback(exc_info[2])
    os.chmod(path, candidate.stat().st_mode | stat.S_IWRITE)
    func(path)


def _remove_owned_fixture_tree(root: Path) -> None:
    resolved_root = root.resolve()
    shutil.rmtree(
        root,
        onerror=lambda func, path, exc_info: _remove_readonly_fixture_file(
            func, path, exc_info, resolved_root
        ),
    )


class PreparedFixtureBuildCache:
    """Lease immutable fixture roots while allowing Cargo artifacts to stay warm."""

    def __init__(self, max_entries: int = MAX_ENTRIES) -> None:
        if max_entries <= 0 or max_entries > MAX_ENTRIES:
            raise ValueError("fixture cache entry limit is invalid")
        self._max_entries = max_entries
        self._entries: dict[str, tuple[Path, dict, tuple]] = {}
        self._lock = threading.RLock()
        self._active = False
        self._root = Path(tempfile.mkdtemp(prefix="katana-native-fixture-cache-"))

    def _entry(self, key: str, prepare: Callable[[Path], None]) -> tuple[Path, dict, tuple]:
        callback_identity = _callback_identity(prepare)
        entry = self._entries.get(key)
        if entry is None:
            if len(self._entries) >= self._max_entries:
                raise RuntimeError("fixture cache entry limit reached")
            root = (self._root / f"entry-{len(self._entries)}").resolve()
            root.mkdir()
            prepare(root)
            identity = _identity(root)
            if identity["source"].get("clean") is not True:
                raise RuntimeError("prepared fixture source is not clean")
            entry = (root, identity, callback_identity)
            self._entries[key] = entry
        return entry

    def _validate_entry(self, key: str, prepare: Callable[[Path], None]) -> Path:
        root, identity, previous_callback = self._entry(key, prepare)
        if _callback_identity(prepare) != previous_callback:
            raise RuntimeError("prepared fixture callback changed")
        if _identity(root) != identity:
            raise RuntimeError("prepared fixture source changed")
        _clear_runtime_outputs(root)
        return root

    @contextmanager
    def lease(self, key: str, prepare: Callable[[Path], None]) -> Iterator[Path]:
        if not isinstance(key, str) or not key:
            raise ValueError("fixture cache key is invalid")
        with self._lock:
            if self._active:
                raise RuntimeError("nested fixture cache lease is forbidden")
            root = self._validate_entry(key, prepare)
            self._active = True
            try:
                yield root
            finally:
                self._active = False
                if _identity(root) != self._entries[key][1]:
                    raise RuntimeError("prepared fixture source changed during lease")

    def close(self) -> None:
        with self._lock:
            if self._root.exists():
                _remove_owned_fixture_tree(self._root)
            self._entries.clear()


def _identity(root: Path) -> dict:
    environment = dict(os.environ)
    observed = environment_identity(root, environment)
    if observed.get("cargo") is None or observed.get("rustc") is None:
        raise RuntimeError("prepared fixture toolchain identity is unavailable")
    observed["environment"].pop("CARGO_TARGET_DIR", None)
    observed["environment"].pop("CARGO_BUILD_BUILD_DIR", None)
    return {"source": source_identity(root), "toolchain": observed}


def _callback_identity(callback: Callable[[Path], None]) -> tuple:
    code = getattr(callback, "__code__", None)
    if code is None:
        raise ValueError("fixture cache callback identity is unavailable")
    closure = tuple(repr(cell.cell_contents) for cell in (callback.__closure__ or ()))
    return (callback.__module__, callback.__qualname__, code.co_filename,
            code.co_firstlineno, hashlib.sha256(code.co_code).hexdigest(), closure)


_CACHE = PreparedFixtureBuildCache()
atexit.register(_CACHE.close)


def prepared_fixture(key: str, prepare: Callable[[Path], None]) -> Iterator[Path]:
    """Lease one process-local prepared fixture; callers still run Cargo every time."""
    return _CACHE.lease(key, prepare)


def _prepare_git_fixture(root: Path) -> None:
    (root / "scripts/screenshot").mkdir(parents=True)
    (root / ".gitignore").write_text("/target\n", encoding="utf-8")
    (root / "Cargo.lock").write_text("version = 4\n", encoding="utf-8")
    (root / "scripts/screenshot/Cargo.lock").write_text("version = 4\n", encoding="utf-8")
    environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
    for command in (("init", "-q"), ("config", "user.name", "Fixture"),
                    ("config", "user.email", "fixture@example.invalid")):
        subprocess.run(["git", "-C", str(root), *command], env=environment, check=True,
                       capture_output=True)
    subprocess.run(["git", "-C", str(root), "add", "."], env=environment, check=True,
                   capture_output=True)
    subprocess.run(["git", "-C", str(root), "-c", "commit.gpgsign=false", "commit", "-qm",
                    "fixture"], env=environment, check=True, capture_output=True)


def _prepare_git_fixture_alt(root: Path) -> None:
    _prepare_git_fixture(root)


class PreparedFixtureBuildCacheTests(unittest.TestCase):
    def setUp(self) -> None:
        self.cache = PreparedFixtureBuildCache()

    def tearDown(self) -> None:
        self.cache.close()

    def test_entry_limit_is_bounded(self) -> None:
        cache = PreparedFixtureBuildCache(max_entries=1)
        try:
            with cache.lease("one", _prepare_git_fixture):
                pass
            with self.assertRaisesRegex(RuntimeError, "entry limit"):
                with cache.lease("two", _prepare_git_fixture):
                    pass
        finally:
            cache.close()

    def test_close_removes_readonly_git_loose_objects(self) -> None:
        with self.cache.lease("one", _prepare_git_fixture):
            pass
        root = self.cache._entries["one"][0]
        loose_objects = [
            path
            for path in (root / ".git/objects").glob("*/*")
            if path.is_file()
        ]
        self.assertTrue(loose_objects)
        for path in loose_objects:
            path.chmod(path.stat().st_mode & ~stat.S_IWUSR)

        self.cache.close()

        self.assertFalse(root.exists())

    def test_readonly_cleanup_rejects_writable_files_and_directories(self) -> None:
        root = Path(tempfile.mkdtemp(prefix="katana-readonly-cleanup-boundary-"))
        try:
            writable = root / "writable"
            writable.write_bytes(b"fixture")
            directory = root / "directory"
            directory.mkdir()
            error = PermissionError(13, "permission denied", str(writable))
            error.winerror = 5
            exc_info = (type(error), error, None)

            with self.assertRaises(PermissionError):
                _remove_readonly_fixture_file(os.unlink, writable, exc_info, root)
            with self.assertRaises(PermissionError):
                _remove_readonly_fixture_file(os.unlink, directory, exc_info, root)
        finally:
            shutil.rmtree(root)

    def test_readonly_cleanup_callback_unlinks_owned_file_and_rejects_boundaries(self) -> None:
        root = Path(tempfile.mkdtemp(prefix="katana-readonly-cleanup-callback-"))
        outside = Path(tempfile.mkdtemp(prefix="katana-readonly-cleanup-outside-"))
        resolved_root = root.resolve()
        try:
            owned = root / "owned"
            owned.write_bytes(b"fixture")
            owned.chmod(owned.stat().st_mode & ~stat.S_IWUSR)
            access_denied = PermissionError(13, "permission denied", str(owned))
            access_denied.winerror = 5
            exc_info = (type(access_denied), access_denied, None)
            _remove_readonly_fixture_file(os.unlink, owned, exc_info, resolved_root)
            self.assertFalse(owned.exists())

            external = outside / "external"
            external.write_bytes(b"keep")
            external.chmod(external.stat().st_mode & ~stat.S_IWUSR)
            with self.assertRaises(PermissionError):
                _remove_readonly_fixture_file(os.unlink, external, exc_info, resolved_root)
            self.assertTrue(external.exists())

            target = outside / "target"
            target.write_bytes(b"keep")
            target.chmod(target.stat().st_mode & ~stat.S_IWUSR)
            link = root / "link"
            link.symlink_to(target)
            with self.assertRaises(PermissionError):
                _remove_readonly_fixture_file(os.unlink, link, exc_info, resolved_root)
            self.assertTrue(link.is_symlink())
            self.assertTrue(target.exists())

            owned.write_bytes(b"keep")
            owned.chmod(owned.stat().st_mode & ~stat.S_IWUSR)
            wrong_error = ValueError("different error")
            with self.assertRaises(ValueError):
                _remove_readonly_fixture_file(
                    os.unlink, owned, (type(wrong_error), wrong_error, None), resolved_root
                )
            with self.assertRaises(PermissionError):
                _remove_readonly_fixture_file(os.rmdir, owned, exc_info, resolved_root)
            self.assertTrue(owned.exists())
        finally:
            if (root / "link").is_symlink():
                (root / "link").unlink()
            for path in (root / "owned", outside / "external", outside / "target"):
                if path.exists():
                    path.chmod(path.stat().st_mode | stat.S_IWRITE)
            shutil.rmtree(root)
            shutil.rmtree(outside)

    def test_nested_lease_is_rejected_even_with_reentrant_lock(self) -> None:
        with self.cache.lease("one", _prepare_git_fixture):
            with self.assertRaisesRegex(RuntimeError, "nested"):
                with self.cache.lease("two", _prepare_git_fixture):
                    pass

    def test_source_drift_and_callback_drift_fail_closed(self) -> None:
        with self.assertRaisesRegex(RuntimeError, "source changed"):
            with self.cache.lease("one", _prepare_git_fixture) as root:
                (root / "Cargo.lock").write_text("version = 3\n", encoding="utf-8")
        with self.cache.lease("two", _prepare_git_fixture):
            pass
        with self.assertRaisesRegex(RuntimeError, "callback changed"):
            with self.cache.lease("two", _prepare_git_fixture_alt):
                pass

    def test_profile_symlink_never_deletes_external_file(self) -> None:
        outside = Path(tempfile.mkdtemp(prefix="katana-profile-outside-"))
        try:
            with self.cache.lease("one", _prepare_git_fixture):
                pass
            root = self.cache._entries["one"][0]
            target = root / "target"
            target.mkdir()
            external = outside / "outside.profraw"
            external.write_bytes(b"keep")
            (target / "linked.profraw").symlink_to(external)
            with self.assertRaisesRegex(RuntimeError, "profile output"):
                with self.cache.lease("one", _prepare_git_fixture):
                    pass
            self.assertEqual((outside / "outside.profraw").read_bytes(), b"keep")
        finally:
            shutil.rmtree(outside)

    def test_target_symlink_never_deletes_external_inventory(self) -> None:
        outside = Path(tempfile.mkdtemp(prefix="katana-inventory-outside-"))
        try:
            with self.cache.lease("one", _prepare_git_fixture):
                pass
            root = self.cache._entries["one"][0]
            external = outside / "inventory"
            external.mkdir()
            (external / "keep.json").write_text("keep", encoding="utf-8")
            (root / "target").symlink_to(external, target_is_directory=True)
            with self.assertRaisesRegex(RuntimeError, "target directory"):
                with self.cache.lease("one", _prepare_git_fixture):
                    pass
            self.assertTrue((external / "keep.json").is_file())
        finally:
            shutil.rmtree(outside)

    def test_child_directory_symlink_and_scan_limit_fail_closed(self) -> None:
        outside = Path(tempfile.mkdtemp(prefix="katana-target-outside-"))
        try:
            with self.cache.lease("one", _prepare_git_fixture):
                pass
            root = self.cache._entries["one"][0]
            target = root / "target"
            target.mkdir()
            (target / "linked").symlink_to(outside, target_is_directory=True)
            with self.assertRaisesRegex(RuntimeError, "symlink directory"):
                with self.cache.lease("one", _prepare_git_fixture):
                    pass
            (target / "linked").unlink()
            (target / "one.profraw").write_bytes(b"profile")
            (target / "two.profraw").write_bytes(b"profile")
            with patch(__name__ + ".MAX_SCAN_ENTRIES", 1):
                with self.assertRaisesRegex(RuntimeError, "entry limit"):
                    with self.cache.lease("one", _prepare_git_fixture):
                        pass
        finally:
            shutil.rmtree(outside)

    def test_missing_toolchain_identity_is_rejected(self) -> None:
        with patch(__name__ + ".environment_identity",
                   return_value={"cargo": None, "rustc": "rustc", "environment": {}}):
            with self.assertRaisesRegex(RuntimeError, "toolchain identity"):
                with self.cache.lease("tool", _prepare_git_fixture):
                    pass

