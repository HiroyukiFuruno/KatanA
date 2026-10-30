#!/usr/bin/env python3
"""Check exact versioned docs/controller scope without duplicating payload acceptance."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

from v043_postpublication_scope import (
    CONTRACT_43, REPOSITORY, require, select_contract, select_tasks, validate_controller, validate_paths,
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


def verify_committed_scope(root, published=None, contract=CONTRACT_43):
    published = contract.source if published is None else published
    git(root, "merge-base", "--is-ancestor", published, "HEAD")
    changed = paths(git(root, "diff", "--name-only", "--no-renames", "-z", published, "HEAD"))
    inventory = paths(git(root, "ls-tree", "-r", "--name-only", "-z", published, "--", contract.active))
    validate_paths(changed, set(inventory), contract)
    tree = git(root, "ls-tree", "-r", "-z", "HEAD", "--", *changed) if changed else b""
    require(all(record.startswith((b"100644 blob ", b"100755 blob "))
                for record in tree.split(b"\0") if record), "nonregular evidence scope tree entry")
    for path, _anchor, _addition in contract.routes:
        validate_controller(path, git(root, "show", f"{published}:{path}"),
                            git(root, "show", f"HEAD:{path}"), contract)
    require(not git(root, "diff", "--name-only", "-z", "HEAD"),
            "postpublication changes must be committed")
    require(not git(root, "ls-files", "--others", "--exclude-standard", "-z"),
            "untracked postpublication files must be resolved")
    return changed


def task_candidates(root, contract=CONTRACT_43):
    archive = "openspec/changes/archive"
    candidates = [contract.active + "tasks.md"]
    if git(root, "ls-tree", "-d", "--name-only", "HEAD", "--", archive):
        folders = paths(git(root, "ls-tree", "-d", "--name-only", "-z", f"HEAD:{archive}"))
        candidates.extend(f"{archive}/{name}/tasks.md" for name in folders
                          if name.endswith("-" + contract.change))
    return paths(git(root, "ls-tree", "-r", "--name-only", "-z", "HEAD", "--", *candidates))


def verify_tasks(root, published=None, contract=CONTRACT_43):
    published = contract.source if published is None else published
    task = select_tasks(task_candidates(root, contract), contract)
    original = git(root, "show", f"{published}:{contract.active}tasks.md").decode("utf-8")
    current = git(root, "show", f"HEAD:{task}").decode("utf-8")
    validate_task_inventory(original, current, contract)
    scripts = root / "scripts/release"
    if task == contract.active + "tasks.md":
        command = [str(scripts / "check-openspec-task-phase.py"),
                   contract.version, contract.change, "post-release-evidence", str(root / task)]
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
    contract = select_contract(args.branch, args.version, args.repository)
    validate_route(args.branch, args.version, args.repository, contract)
    root = args.root.resolve()
    branch = os.environ.get("GITHUB_HEAD_REF") or git(root, "branch", "--show-current").decode().strip()
    require(branch == contract.branch, "checkout does not match exact evidence branch")
    validate_publication(api(f"releases/tags/{contract.tag}"), api(f"commits/{contract.tag}").get("sha"),
                         git(root, "rev-parse", "--verify", contract.tag + "^{commit}").decode().strip(), contract)
    changed = verify_committed_scope(root, contract=contract)
    task = verify_tasks(root, contract=contract)
    print(json.dumps({"published_source": contract.source, "scoped_paths": changed, "task_file": task}))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        raise SystemExit(f"postpublication scope refused: {error}") from error
