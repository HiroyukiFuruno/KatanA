"""Register package path/name/version identities from one actual Git commit."""

from collections.abc import Mapping
from dataclasses import dataclass, field
from pathlib import Path
from types import MappingProxyType
import time

from bounded_git_output import bounded_git
from cargo_workspace_membership import workspace_owners
from cargo_version_contract import validate_versions
from downstream_dependency_records import DependencySnapshotError, load, table, text
from downstream_evidence_fields import SEMVER_RE, _safe_path
from git_dependency_tree import (DEPENDENCY_TREE_TIMEOUT_SECONDS, GitDependencyFile,
                                 read_dependency_tree)
from git_dependency_context import relative_manifest_path
from manifest_dependency_origins import ManifestDependencyOrigin, snapshot_manifest_origins
from push_ref_updates import OID_RE


def _directories(root, commit, deadline):
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise DependencySnapshotError("workspace directory discovery exceeded its deadline")
    listing = bounded_git(root, ["ls-tree", "-r", "-d", "-z", "--full-tree", commit],
                          stdout_limit=2_097_152, timeout=remaining)
    if listing and not listing.endswith(b"\0"):
        raise DependencySnapshotError("workspace directory tree listing is incomplete")
    result = {""}
    for entry in listing.split(b"\0")[:-1]:
        metadata, separator, path = entry.partition(b"\t")
        fields = metadata.split(b" ")
        if not separator or len(fields) != 3 or fields[:2] != [b"040000", b"tree"]:
            raise DependencySnapshotError("workspace directory tree metadata is invalid")
        try:
            name, oid = path.decode("utf-8"), fields[2].decode("ascii")
        except UnicodeError as error:
            raise DependencySnapshotError("workspace directory identity is invalid") from error
        if OID_RE.fullmatch(oid) is None:
            raise DependencySnapshotError("workspace directory requires a full tree identity")
        result.add(_safe_path(name, "workspace directory"))
    return result


@dataclass(frozen=True)
class RegisteredPackage:
    manifest_path: str
    manifest_blob: str
    workspace_path: str
    workspace_blob: str
    name: str
    version: str


@dataclass(frozen=True)
class RegisteredDependency:
    origin: ManifestDependencyOrigin
    internal: RegisteredPackage | None


def _version(document, workspace):
    version = table(document["package"], "package").get("version", "0.0.0")
    if isinstance(version, dict):
        if set(version) != {"workspace"} or version["workspace"] is not True:
            raise DependencySnapshotError("package version inheritance is invalid")
        definition = table(workspace.get("workspace"), "workspace")
        version = table(definition.get("package"), "workspace.package").get("version")
    version = text(version, "package.version")
    if SEMVER_RE.fullmatch(version) is None:
        raise DependencySnapshotError("registered package version must be SemVer")
    return version


@dataclass(frozen=True)
class GitWorkspaceContext:
    root: Path
    commit: str
    files: Mapping[str, GitDependencyFile] = field(init=False, repr=False, compare=False)
    owners: Mapping[str, str] = field(init=False, repr=False, compare=False)
    packages: Mapping[str, RegisteredPackage] = field(init=False, repr=False, compare=False)
    _documents: Mapping[str, dict[str, object]] = field(init=False, repr=False, compare=False)

    def __post_init__(self):
        deadline = time.monotonic() + DEPENDENCY_TREE_TIMEOUT_SECONDS
        files = {item.path: item for item in read_dependency_tree(self.root, self.commit)}
        documents = {path: load(item.content, path) for path, item in files.items() if path.endswith("Cargo.toml")}
        object.__setattr__(self, "files", MappingProxyType(files))
        object.__setattr__(self, "_documents", MappingProxyType(documents))
        directories = _directories(self.root, self.commit, deadline)
        object.__setattr__(self, "owners", MappingProxyType(workspace_owners(documents, directories)))
        packages = {path: self._package(path, owner) for path, owner in self.owners.items()}
        object.__setattr__(self, "packages", MappingProxyType(packages))

    def _package(self, path, owner):
        package = table(self._documents[path]["package"], "package")
        return RegisteredPackage(path, self.files[path].blob, owner, self.files[owner].blob,
                                 text(package.get("name"), "package.name"),
                                 _version(self._documents[path], self._documents[owner]))

    def origins(self, path):
        owner = self.owners.get(path)
        if path not in self._documents:
            raise DependencySnapshotError("manifest is absent from the selected tree")
        if owner is None and "workspace" in self._documents[path]:
            owner = path
        if owner is None:
            return snapshot_manifest_origins(self.files[path])
        return snapshot_manifest_origins(self.files[path], self.files[owner])

    def _internal_target(self, origin):
        if origin.manifest_path not in self.owners and "workspace" not in self._documents[origin.manifest_path]:
            return None
        source = origin.declaration.source
        if source is None or source.kind != "path":
            return None
        path = relative_manifest_path(origin.origin_path, source.locator)
        package = self.packages.get(path)
        if package is None or package.name != origin.declaration.package:
            return None
        return package

    def bindings(self, path):
        bindings = tuple(RegisteredDependency(origin, self._internal_target(origin))
                         for origin in self.origins(path))
        validate_versions([{"requirement": item.origin.declaration.version, "version": item.internal.version}
                           for item in bindings if item.internal is not None
                           and item.origin.declaration.version is not None])
        return bindings
