"""Initialize a producer-owned Cargo cache before writing non-Cargo outputs."""

from pathlib import Path
import argparse
from contextlib import nullcontext
import os
import stat
import sys

from cargo_target_lease import LeaseUnavailable, run_process_group, target_lease


SIGNATURE = b"Signature: 8a477f597d28d172789f06886806bc55"
TAG_CONTENT = SIGNATURE + b"\n# Cargo cache initialized by KatanA's output producer.\n"


def initialize_target(target: Path, root: Path, command: list[str] | None = None) -> int:
    root = root.resolve()
    target = _validated_target(target, root)
    # WHY: Windowsのcleanは互換lock・filesystem検査で拒否され、親の排他化された共有leaseも再取得できない。
    lifetime = target_lease(target, exclusive=False) if os.name != "nt" else nullcontext(None)
    with lifetime as descriptor:
        _initialize_owned_target(target, root)
        if not command:
            return 0
        # WHY: producerの子孫が出力を使用し終わるまで、cleanと競合する共有leaseを保持する。
        inherited_fds = (descriptor,) if os.name == "posix" else ()
        return run_process_group(command, Path.cwd(), os.environ.copy(), inherited_fds)


def _validated_target(target: Path, root: Path) -> Path:
    target = Path(os.path.abspath(target))
    relative = target.relative_to(root)
    if not relative.parts or {"src", ".git"}.intersection(relative.parts):
        raise ValueError("target overlaps repository sources or metadata")
    current = root
    for part in relative.parts:
        current /= part
        if current.is_symlink() or (hasattr(current, "is_junction") and current.is_junction()):
            raise ValueError(f"target path contains symlink: {current}")
    return target


def _initialize_owned_target(target: Path, root: Path) -> None:
    tag = target / "CACHEDIR.TAG"
    initialization = target.with_name(f".{target.name}.katana-producer-init")
    if initialization.is_symlink() or (hasattr(initialization, "is_junction") and initialization.is_junction()):
        raise ValueError(f"initialization lock identity contains symlink: {initialization}")
    # WHY: targetの共有leaseを昇格せず、別のsidecar identityで空targetの作成とタグ書込みだけ直列化する。
    with target_lease(initialization, exclusive=True):
        _validated_target(target, root)
        target.mkdir(parents=True, exist_ok=True)
        if _existing_tag_is_valid(tag):
            return
        # WHY: cleanの安全検査を迂回しないよう、既存データがあるtargetを所有済みに変えない。
        if any(target.iterdir()):
            raise ValueError(f"nonempty target has no ownership tag: {target}")
        with tag.open("xb") as stream:
            stream.write(TAG_CONTENT)


def _existing_tag_is_valid(tag: Path) -> bool:
    try:
        metadata = tag.lstat()
    except FileNotFoundError:
        return False
    if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
        raise ValueError(f"target tag is not a regular file: {tag}")
    descriptor = os.open(tag, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
                         | getattr(os, "O_NONBLOCK", 0))
    try:
        if not stat.S_ISREG(os.fstat(descriptor).st_mode) or os.read(descriptor, len(SIGNATURE)) != SIGNATURE:
            raise ValueError(f"target tag is invalid: {tag}")
    finally:
        os.close(descriptor)
    return True


def main() -> int:
    root = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("target", type=Path, nargs="?", default=root / "target")
    parser.add_argument("--run", dest="command", nargs=argparse.REMAINDER,
                        help="run a producer while its target remains protected")
    arguments = parser.parse_args()
    if arguments.command == []:
        parser.error("--run requires a producer command")
    try:
        return initialize_target(arguments.target, root, arguments.command)
    except (LeaseUnavailable, OSError, ValueError) as error:
        print(f"Cargo target initialization refused safely: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
