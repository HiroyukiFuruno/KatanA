"""Keep published product sources immutable while repairing post-publication controllers."""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

from document_host_io import legacy, require

GIT_TIMEOUT_SECONDS = 60
OID = re.compile(r"[0-9a-f]{40}\Z")
# WHY: 公開物のソースはタグへ固定し、変更を非配布の公開後検証・整理コードだけに限定する。
CONTROLLER_PATHS = frozenset({
    ".github/workflows/build-and-release.yml",
    ".github/workflows/release-readiness.yml",
    ".github/workflows/test-and-build.yml",
    "scripts/ci/test_ci_resource_contract.py",
    "scripts/release/postpublication_controller_scope.py",
    "scripts/release/test-postpublication-controller-scope.py",
    "scripts/release/published_git_source.py",
    "scripts/release/test-published-git-source.py",
    "scripts/release/postpublication_run_admission.py",
    "scripts/release/test-postpublication-run-admission.py",
    "scripts/release/document_host_publication.py",
    "scripts/release/record-published-artifact-download.py",
    "scripts/release/check-published-artifact-identity.py",
    "scripts/release/test-release-flow-contract.py",
    "scripts/release/sync-external.sh",
    "scripts/release/test-sync-external.py",
    "scripts/release/test_windows_startup_comparison_contract.py",
})
WORKFLOW_BOUNDARIES = {
    ".github/workflows/build-and-release.yml": b"  cleanup_default:\n",
    ".github/workflows/release-readiness.yml": b"      - name: Verify actual published release artifacts\n",
}
UNPINNED_SWEEP = b"cargo install cargo-sweep --locked"
PINNED_SWEEP = b"cargo install cargo-sweep --version 0.8.0 --locked"
OLD_CLEANUP_INSTALL = b"          tool: just,cargo-deny@0.20.2,cargo-sweep@0.8.0\n          fallback: none\n"
NEW_CLEANUP_INSTALL = (b"          tool: just,cargo-deny@0.20.2\n          fallback: none\n"
                       b"      - name: Install pinned cleanup tool\n        run: " + PINNED_SWEEP + b"\n")
OLD_DOCUMENT_FILTER = b"      - 'docs/**'\n"
NEW_DOCUMENT_FILTER = OLD_DOCUMENT_FILTER + b"      - 'evidence/**'\n"
REGISTRY_RECEIPT_UPLOAD = (
    b"      - name: Preserve external registry sync results\n"
    b"        if: always()\n"
    b"        continue-on-error: true\n"
    b"        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7.0.1\n"
    b"        with:\n"
    b"          name: external-registry-sync\n"
    b"          path: release-files/external-registry-sync.json\n"
    b"          if-no-files-found: warn\n\n"
)
PIN_ONLY_PATHS = (".github/workflows/test-and-build.yml", "scripts/ci/test_ci_resource_contract.py")


def git(root: Path, *arguments: str) -> bytes:
    return subprocess.run(["git", "-C", str(root), *arguments], check=True, capture_output=True,
                          env=legacy().git_environment(), timeout=GIT_TIMEOUT_SECONDS).stdout


def require_oid(root: Path, value: str) -> None:
    require(isinstance(value, str) and OID.fullmatch(value) is not None,
            "post-publication source requires a full commit ID")
    require(git(root, "cat-file", "-t", value) == b"commit\n", "source object is not a commit")


def validate_controller_paths(paths: list[str]) -> None:
    unexpected = sorted(set(paths) - CONTROLLER_PATHS)
    require(not unexpected, "published product/build source changed: " + ", ".join(unexpected))


def validate_workflow_prefix(path: str, published: bytes, controller: bytes) -> None:
    boundary = WORKFLOW_BOUNDARIES[path]
    require(published.count(boundary) == controller.count(boundary) == 1,
            "post-publication workflow boundary missing or ambiguous")
    comparison = controller
    if path == ".github/workflows/build-and-release.yml":
        comparison = controller.replace(REGISTRY_RECEIPT_UPLOAD, b"", 1)
        comparison = comparison.replace(NEW_DOCUMENT_FILTER, OLD_DOCUMENT_FILTER, 1)
    require(published.split(boundary)[0] == comparison.split(boundary)[0],
            "build/native/publication or required readiness workflow changed")
    if path == ".github/workflows/build-and-release.yml" and published != controller:
        require(published.count(OLD_CLEANUP_INSTALL) == 1,
                "published cleanup tool definition differs")
        expected = published.replace(OLD_CLEANUP_INSTALL, NEW_CLEANUP_INSTALL, 1)
        expected = expected.replace(OLD_DOCUMENT_FILTER, NEW_DOCUMENT_FILTER, 1)
        if REGISTRY_RECEIPT_UPLOAD in controller:
            expected = expected.replace(boundary, REGISTRY_RECEIPT_UPLOAD + boundary, 1)
        require(expected == controller,
                "controller change exceeds cleanup repair and registry receipt retention")


def validate_tool_pin(published: bytes, controller: bytes) -> None:
    if published == controller:
        return
    require(published.count(UNPINNED_SWEEP) == 1 and controller.count(PINNED_SWEEP) == 1,
            "CI cleanup tool pin is missing or ambiguous")
    require(published.replace(UNPINNED_SWEEP, PINNED_SWEEP, 1) == controller,
            "CI change exceeds the pinned cleanup tool installation")


def verify_committed_scope(root: Path, published: str, controller: str) -> list[str]:
    require_oid(root, published)
    require_oid(root, controller)
    raw = git(root, "diff", "--name-only", "-z", published, controller, "--", *legacy().SOURCE_ROOTS)
    paths = [item.decode("utf-8") for item in raw.split(b"\0") if item]
    validate_controller_paths(paths)
    for path in WORKFLOW_BOUNDARIES:
        validate_workflow_prefix(path, git(root, "show", f"{published}:{path}"),
                                 git(root, "show", f"{controller}:{path}"))
    for path in PIN_ONLY_PATHS:
        validate_tool_pin(git(root, "show", f"{published}:{path}"), git(root, "show", f"{controller}:{path}"))
    return paths


def verify_controller_scope(root: Path, published: str) -> dict:
    controller = git(root, "rev-parse", "HEAD").decode().strip()
    paths = verify_committed_scope(root, published, controller)
    dirty = git(root, "diff", "--name-only", "-z", "HEAD", "--", *legacy().SOURCE_ROOTS)
    untracked = git(root, "ls-files", "--others", "--exclude-standard", "-z", "--", *legacy().SOURCE_ROOTS)
    validate_worktree_clean(dirty, untracked)
    return {"published_source": published, "controller_commit": controller,
            "controller_only_changes": paths}


def validate_worktree_clean(dirty: bytes, untracked: bytes) -> None:
    require(not dirty and not untracked, "post-publication controller source must be committed")
