#!/usr/bin/env python3
import subprocess
import json
import os
import sys

def get_workspace_members():
    cmd = ["cargo", "metadata", "--format-version", "1", "--no-deps"]
    result = subprocess.run(cmd, capture_output=True, text=True, check=True)
    metadata = json.loads(result.stdout)
    return metadata['packages'], metadata.get('resolve', {'nodes': []})

def get_git_diff():
    try:
        cmd = ["git", "diff", "--name-only", "origin/master...HEAD"]
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        lines = result.stdout.splitlines()
        if not lines:
            cmd = ["git", "diff", "--name-only", "HEAD"]
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            lines = result.stdout.splitlines()
        return lines
    except subprocess.CalledProcessError:
        cmd = ["git", "diff", "--name-only", "HEAD"]
        result = subprocess.run(cmd, capture_output=True, text=True)
        return result.stdout.splitlines()

def determine_impacted_packages():
    packages, resolve = get_workspace_members()
    diffs = get_git_diff()

    impacted_packages = set()
    global_impact = False

    for diff in diffs:
        if diff in ["Cargo.toml", "Cargo.lock", "Makefile"] or diff.startswith("scripts/") or diff.startswith(".github/"):
            global_impact = True
            break

        if diff.startswith("assets/"):
            impacted_packages.add("katana-ui")
            continue

        if diff.startswith("crates/"):
            crate_name = diff.split("/")[1]
            impacted_packages.add(crate_name)

    if global_impact:
        return [m['name'] for m in packages]

    dep_graph = {}
    for pkg in packages:
        dep_graph[pkg['name']] = [dep['name'] for dep in pkg['dependencies'] if dep['name'] in [p['name'] for p in packages]]

    final_impacted = set(impacted_packages)

    changed = True
    while changed:
        changed = False
        for pkg in packages:
            if pkg['name'] not in final_impacted:
                if any(dep in final_impacted for dep in dep_graph[pkg['name']]):
                    final_impacted.add(pkg['name'])
                    changed = True

    return list(final_impacted)

if __name__ == "__main__":
    action = sys.argv[1] if len(sys.argv) > 1 else "list"
    packages = determine_impacted_packages()

    if action == "clippy":
        if not packages:
            print("No packages impacted. Skipping clippy.")
            sys.exit(0)
        cmd = ["cargo", "clippy"]
        for p in packages:
            cmd.extend(["-p", p])
        cmd.extend(["--", "-D", "warnings"])
        print(f"Running clippy for: {', '.join(packages)}")
        sys.exit(subprocess.run(cmd).returncode)
    elif action == "test":
        if not packages:
            print("No packages impacted. Skipping tests.")
            sys.exit(0)
        cmd = ["cargo", "test"]
        for p in packages:
            cmd.extend(["-p", p])
        if len(sys.argv) > 2:
            cmd.extend(sys.argv[2:])
        print(f"Running tests for: {', '.join(packages)}")
        sys.exit(subprocess.run(cmd).returncode)
    else:
        print(",".join(packages))
