"""許可済み版の公開後記録を実tag・CI証跡・不変の製品sourceへ結合する。"""

import argparse
from datetime import date
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
import tempfile

from document_host_io import decode, fields, integer, moment, read, require, sha
from future_publication_download import api, selected_assets, verify_unchanged
from generated_asset_identity import file_digest, manifest_metadata, run_identity
from postpublication_controller_scope import git, validate_worktree_clean
from postpublication_run_admission import admit
from postpublication_source_binding import COMMIT_PATTERN, REPOSITORY, verify_tag_source
from published_git_source import SOURCE_ROOTS, git_source_identity, git_workspace_version
from v043_postpublication_scope import EVIDENCE_FILES
from v047_evidence_paths import HOST_FILES, CANONICAL_FILES, HOST_RECEIPT, HOST_MANIFESTS

VERSION = "0.22.45"
TAG = "v" + VERSION
BRANCH = "release/" + TAG + "-public-release-evidence"
EVIDENCE = "evidence/releases/" + TAG + "/"
CHANGES = {"v0-22-45-update-storage-preflight": "update-storage-preflight",
           "v0-22-45-build-identity": "generated-release-identity"}
POLICIES = {
    "0.22.47": {"v0-22-47-katana-contained-fixes": (
        "tab-group-dialog-interaction", "multi-format-document-preview",
        "generated-release-identity", "release-ci-integrity", "canonical-diagram-reference",
        "oss-repository-security", "auto-update", "workspace-shell")},
    "0.22.45": {change: (capability,) for change, capability in CHANGES.items()},
    "0.22.46": {"v0-22-46-startup-png-regex": ("local-asset-preview", "workspace-file-filter")},
}
MANIFESTS = {platform: f"manifests/artifacts-identity-{platform}/identity.json"
             for platform in ("macos", "linux", "windows")}
RECORD_FILES = EVIDENCE_FILES | {"publication-identity.json", "run-admission.json", *MANIFESTS.values()}
TASK = re.compile(r"(?m)^(\s*-\s*\[)[^\]](\]\s+(\d+\.\d+)\b[^\n]*)$")
READINESS_WORKFLOW = ".github/workflows/release-readiness.yml"


def paths(raw):
    return [path.decode("utf-8") for path in raw.split(b"\0") if path]


def validate_route(branch, version, repository):
    require(version in POLICIES and branch == f"release/v{version}-public-release-evidence"
            and repository == REPOSITORY, "exact supported records branch/version/repository required")


def task_inventory(text):
    rows = [(match[3], match[1] + match[2]) for match in TASK.finditer(text)]
    require(rows and len({number for number, _ in rows}) == len(rows), "empty or duplicate task inventory")
    return rows


def archive_change(path, version=VERSION):
    match = re.fullmatch(r"openspec/changes/archive/(\d{4}-\d{2}-\d{2})-([^/]+)/(.+)", path)
    if match is None or match[2] not in POLICIES[version]:
        return None
    date.fromisoformat(match[1])
    return match[2], match[3]


def allowed_path(path, inventory, version=VERSION):
    parsed = PurePosixPath(path)
    if parsed.is_absolute() or ".." in parsed.parts or "\\" in path or str(parsed) != path:
        return False
    evidence = f"evidence/releases/v{version}/"
    if path.startswith(evidence):
        return path[len(evidence):] in RECORD_FILES or (version == "0.22.47" and path in HOST_FILES | CANONICAL_FILES)
    if path in {f"openspec/specs/{capability}/spec.md" for capabilities in POLICIES[version].values()
                for capability in capabilities}:
        return True
    if path in inventory:
        return True
    archived = archive_change(path, version)
    return archived is not None and f"openspec/changes/{archived[0]}/{archived[1]}" in inventory


