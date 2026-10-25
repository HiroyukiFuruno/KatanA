"""Keep immutable declaration provenance without granting internal exemption."""

from dataclasses import dataclass

from downstream_dependency_records import (
    DependencyDeclaration, DependencySnapshotError, load,
)
from downstream_evidence_fields import EvidenceSchemaError, _safe_path
from downstream_manifest_snapshot import (
    _declaration, _dependency_tables, _resolve_workspace_entry, _workspace_dependencies,
)
from git_dependency_tree import GitDependencyFile
from push_ref_updates import OID_RE


@dataclass(frozen=True)
class ManifestDependencyOrigin:
    declaration: DependencyDeclaration
    manifest_path: str
    manifest_blob: str
    origin_path: str
    origin_blob: str


def _manifest_record(record: GitDependencyFile) -> None:
    if not isinstance(record, GitDependencyFile):
        raise DependencySnapshotError("manifest origin requires a dependency-tree record")
    try:
        path = _safe_path(record.path, "manifest origin")
    except EvidenceSchemaError as error:
        raise DependencySnapshotError("manifest origin path is invalid") from error
    if path.rsplit("/", 1)[-1] != "Cargo.toml":
        raise DependencySnapshotError("manifest origin must identify Cargo.toml")
    if not isinstance(record.blob, str) or OID_RE.fullmatch(record.blob) is None:
        raise DependencySnapshotError("manifest origin requires a full blob OID")


def _origins(manifest: GitDependencyFile, document: dict[str, object],
             workspace: dict[str, object] | None,
             provider: GitDependencyFile) -> tuple[ManifestDependencyOrigin, ...]:
    result = []
    for scope, entries in _dependency_tables(document):
        for alias, value in entries.items():
            inherited = isinstance(value, dict) and value.get("workspace") is True
            resolved = _resolve_workspace_entry(alias, value, workspace)
            declaration = _declaration(scope, alias, resolved, frozenset())
            if declaration is None:
                continue
            origin = provider if inherited else manifest
            result.append(ManifestDependencyOrigin(
                declaration, manifest.path, manifest.blob, origin.path, origin.blob,
            ))
    return tuple(sorted(result, key=lambda item: item.declaration))


def snapshot_manifest_origins(manifest: GitDependencyFile,
                              workspace_manifest: GitDependencyFile | None = None
                              ) -> tuple[ManifestDependencyOrigin, ...]:
    """Retain candidate declarations; workspace membership is a separate gate."""
    _manifest_record(manifest)
    provider = manifest if workspace_manifest is None else workspace_manifest
    _manifest_record(provider)
    document = load(manifest.content, "manifest")
    workspace_document = load(provider.content, "workspace manifest")
    workspace = _workspace_dependencies(workspace_document)
    return _origins(manifest, document, workspace, provider)
