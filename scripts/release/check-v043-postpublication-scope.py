#!/usr/bin/env python3
"""Check exact v43 docs/controller scope without duplicating payload acceptance."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

from v043_postpublication_scope import (
    ACTIVE, BRANCH, CHANGE, REPOSITORY, ROUTE_EDITS, SOURCE, TAG, VERSION,
    require, select_tasks, validate_controller, validate_paths,
    validate_publication, validate_route, validate_task_inventory,
)


def git(root, *arguments):
    return subprocess.run(["git", "-C", str(root), *arguments], check=True,
                          capture_output=True, timeout=60).stdout


def paths(raw):
    return [path.decode("utf-8") for path in raw.split(b"\0") if path]


def api(endpoint):
    result = subprocess.run(["gh", "api", f"repos/{REPOSITORY}/{endpoint}"],
                            check=True, capture_output=True, text=True, timeout=60)
    return json.loads(result.stdout)


def verify_committed_scope(root, published=SOURCE):
    git(root, "merge-base", "--is-ancestor", published, "HEAD")
    changed = paths(git(root, "diff", "--name-only", "--no-renames", "-z", published, "HEAD"))
    inventory = paths(git(root, "ls-tree", "-r", "--name-only", "-z", published, "--", ACTIVE))
    validate_paths(changed, set(inventory))
    tree = git(root, "ls-tree", "-r", "-z", "HEAD", "--", *changed) if changed else b""
    require(all(record.startswith((b"100644 blob ", b"100755 blob "))
                for record in tree.split(b"\0") if record), "nonregular v43 scope tree entry")
    for path in ROUTE_EDITS:
        validate_controller(path, git(root, "show", f"{published}:{path}"),
                            git(root, "show", f"HEAD:{path}"))
    require(not git(root, "diff", "--name-only", "-z", "HEAD"),
            "postpublication changes must be committed")
    require(not git(root, "ls-files", "--others", "--exclude-standard", "-z"),
            "untracked postpublication files must be resolved")
    return changed


def task_candidates(root):
    archive = "openspec/changes/archive"
    candidates = [ACTIVE + "tasks.md"]
    if git(root, "ls-tree", "-d", "--name-only", "HEAD", "--", archive):
        folders = paths(git(root, "ls-tree", "-d", "--name-only", "-z", f"HEAD:{archive}"))
        candidates.extend(f"{archive}/{name}/tasks.md" for name in folders
                          if name.endswith("-" + CHANGE))
    return paths(git(root, "ls-tree", "-r", "--name-only", "-z", "HEAD", "--", *candidates))


def verify_tasks(root, published=SOURCE):
    task = select_tasks(task_candidates(root))
    original = git(root, "show", f"{published}:{ACTIVE}tasks.md").decode("utf-8")
    current = git(root, "show", f"HEAD:{task}").decode("utf-8")
    validate_task_inventory(original, current)
    scripts = root / "scripts/release"
    if task == ACTIVE + "tasks.md":
        command = [str(scripts / "check-openspec-task-phase.py"),
                   VERSION, CHANGE, "post-release-evidence", str(root / task)]
    else:
        command = [str(scripts / "check-openspec-task-completion.py"), str(root / task)]
    subprocess.run([sys.executable, "-B", *command], check=True)
    return task


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--branch", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--repository", required=True)
    args = parser.parse_args()
    validate_route(args.branch, args.version, args.repository)
    root = args.root.resolve()
    branch = os.environ.get("GITHUB_HEAD_REF") or git(root, "branch", "--show-current").decode().strip()
    require(branch == BRANCH, "checkout does not match exact v43 evidence branch")
    validate_publication(api(f"releases/tags/{TAG}"), api(f"commits/{TAG}").get("sha"),
                         git(root, "rev-parse", "--verify", TAG + "^{commit}").decode().strip())
    changed = verify_committed_scope(root)
    task = verify_tasks(root)
    print(json.dumps({"published_source": SOURCE, "scoped_paths": changed, "task_file": task}))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        raise SystemExit(f"v43 postpublication scope refused: {error}") from error
