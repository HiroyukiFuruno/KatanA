"""完全一致の公開後契約でsource不変性とtaskfile構造を検査する。"""
from dataclasses import dataclass
from datetime import date
from pathlib import PurePosixPath
import re

BRANCH = "release/v0.22.43-public-release-evidence"
VERSION = "0.22.43"
TAG = "v0.22.43"
REPOSITORY = "HiroyukiFuruno/KatanA"
SOURCE = "0d139094a39887a7a5657ecdd63f53a6db012fde"
CHANGE = "v0-22-43-document-ui-regressions"
ACTIVE = f"openspec/changes/{CHANGE}/"
ARCHIVE = re.compile(rf"openspec/changes/archive/(\d{{4}}-\d{{2}}-\d{{2}})-{CHANGE}/(.+)\Z")
EVIDENCE = "evidence/releases/v0.22.43/"
EVIDENCE_FILES = frozenset({
    "publication.md", "release-run.json", "release-jobs.json", "public-release.json",
    "source-binding.json", "downloaded-release.sha256", "payload-byte-comparison.json",
    "archive-contract.log", "attestation-verification.json", "external-registry-sync.json",
    "homebrew-cask-validation.log",
})
CAPABILITIES = (
    "docker-resource-lifecycle", "image-preview-interaction", "multi-format-document-preview",
    "release-ci-integrity", "tab-context-menu",
)
SELECTOR_ANCHOR = b"else\n    printf '%s\\n' \"$requested_mode\"\nfi\n"
SELECTOR_INSERT = (
    b'elif [[ "$branch" == "release/v0.22.43-public-release-evidence" &&\n'
    b'      "$target_version" == "0.22.43" &&\n'
    b'      ( -z "$requested_mode" || "$requested_mode" == "pr-bootstrap" ) ]]; then\n'
    b"    printf '%s\\n' 'post-release-evidence'\n"
)
READINESS_ANCHOR = b'          elif [[ "${GITHUB_HEAD_REF}" == docs/v*-public-release-evidence ]]; then\n'
READINESS_INSERT = (
    b'          elif [[ "${GITHUB_HEAD_REF}" == release/v0.22.43-public-release-evidence && "$version" == "0.22.43" ]]; then\n'
    b'            ./scripts/release/check-pr-ready.sh "$version" --post-release-evidence\n'
)
CHECKER_ANCHOR = b"# 6. Branch naming vs Target Version for Release branches\n"
CHECKER_INSERT = "# WHY: v43の公開後記録だけを公開sourceと実在するtaskfileへ結び付ける。\n".encode("utf-8") + (
    b'if [[ "$CURRENT_BRANCH" == "release/v0.22.43-public-release-evidence" &&\n'
    b'      "$TARGET_VERSION" == "0.22.43" && "$TASK_GATE_MODE" == "post-release-evidence" ]]; then\n'
    b'    python3 -B scripts/release/check-v043-postpublication-scope.py \\\n'
    b'        --root "$PWD" --branch "$CURRENT_BRANCH" --version "$TARGET_VERSION" \\\n'
    b'        --repository "${KATANA_RELEASE_REPO:-HiroyukiFuruno/KatanA}"\n'
    b'fi\n\n'
)
ROUTE_EDITS = {
    "scripts/release/select-pr-readiness-mode.sh": (SELECTOR_ANCHOR, SELECTOR_INSERT),
    ".github/workflows/release-readiness.yml": (READINESS_ANCHOR, READINESS_INSERT),
    "scripts/release/check-pr-ready.sh": (CHECKER_ANCHOR, CHECKER_INSERT),
}
CONTROLLERS = frozenset({
    *ROUTE_EDITS, "scripts/release/test-release-flow-contract.py",
    "scripts/release/v043_postpublication_scope.py",
    "scripts/release/check-v043-postpublication-scope.py",
    "scripts/release/test_v043_postpublication_scope.py",
    "scripts/release/test_v043_postpublication_scope_git.py",
    "scripts/release/postpublication_source_binding.py",
    "scripts/release/release_branch_version.py",
    "scripts/release/test_postpublication_suffix_binding.py",
    "scripts/release/cleanup_target.py",
    "scripts/release/cleanup_remote_push.py",
    "scripts/release/test_cleanup_target.py",
    "scripts/release/test_cleanup_remote_push.py",
    "scripts/release/sync-external.sh",
    "scripts/release/test_sync_external_diagnostics.py",
    "scripts/release/probe_registry_credential.py",
    "scripts/release/test_probe_registry_credential.py",
    ".github/workflows/registry-credential-diagnostics.yml",
})
TASK_PATTERN = re.compile(r"(?m)^\s*-\s*\[([^\]])\]\s+(\d+\.\d+)\b")


