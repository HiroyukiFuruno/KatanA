"""Bind candidates to same-commit manifests; this is not internal ownership."""

from collections.abc import Mapping
from dataclasses import dataclass, field
from pathlib import Path
import re
from types import MappingProxyType

from downstream_dependency_records import DependencySnapshotError, load, table, text
from downstream_evidence_fields import EvidenceSchemaError, _safe_path
from git_dependency_tree import GitDependencyFile, read_dependency_tree
from manifest_dependency_origins import ManifestDependencyOrigin, snapshot_manifest_origins


@dataclass(frozen=True)
class GitDependencyBinding:
    commit: str
    origin: ManifestDependencyOrigin
    target: GitDependencyFile | None


def _relative_target(origin: ManifestDependencyOrigin) -> str | None:
    return relative_manifest_path(origin.origin_path, origin.declaration.source.locator)


def relative_manifest_path(origin_path: str, locator: str) -> str | None:
    if locator.startswith("/") or "\\" in locator or re.match(r"[A-Za-z]:", locator):
        return None
    parts = origin_path.split("/")[:-1]
    for component in locator.split("/"):
        if component in {"", "."}:
            continue
        if component == "..":
            if not parts:
                return None
            parts.pop()
        else:
            parts.append(component)
    try:
        return _safe_path("/".join(parts + ["Cargo.toml"]), "dependency target")
    except EvidenceSchemaError as error:
        raise DependencySnapshotError("dependency target path is invalid") from error


@dataclass(frozen=True)
class GitDependencyContext:
    root: Path
    commit: str
    _files: Mapping[str, GitDependencyFile] = field(init=False, repr=False, compare=False)
    _package_names: dict[str, str] = field(default_factory=dict, init=False, repr=False, compare=False)

    def __post_init__(self) -> None:
        files = read_dependency_tree(self.root, self.commit)
        object.__setattr__(self, "_files", MappingProxyType({item.path: item for item in files}))

    def _file(self, path: str) -> GitDependencyFile:
        try:
            checked = _safe_path(path, "dependency context manifest")
        except EvidenceSchemaError as error:
            raise DependencySnapshotError("dependency context path is invalid") from error
        result = self._files.get(checked)
        if result is None:
            raise DependencySnapshotError("dependency manifest is absent from the selected tree")
        return result

    def _target(self, origin: ManifestDependencyOrigin) -> GitDependencyFile | None:
        source = origin.declaration.source
        if source is None or source.kind not in {"path", "registry", "git"}:
            raise DependencySnapshotError("dependency source kind is invalid")
        if source.kind != "path":
            return None
        path = _relative_target(origin)
        if path is None:
            return None
        target = self._file(path)
        name = self._package_name(target)
        if name != origin.declaration.package:
            raise DependencySnapshotError("dependency target package name does not match the declaration")
        return target

    def _package_name(self, target: GitDependencyFile) -> str:
        if target.path not in self._package_names:
            package = table(load(target.content, "dependency target").get("package"), "target.package")
            self._package_names[target.path] = text(package.get("name"), "target.package.name")
        return self._package_names[target.path]

    def bindings(self, manifest_path: str,
                 workspace_path: str | None = None) -> tuple[GitDependencyBinding, ...]:
        manifest = self._file(manifest_path)
        provider = None if workspace_path is None else self._file(workspace_path)
        origins = snapshot_manifest_origins(manifest, provider)
        return tuple(GitDependencyBinding(self.commit, origin, self._target(origin))
                     for origin in origins)