def verify_scope(root, source, version=VERSION):
    require(version in POLICIES, "unsupported records version")
    changes = POLICIES[version]
    require(COMMIT_PATTERN.fullmatch(source) is not None, "actual published source required")
    require(git_workspace_version(root, source) == "v" + version, "published source version differs")
    git(root, "merge-base", "--is-ancestor", source, "HEAD")
    require(not git(root, "diff", "--name-only", "-z", source, "HEAD", "--", *SOURCE_ROOTS),
            "published production tree must remain unchanged")
    validate_worktree_clean(git(root, "diff", "--name-only", "-z")
                            + git(root, "diff", "--cached", "--name-only", "-z", "HEAD"),
                            git(root, "ls-files", "--others", "--exclude-standard", "-z"))
    inventory = set(paths(git(root, "ls-tree", "-r", "--name-only", "-z", source, "--",
                              *(f"openspec/changes/{change}" for change in changes))))
    changed = paths(git(root, "diff", "--name-only", "--no-renames", "-z", source, "HEAD"))
    require(all(allowed_path(path, inventory, version) for path in changed), f"outside exact v{version[-2:]} records scope")
    tree = git(root, "ls-tree", "-r", "-z", "HEAD", "--", *changed) if changed else b""
    require(all(record.startswith((b"100644 blob ", b"100755 blob "))
                for record in tree.split(b"\0") if record), "nonregular records tree entry")
    locations = {}
    current = set(paths(git(root, "ls-tree", "-r", "--name-only", "-z", "HEAD", "--", "openspec/changes")))
    for change, capabilities in changes.items():
        prefix = f"openspec/changes/{change}/"
        originals = {path for path in inventory if path.startswith(prefix)}
        require(prefix + "tasks.md" in originals, "published task inventory missing")
        roots = {prefix if path.startswith(prefix) else path.rsplit(change + "/", 1)[0] + change + "/"
                 for path in current if path.startswith(prefix) or
                 (archive_change(path, version) is not None and archive_change(path, version)[0] == change)}
        require(len(roots) == 1, "exactly one complete active/archive change required")
        location = roots.pop()
        actual = {path for path in current if path.startswith(location)}
        require(actual == {location + path[len(prefix):] for path in originals}, "change file inventory differs")
        for original in originals:
            target = location + original[len(prefix):]
            before = git(root, "show", f"{source}:{original}")
            after = git(root, "show", f"HEAD:{target}")
            if original == prefix + "tasks.md":
                require(task_inventory(before.decode()) == task_inventory(after.decode()),
                        "task ID/text inventory changed")
                checker = "check-openspec-task-phase.py" if location == prefix else "check-openspec-task-completion.py"
                arguments = [version, change, "post-release-evidence"] if location == prefix else []
                subprocess.run([sys.executable, "-B", str(root / "scripts/release" / checker),
                                *arguments, str(root / target)], check=True, timeout=60)
            else:
                require(before == after, "published change content changed")
        for capability in capabilities:
            spec = f"openspec/specs/{capability}/spec.md"
            if version == "0.22.47":
                delta = git(root, "show", f"{source}:{prefix}specs/{capability}/spec.md").decode()
                require(delta.startswith("## ADDED Requirements\n"), "only published added requirements may sync")
                old = git(root, "show", f"{source}:{spec}").decode()
                require(old.count(delta.removeprefix("## ADDED Requirements\n").strip()) == 1,
                        "published main spec must already contain its delta exactly once")
                require(spec not in changed, "published main spec must remain unchanged")
                continue
            if spec in changed:
                delta = git(root, "show", f"{source}:{prefix}specs/{capability}/spec.md").decode()
                require(delta.startswith("## ADDED Requirements\n"), "only published added requirements may sync")
                synced = git(root, "show", f"HEAD:{spec}").decode()
                parts = synced.split("## Requirements\n")
                previous = git(root, "ls-tree", "--name-only", source, "--", spec)
                old = git(root, "show", f"{source}:{spec}").decode() if previous else None
                expected = (old.rstrip() + "\n\n" if old else "") + delta.removeprefix("## ADDED Requirements\n").strip()
                require(len(parts) == 2 and "## Purpose\n" in parts[0]
                        and (synced.strip() == expected.strip() if old else parts[1].strip() == expected),
                        "spec sync differs from published added requirements")
        locations[change] = location + "tasks.md"
    return changed, locations


