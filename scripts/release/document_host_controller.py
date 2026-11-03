"""公開製品Pと、承認された検査controller Cを別の不変identityへ束縛する。"""

import ast
from pathlib import Path

from document_host_io import fields, legacy, require
from published_git_source import SOURCE_ROOTS, git_source_identity, git_workspace_version, verify_git_provenance

PUBLISHED_SOURCE = "6dabe8531e2b340630887967b63206b59ebc6de3"
SCHEMA = "katana-v47-host-controller-v1"
CONTROLLER_FIELDS = {"schema", "published_source", "controller_commit", "controller_source_tree_sha256"}
# WHY: scripts全体を免除せず、本人承認の検査訂正に必要な閉包だけを固定する。
HOST_CONTROLLER_PATHS = frozenset({
    "scripts/release/postpublication_controller_scope.py",
    "scripts/release/test-postpublication-controller-scope.py",
    "scripts/release/document_host_controller.py",
    "scripts/release/test-document-host-controller.py",
    "scripts/release/document_host_heartbeat.py",
    "scripts/release/test-document-host-acceptance.py",
    "scripts/release/check-pr-ready.sh",
    "scripts/release/select-pr-readiness-mode.sh",
    "scripts/release/test-postpublication-controller-readiness.py",
    "scripts/release/collect-document-host-memory.py",
    "scripts/release/document_host_processes.py",
    "scripts/release/process_memory_sampler.py",
    "scripts/release/test-document-host-exit-race.py",
    "scripts/release/record-document-host-snapshot.py",
    "scripts/release/run-macos-document-host-qa.sh",
    "scripts/release/document_host_acceptance.py",
    "scripts/release/document_host_transcript.py",
    "scripts/release/produce-document-host-evidence.py",
    "scripts/release/produce-document-host-evidence-v47.py",
    "scripts/release/future_publication_records.py",
    "scripts/release/test_publication_records_47.py",
    "scripts/release/test-document-host-v47.py",
    "scripts/release/test-document-host-provenance-publication.py",
    "scripts/release/document_host_html_policy.py",
    "scripts/release/test-document-host-html-policy.py",
    "scripts/release/run-document-host-observation-watch.py",
    "scripts/release/test-document-host-watch.py",
    "scripts/release/document_host_control_panel.py",
    "scripts/release/run-document-host-control-panel.py",
    "scripts/release/test-document-host-control-panel.py",
})


def validate_paths(paths: list[str]) -> None:
    require(not set(paths) - HOST_CONTROLLER_PATHS,
            "change outside approved v47 verification closure: " + ", ".join(sorted(set(paths) - HOST_CONTROLLER_PATHS)))


def _function(raw: bytes, name: str) -> ast.FunctionDef:
    functions = [node for node in ast.parse(raw).body if isinstance(node, ast.FunctionDef) and node.name == name]
    require(len(functions) == 1, "protected verification function missing or duplicated: " + name)
    return functions[0]


def _same_function(before: bytes, after: bytes, name: str) -> None:
    require(ast.dump(_function(before, name), include_attributes=False)
            == ast.dump(_function(after, name), include_attributes=False),
            "protected verification contract changed: " + name)


def _constants(raw: bytes) -> dict:
    return {name.id:ast.dump(node.value, include_attributes=False)
            for node in ast.parse(raw).body if isinstance(node, ast.Assign)
            for name in node.targets if isinstance(name, ast.Name) and name.id.isupper()}


