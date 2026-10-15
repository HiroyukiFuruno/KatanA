#!/usr/bin/env python3
"""Install the owner-preserving reference hook only for compatible worktrees."""

import argparse
import subprocess
import sys

from hook_installation_scope import git_path, validate_checkout_runtimes, validate_hooks_scope
from reference_hook_installation import install


def main(arguments: list[str] | None = None) -> int:
    argparse.ArgumentParser(description=__doc__).parse_args(arguments)
    try:
        common = git_path("rev-parse", "--git-common-dir")
        hooks = git_path("rev-parse", "--git-path", "hooks")
        validate_hooks_scope(hooks, common)
        validate_checkout_runtimes()
        changed = install(hooks)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"reference dispatcher install failed: {error}", file=sys.stderr)
        return 2
    print("installed reference dispatcher" if changed else "reference dispatcher already installed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