def verify_receipt(root, receipt, release, source, build_run, build_attempt, manifests, version=VERSION):
    require(version in POLICIES, "unsupported records version")
    tag = "v" + version
    fields(receipt, {"schema", "version", "scope", "document_qa", "github_run_id", "github_run_attempt",
                     "provenance", "release_id", "assets", "controller_commit", "build_manifests"}, "identity receipt")
    require(receipt["schema"] == "katana-published-asset-identity-v1" and receipt["version"] == tag
            and receipt["scope"] == "asset-identity-only" and receipt["document_qa"] == "not_evaluated",
            "actual version-bound identity-only receipt required")
    require(run_identity(receipt["github_run_id"], receipt["github_run_attempt"]) == (build_run, build_attempt),
            "receipt run/attempt differs from admission")
    expected = git_source_identity(root, source)
    require(receipt["provenance"] == expected, "receipt source differs from published Git source")
    controller = receipt["controller_commit"]
    require(isinstance(controller, str) and COMMIT_PATTERN.fullmatch(controller) is not None,
            "actual verification controller commit required")
    require(not git(root, "diff", "--name-only", "-z", source, controller, "--", *SOURCE_ROOTS),
            "receipt controller uses different production input")
    require(integer(receipt["release_id"], "receipt Release id", minimum=1)
            == integer(release["id"], "Release id", minimum=1), "receipt Release id differs")
    selected = selected_assets(release, tag)
    fields(receipt["assets"], set(selected), "receipt assets")
    fields(receipt["build_manifests"], set(MANIFESTS), "receipt build manifests")
    for name, item in selected.items():
        record = fields(receipt["assets"][name], {"asset_id", "size", "sha256", "downloaded_at", "download_url"}, "receipt asset")
        actual_sha = sha(record["sha256"], "receipt asset")
        require(integer(record["asset_id"], "receipt asset id", minimum=1) == integer(item["id"], "asset id", minimum=1)
                and integer(record["size"], "receipt asset size", minimum=1) == integer(item["size"], "asset size", minimum=1)
                and item.get("digest") == "sha256:" + actual_sha
                and record["download_url"] == item.get("browser_download_url")
                == f"https://github.com/{REPOSITORY}/releases/download/{tag}/{name}",
                "receipt asset was replaced or differs from actual API bytes")
        require(moment(record["downloaded_at"], "download time") >= moment(release["published_at"], "publication time"),
                "receipt predates publication")
    for platform, filename in MANIFESTS.items():
        descriptor = fields(receipt["build_manifests"][platform], {"path", "sha256"}, "receipt manifest")
        require(descriptor["path"] == filename and descriptor["sha256"] == file_digest(manifests / filename),
                "receipt manifest path/hash differs")
        record = decode(read(manifests / filename))
        _, identity = manifest_metadata(platform, record, tag)
        require(identity == (build_run, build_attempt) and record["provenance"] == expected,
                "raw manifest source/run/attempt differs")
        for name, descriptor in record["assets"].items():
            downloaded = receipt["assets"][name]
            require((descriptor["sha256"], descriptor["size"]) == (downloaded["sha256"], downloaded["size"]),
                    "raw manifest and actual public asset differ")


def canonical_identity(root, source, directory, api_reader=api):
    result = api_reader(f"repos/{REPOSITORY}/actions/workflows/release-readiness.yml/runs?event=workflow_run&status=success&per_page=100")
    require(isinstance(result, dict) and isinstance(result.get("workflow_runs"), list), "readiness runs unavailable")
    candidates = []
    for run in result["workflow_runs"]:
        require(isinstance(run, dict), "invalid readiness run")
        if (run.get("path") != READINESS_WORKFLOW or run.get("event") != "workflow_run" or run.get("head_branch") != "master"
                or run.get("status") != "completed" or run.get("conclusion") != "success"
                or any(not isinstance(run.get(key), dict) or run[key].get("full_name") != REPOSITORY
                       for key in ("repository", "head_repository"))):
            continue
        head = run.get("head_sha")
        require(isinstance(head, str) and COMMIT_PATTERN.fullmatch(head) is not None, "invalid readiness source")
        if git(root, "diff", "--name-only", "-z", source, head, "--", *SOURCE_ROOTS):
            continue
        run_id, attempt = str(integer(run.get("id"), "readiness run", minimum=1)), str(integer(run.get("run_attempt"), "readiness attempt", minimum=1))
        jobs = api_reader(f"repos/{REPOSITORY}/actions/runs/{run_id}/attempts/{attempt}/jobs?per_page=100")
        require(isinstance(jobs, dict) and isinstance(jobs.get("jobs"), list)
                and jobs.get("total_count") == len(jobs["jobs"]), "complete readiness jobs required")
        identity_jobs = [job for job in jobs["jobs"] if job.get("name") == "Verify published asset identity"]
        if len(identity_jobs) != 1 or identity_jobs[0].get("status") != "completed" or identity_jobs[0].get("conclusion") != "success":
            continue
        job = identity_jobs[0]
        require(job.get("run_id") == int(run_id) and job.get("run_attempt") == int(attempt)
                and job.get("head_sha") == head, "readiness job run/attempt/source differs")
        artifacts = api_reader(f"repos/{REPOSITORY}/actions/runs/{run_id}/artifacts?per_page=100")
        require(isinstance(artifacts, dict) and isinstance(artifacts.get("artifacts"), list)
                and artifacts.get("total_count") == len(artifacts["artifacts"]), "complete readiness artifacts required")
        selected = [item for item in artifacts["artifacts"] if item.get("name") == "published-asset-identity"]
        if len(selected) == 1 and selected[0].get("expired") is False:
            artifact_run = selected[0].get("workflow_run")
            require(isinstance(artifact_run, dict) and artifact_run.get("id") == int(run_id)
                    and artifact_run.get("head_sha") == head, "canonical artifact source/run differs")
            candidates.append((integer(run["id"], "readiness run", minimum=1), head))
    require(candidates, "successful canonical public identity artifact missing")
    selected, head = max(candidates)
    subprocess.run(["gh", "run", "download", str(selected), "--repo", REPOSITORY,
                    "--name", "published-asset-identity", "--dir", str(directory)], check=True, timeout=1800)
    receipt = decode(read(directory / "publication-identity.json"))
    require(isinstance(receipt, dict), "canonical identity receipt missing")
    controller = receipt.get("controller_commit")
    require(isinstance(controller, str) and COMMIT_PATTERN.fullmatch(controller) is not None,
            "canonical controller identity missing")
    require(not git(root, "diff", "--name-only", "-z", head, controller, "--", *SOURCE_ROOTS),
            "receipt differs from trusted readiness production source")
    return receipt


