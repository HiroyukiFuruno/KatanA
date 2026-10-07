"""Cargoの絶対targetパスをGit Bashでも使用できる表現へ揃える。"""

import json
from pathlib import PurePosixPath, PureWindowsPath
import sys


def normalize_target(value: str, windows: bool) -> str:
    path = PureWindowsPath(value) if windows else PurePosixPath(value)
    if not path.is_absolute():
        raise ValueError(f"cargo metadata returned a non-absolute target directory: {value}")
    return path.as_posix()


if __name__ == "__main__":
    metadata = json.load(sys.stdin)
    print(normalize_target(metadata["target_directory"], sys.platform == "win32"))
