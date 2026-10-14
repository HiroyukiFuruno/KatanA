"""Compare every committed dependency input using registered path identities."""

from dataclasses import dataclass
from pathlib import Path
from types import MappingProxyType

from cargo_workspace_membership import _path_targets
from downstream_dependency_records import DependencyDeclaration, changes
from downstream_lock_snapshot import lock_snapshot
from git_workspace_context import GitWorkspaceContext


@dataclass(frozen=True)
class ExternalDeclaration:
    manifest_path: str
    declaration: DependencyDeclaration
    path_origin: str | None

    @property
    def identity(self):
        return repr((self.manifest_path, self.declaration.scope, self.declaration.alias))


class DependencyPushSnapshot:
    def __init__(self, root: Path, commit: str):
        self.context = GitWorkspaceContext(root, commit)
        self.manifests = MappingProxyType({path: self._manifest(path) for path in self.context.files
                                          if path.endswith("Cargo.toml")})
        self.lockfiles = MappingProxyType({path: self._lockfile(path) for path in self.context.files
                                          if path.endswith("Cargo.lock")})

    def _manifest(self, path):
        result = []
        for binding in self.context.bindings(path):
            if binding.internal is not None:
                continue
            declaration = binding.origin.declaration
            base = binding.origin.origin_path if declaration.source.kind == "path" else None
            result.append(ExternalDeclaration(path, declaration, base))
        return tuple(result)

    def _reachable(self, workspace):
        visited = set()
        pending = [path for path, owner in self.context.owners.items() if owner == workspace]
        while pending:
            path = pending.pop()
            if path in visited:
                continue
            visited.add(path)
            owner = self.context.owners[path]
            targets = _path_targets(path, owner, self.context._documents)
            pending.extend(target for target in targets if target in self.context.packages and target not in visited)
        return {(self.context.packages[path].name, self.context.packages[path].version) for path in visited}

    def _lockfile(self, path):
        manifest = path.removesuffix("Cargo.lock") + "Cargo.toml"
        document = self.context._documents.get(manifest)
        if document is None:
            internal = set()
        else:
            owner = self.context.owners.get(manifest)
            workspace = manifest if "workspace" in document or owner == manifest else None
            internal = self._reachable(workspace) if workspace is not None else set()
        packages = lock_snapshot(self.context.files[path].content, frozenset())
        return tuple(package for package in packages
                     if package.source is not None or (package.name, package.version) not in internal)

    def compare(self, before):
        old_manifests = () if before is None else tuple(item for values in before.manifests.values() for item in values)
        manifests = tuple(item for values in self.manifests.values() for item in values)
        manifest_changes = changes(old_manifests, manifests, lambda item: item.identity)
        lock_changes = []
        paths = set(self.lockfiles) | (set(before.lockfiles) if before is not None else set())
        for path in sorted(paths):
            old = () if before is None else before.lockfiles.get(path, ())
            new = self.lockfiles.get(path, ())
            delta = changes(old, new, lambda item: repr((item.name, item.version, item.source)))
            lock_changes.extend((path, item) for item in delta)
        return manifest_changes, tuple(lock_changes)