def verify_v47_host_binding(root, release, source, run, attempt, canonical):
    host = decode(read(root / HOST_RECEIPT))
    identity = decode(read(canonical / "publication-identity.json"))
    require(host["source_commit"] == source and host["release_id"] == release["id"] == identity["release_id"],
            "host receipt differs from admitted public release")
    require(set(host["assets"]) == set(identity["assets"]), "host asset set differs from published identity")
    for name, item in host["assets"].items():
        actual = identity["assets"][name]
        require(all(item[key] == actual[key] for key in ("asset_id", "size", "sha256", "download_url")),
                "host download differs from admitted public asset")
    for platform, path in HOST_MANIFESTS.items():
        host_manifest = decode(read(root / path))
        admitted = decode(read(canonical / MANIFESTS[platform]))
        require(host_manifest == admitted, "host build manifest differs from admitted publication")
        _, actual_run = manifest_metadata(platform, host_manifest, "v0.22.47")
        require(actual_run == (run, attempt), "host build manifest run or attempt differs")


def verify_v47_record_inventory(root):
    expected = HOST_FILES | CANONICAL_FILES
    tracked = set(paths(git(root, "ls-tree", "-r", "--name-only", "-z", "HEAD", "--", *sorted(expected))))
    require(tracked == expected, "complete v47 host and canonical records required")


def verify(root, branch, version, repository):
    validate_route(branch, version, repository)
    tag_name = "v" + version
    actual_branch = os.environ.get("GITHUB_HEAD_REF") or git(root, "branch", "--show-current").decode().strip()
    require(actual_branch == branch, "checkout is not exact records branch")
    release = api(f"repos/{REPOSITORY}/releases/tags/{tag_name}")
    selected_assets(release, tag_name)
    tag = api(f"repos/{REPOSITORY}/commits/{tag_name}")
    require(isinstance(tag, dict), "actual published tag missing")
    source = tag.get("sha")
    verify_tag_source(tag, source)
    git(root, "fetch", "--no-tags", f"https://github.com/{REPOSITORY}.git",
        f"refs/tags/{tag_name}:refs/tags/{tag_name}", "refs/heads/master")
    require(git(root, "rev-parse", "--verify", tag_name + "^{commit}").decode().strip() == source,
            "local tag differs from actual published tag")
    changed, tasks = verify_scope(root, source, version)
    if version == "0.22.47":
        verify_v47_record_inventory(root)
    with tempfile.TemporaryDirectory(prefix=f"katana-v{version}-records-") as temporary:
        work = Path(temporary)
        run, attempt, admitted = admit(source, tag_name, work / "admission.json")
        require(admitted == source, "admitted source differs")
        canonical = work / "canonical"
        receipt = canonical_identity(root, source, canonical)
        verify_receipt(root, receipt, release, source, str(run), str(attempt), canonical, version)
        if version == "0.22.47":
            from document_host_acceptance import verify as verify_host
            from v047_canonical_reference import verify as verify_canonical
            verify_host(root, phase="postpublication", publication_version=tag_name)
            verify_canonical(root, source, release)
            verify_v47_host_binding(root, release, source, str(run), str(attempt), canonical)
        evidence = root / f"evidence/releases/{tag_name}/"
        for filename in ("publication-identity.json", *MANIFESTS.values()):
            require(read(evidence / filename) == read(canonical / filename), "committed evidence differs from canonical CI artifact")
        verify_unchanged(release, api(f"repos/{REPOSITORY}/releases/tags/{tag_name}"), tag_name)
        verify_tag_source(api(f"repos/{REPOSITORY}/commits/{tag_name}"), source)
    return {"published_source": source, "scoped_paths": changed, "task_files": tasks}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--branch", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--repository", required=True)
    args = parser.parse_args()
    print(verify(args.root.resolve(), args.branch, args.version, args.repository))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, TypeError, KeyError, subprocess.SubprocessError) as error:
        raise SystemExit(f"publication records refused: {error}") from error