@dataclass(frozen=True)
class ScopeContract:
    version: str
    source: str
    change: str
    capabilities: tuple[str, ...]
    controllers: frozenset[str]
    routes: tuple[tuple[str, bytes, bytes], ...]

    @property
    def branch(self):
        return f"release/v{self.version}-public-release-evidence"

    @property
    def tag(self):
        return f"v{self.version}"

    @property
    def active(self):
        return f"openspec/changes/{self.change}/"

    @property
    def evidence(self):
        return f"evidence/releases/v{self.version}/"


CONTRACT_43 = ScopeContract(VERSION, SOURCE, CHANGE, CAPABILITIES, CONTROLLERS,
                           tuple((path, *edit) for path, edit in ROUTE_EDITS.items()))
# WHY: 候補SHAの採用は実公開tag/API/local tagの一致検査を通った場合だけに限定する。
CONTRACT_44 = ScopeContract(
    "0.22.44", "20bf151f40165084c8748b4cca8c7ab54ff6bdf2", "v0-22-44-document-loading-input-regressions",
    ("multi-format-document-preview",), frozenset({*ROUTE_EDITS,
        "scripts/release/v043_postpublication_scope.py", "scripts/release/check-v043-postpublication-scope.py",
        "scripts/release/test_v044_postpublication_scope.py", "scripts/release/test-release-flow-contract.py"}),
    tuple((path, anchor, addition.replace(b"0.22.43", b"0.22.44").replace(
        CHECKER_INSERT.splitlines()[0],
        "# WHY: v44も公開sourceに固定し、製品を変更しない記録と整理だけを許可する。".encode()))
          for path, (anchor, addition) in ROUTE_EDITS.items()))


def select_contract(branch, version, repository):
    matches = [contract for contract in (CONTRACT_43, CONTRACT_44)
               if (branch, version, repository) == (contract.branch, contract.version, REPOSITORY)]
    require(len(matches) == 1, "exact supported evidence branch/version/repository required")
    return matches[0]


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def validate_route(branch, version, repository, contract=CONTRACT_43):
    require((branch, version, repository) == (contract.branch, contract.version, REPOSITORY),
            "exact evidence branch/version/repository required")


def validate_publication(release, api_source, local_source, contract=CONTRACT_43):
    require(isinstance(release, dict) and release.get("tag_name") == contract.tag
            and release.get("draft") is False and release.get("prerelease") is False
            and release.get("published_at"), "actual stable published Release required")
    require(api_source == local_source == contract.source, "published tag differs from fixed source")


def archive_suffix(path, contract=CONTRACT_43):
    match = re.fullmatch(rf"openspec/changes/archive/(\d{{4}}-\d{{2}}-\d{{2}})-{contract.change}/(.+)", path)
    if match is None:
        return None
    date.fromisoformat(match[1])
    return match[2]


def allowed_path(path, change_files, contract=CONTRACT_43):
    parsed = PurePosixPath(path)
    if parsed.is_absolute() or ".." in parsed.parts or "\\" in path or str(parsed) != path:
        return False
    if path in contract.controllers or path in {f"openspec/specs/{name}/spec.md" for name in contract.capabilities}:
        return True
    if path.startswith(contract.evidence):
        return path[len(contract.evidence):] in EVIDENCE_FILES
    if path.startswith(contract.active):
        return path in change_files
    suffix = archive_suffix(path, contract)
    return suffix is not None and contract.active + suffix in change_files


def validate_paths(paths, change_files, contract=CONTRACT_43):
    rejected = [path for path in paths if not allowed_path(path, change_files, contract)]
    require(not rejected, "outside exact evidence scope: " + ", ".join(rejected))


def expected_controller(path, published, contract=CONTRACT_43):
    anchor, addition = {path: (anchor, addition) for path, anchor, addition in contract.routes}[path]
    require(published.count(anchor) == 1 and addition not in published,
            "published routing boundary missing or duplicated")
    return published.replace(anchor, addition + anchor, 1)


def validate_controller(path, published, current, contract=CONTRACT_43):
    require(current == expected_controller(path, published, contract),
            f"routing exceeds exact evidence addition: {path}")


def task_rows(text):
    rows = TASK_PATTERN.findall(text)
    ids = [number for _mark, number in rows]
    require(ids and len(ids) == len(set(ids)), "task inventory empty or duplicated")
    return {number: mark.lower() for mark, number in rows}


def select_tasks(paths, contract=CONTRACT_43):
    candidates = [path for path in paths
                  if path == contract.active + "tasks.md" or archive_suffix(path, contract) == "tasks.md"]
    require(len(candidates) == 1, "exactly one active/archive tasks file required")
    return candidates[0]


def validate_task_inventory(original, current, contract=CONTRACT_43):
    require(set(task_rows(original)) == set(task_rows(current)), f"{contract.version} task ID inventory changed")
