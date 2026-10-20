"""Create a non-instrumented Office worker snapshot for one run."""

import argparse
import os
from pathlib import Path
import shutil
import tempfile

from native_test_run_identity import file_digest


READONLY_EXECUTABLE_MODE = 0o555


def validate_physical_chain(path: Path) -> None:
    for parent in (path, *path.parents):
        if parent.is_symlink() or (parent.exists() and not parent.is_dir()):
            raise ValueError("Office worker snapshot parent is not physical")


def validate_destination_parent(destination: Path) -> None:
    validate_physical_chain(destination.parent)


def create_snapshot_destination(base: Path, source: Path) -> Path:
    base = base.absolute()
    validate_physical_chain(base)
    base.mkdir(parents=True, exist_ok=True)
    directory = Path(tempfile.mkdtemp(prefix="coverage-worker.", dir=base))
    return directory / source.name


def snapshot_worker(source: Path, destination: Path) -> str:
    validate_destination_parent(destination)
    if not source.exists():
        raise FileNotFoundError(source)
    digest = file_digest(source)
    if not os.access(source, os.X_OK):
        raise ValueError("Office worker is not executable")
    # coverageの再linkから非instrumented workerを分離し、同一実行中に固定する。
    with source.open("rb") as original, destination.open("xb") as snapshot:
        shutil.copyfileobj(original, snapshot, length=65536)
    destination.chmod(READONLY_EXECUTABLE_MODE)
    if file_digest(source) != digest or file_digest(destination) != digest:
        raise ValueError("Office worker changed while taking snapshot")
    return digest


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("destination", type=Path, nargs="?")
    parser.add_argument("--directory", type=Path)
    args = parser.parse_args()
    if args.directory:
        if args.destination:
            parser.error("--directory cannot use a destination")
        destination = create_snapshot_destination(args.directory, args.source)
    elif args.destination:
        destination = args.destination
    else:
        parser.error("destination is required")
    snapshot_worker(args.source, destination)
    if args.directory:
        print(destination)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