def validate_protected_contracts(path: str, before: bytes, after: bytes) -> None:
    if path == "scripts/release/document_host_transcript.py":
        def fixed_nodes(raw):
            return [ast.dump(node, include_attributes=False) for node in ast.parse(raw).body
                    if not (isinstance(node, ast.FunctionDef) and node.name == "verify")]
        require(fixed_nodes(before) == fixed_nodes(after), "transcript schema/resource/operation contract changed")
        for name in ("resources", "settings", "loaded_source", "rendered_input", "operation"):
            _same_function(before, after, name)
        # WHY: controller照合の入口だけを追加し、全sample/時間/メモリ/操作の実判定は元のまま固定する。
        def body(raw):
            nodes = _function(raw, "verify").body
            start = next((index for index, node in enumerate(nodes)
                          if isinstance(node, ast.Expr) and isinstance(node.value, ast.Call)
                          and ast.unparse(node.value).startswith("require(record['schema']")), None)
            require(start is not None, "protected transcript boundary missing")
            return [ast.dump(node, include_attributes=False) for node in nodes[start:]]
        require(body(before) == body(after), "transcript timing/memory/identity contract changed")
    elif path == "scripts/release/document_host_acceptance.py":
        require(_constants(before) == _constants(after), "host scope/approval/schema constants changed")
        _same_function(before, after, "verify_v47_timing")
    elif path == "scripts/release/document_host_heartbeat.py":
        def constants(raw):
            return [ast.dump(node, include_attributes=False) for node in ast.parse(raw).body
                    if isinstance(node, ast.Assign)
                    and any(isinstance(name, ast.Name) and name.id.isupper() for name in node.targets)]
        require(constants(before) == constants(after), "heartbeat interval/retry constants changed")
        _same_function(before, after, "verify")
        def observer(raw):
            classes = [node for node in ast.parse(raw).body
                       if isinstance(node, ast.ClassDef) and node.name == "HeartbeatObserver"]
            require(len(classes) == 1, "protected heartbeat observer missing or duplicated")
            return ast.dump(classes[0], include_attributes=False)
        require(observer(before) == observer(after), "heartbeat observer poll/lifecycle contract changed")
    elif path == "scripts/release/process_memory_sampler.py":
        require(_constants(before) == _constants(after), "sampler ABI/identity constants changed")
        for name in ("_base", "validate_pid", "_linux_starttime", "_darwin_identity", "process_identity",
                     "_identity_mismatch", "_darwin_memory", "_linux_memory", "_read_memory"):
            _same_function(before, after, name)
    elif path == "scripts/release/document_host_processes.py":
        for name in ("process_table", "descendants", "signature"):
            _same_function(before, after, name)
        def methods(raw):
            cls = next(node for node in ast.parse(raw).body if isinstance(node, ast.ClassDef) and node.name == "ProcessObserver")
            return {node.name:ast.dump(node, include_attributes=False) for node in cls.body
                    if isinstance(node, ast.FunctionDef) and node.name in {"__init__", "unchanged", "finish"}}
        require(methods(before) == methods(after), "OS owned-child/executable terminal contract changed")
    elif path == "scripts/release/collect-document-host-memory.py":
        _same_function(before, after, "main")


def verify_controller(root: Path, published: str, controller: str | None = None) -> dict:
    from postpublication_controller_scope import git, require_oid, validate_worktree_clean
    require(published == PUBLISHED_SOURCE, "v47 controller requires the exact published source")
    require_oid(root, published)
    require(git_workspace_version(root, published) == "v0.22.47", "published controller version differs")
    controller = controller or git(root, "rev-parse", "HEAD").decode().strip()
    require_oid(root, controller)
    git(root, "merge-base", "--is-ancestor", published, controller)
    changed = [path.decode() for path in git(root, "diff", "--name-only", "--no-renames", "-z",
                                           published, controller).split(b"\0") if path]
    validate_paths(changed)
    for path in changed:
        current = git(root, "ls-tree", "-z", controller, "--", path)
        require(current.startswith((b"100644 blob ", b"100755 blob ")) and current.count(b"\0") == 1,
                "controller path must remain a regular committed file")
        previous = git(root, "ls-tree", "-z", published, "--", path)
        if previous:
            require(previous.split(b" ", 1)[0] == current.split(b" ", 1)[0], "controller file mode changed")
            validate_protected_contracts(path, git(root, "show", f"{published}:{path}"),
                                         git(root, "show", f"{controller}:{path}"))
    require(not git(root, "diff", "--name-only", "-z", controller, "HEAD", "--", *SOURCE_ROOTS),
            "current verification source differs from selected controller")
    validate_worktree_clean(git(root, "diff", "--name-only", "-z", "HEAD", "--", *SOURCE_ROOTS),
                            git(root, "ls-files", "--others", "--exclude-standard", "-z", "--", *SOURCE_ROOTS))
    controller_hash = git_source_identity(root, controller)["source_tree_sha256"]
    # WHY: Gitの改行filterでdiffが空でも、実行したcheckout bytesがCと異なれば拒否する。
    require(legacy().source_tree_sha256(root) == controller_hash,
            "controller checkout bytes differ from committed Git objects")
    return {"schema": SCHEMA, "published_source": published, "controller_commit": controller,
            "controller_source_tree_sha256": controller_hash}


def verify_binding(root: Path, source: dict, value: object) -> dict:
    binding = fields(value, CONTROLLER_FIELDS, "v47 host controller binding")
    require(binding["schema"] == SCHEMA and binding["published_source"] == source.get("source_commit"),
            "controller binding differs from published product source")
    verify_git_provenance(root, source, PUBLISHED_SOURCE)
    expected = verify_controller(root, PUBLISHED_SOURCE, binding["controller_commit"])
    require(binding == expected, "verification controller commit/tree changed")
    return binding


def published_provenance(root: Path, source: dict, controller: object) -> dict:
    verify_binding(root, source, controller)
    return source
