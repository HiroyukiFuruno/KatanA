"""Derive Cargo workspace registration from immutable manifest documents."""

from fnmatch import fnmatchcase

from downstream_dependency_records import DependencySnapshotError, table
from downstream_manifest_snapshot import (
    _dependency_tables, _resolve_workspace_entry, _workspace_dependencies,
)
from git_dependency_context import relative_manifest_path


def _patterns(workspace, key):
    values = workspace.get(key, [])
    if not isinstance(values, list) or any(not isinstance(value, str) or not value for value in values):
        raise DependencySnapshotError(f"workspace.{key} must be a string array")
    return values


def _matches(path, pattern):
    parts, masks = path.split("/"), pattern.split("/")
    reachable = {0}
    for mask in masks:
        if mask == "**":
            reachable = set(range(min(reachable), len(parts) + 1)) if reachable else set()
        else:
            reachable = {index + 1 for index in reachable if index < len(parts)
                         and fnmatchcase(parts[index], mask)}
    return len(parts) in reachable


def _relative_directory(root, manifest):
    prefix = root.removesuffix("Cargo.toml")
    return manifest[len(prefix):].removesuffix("/Cargo.toml") if manifest.startswith(prefix) else None


def _excluded(root, manifest, document):
    if manifest == root:
        return False
    members = [relative_manifest_path(root, member) for member in _patterns(document["workspace"], "members")]
    if manifest in members:
        return False
    directory = manifest.removesuffix("Cargo.toml").rstrip("/")
    masks = [relative_manifest_path(root, mask) for mask in _patterns(document["workspace"], "exclude")]
    prefixes = [mask.removesuffix("Cargo.toml").rstrip("/") for mask in masks if mask is not None]
    return any(not prefix or directory == prefix or directory.startswith(prefix + "/") for prefix in prefixes)


def _nearest_root(manifest, roots):
    matches = [root for root in roots if root != manifest
               and manifest.startswith(root.removesuffix("Cargo.toml"))]
    return max(matches, key=len) if matches else None


def _assign(owners, member, root, documents):
    if member not in documents or "package" not in documents[member]:
        raise DependencySnapshotError("workspace member must identify an existing package manifest")
    if member in owners and owners[member] != root:
        raise DependencySnapshotError("package is registered by multiple workspaces")
    if member != root and "workspace" in documents[member]:
        raise DependencySnapshotError("workspace member declares a different workspace root")
    explicit = table(documents[member]["package"], "package").get("workspace")
    if explicit is not None and not isinstance(explicit, str):
        raise DependencySnapshotError("package.workspace must be text")
    if explicit is not None and relative_manifest_path(member, explicit) != root:
        raise DependencySnapshotError("package.workspace does not match its registered workspace")
    owners[member] = root


def _explicit_members(root, documents, directories):
    workspace = table(documents[root]["workspace"], "workspace")
    members = {root} if "package" in documents[root] else set()
    for mask in _patterns(workspace, "members"):
        if any(char in mask for char in "*?["):
            normalized = relative_manifest_path(root, mask)
            if normalized is None:
                raise DependencySnapshotError("workspace member pattern escapes the repository")
            pattern = normalized.removesuffix("Cargo.toml").rstrip("/")
            selected = {f"{directory}/Cargo.toml" if directory else "Cargo.toml"
                        for directory in directories if _matches(directory, pattern)}
        else:
            target = relative_manifest_path(root, mask)
            selected = {target} if target is not None else set()
        if not selected:
            raise DependencySnapshotError("workspace member pattern has no committed manifest")
        members.update(selected)
    return {path for path in members if not _excluded(root, path, documents[root])}


def _path_targets(member, root, documents):
    workspace = _workspace_dependencies(documents[root])
    for scope, entries in _dependency_tables(documents[member]):
        if scope == "workspace.dependencies" or scope.startswith("patch.") or scope == "replace":
            continue
        for alias, value in entries.items():
            inherited = isinstance(value, dict) and value.get("workspace") is True
            resolved = _resolve_workspace_entry(alias, value, workspace)
            if isinstance(resolved, dict) and "path" in resolved:
                locator = resolved["path"]
                if not isinstance(locator, str):
                    raise DependencySnapshotError("path dependency must be text")
                yield relative_manifest_path(root if inherited else member, locator)


def _eligible_target(target, root, owners, documents, roots):
    if target is None or target not in documents:
        return False
    excluded = _excluded(root, target, documents[root])
    if target in owners:
        if owners[target] != root and _relative_directory(root, target) is not None and not excluded:
            raise DependencySnapshotError("path dependency introduces a nested workspace without exclusion")
        return False
    return _nearest_root(target, roots) == root and not excluded


def workspace_owners(documents, directories):
    roots = {path for path, document in documents.items() if "workspace" in document}
    owners = {}
    for root in sorted(roots):
        for member in _explicit_members(root, documents, directories):
            _assign(owners, member, root, documents)
    pending = list(owners)
    while pending:
        member = pending.pop()
        root = owners[member]
        for target in _path_targets(member, root, documents):
            if not _eligible_target(target, root, owners, documents, roots):
                continue
            _assign(owners, target, root, documents)
            pending.append(target)
    for path, document in documents.items():
        if "package" in document and path not in owners and _nearest_root(path, roots) is None:
            _assign(owners, path, path, documents)
    return owners
