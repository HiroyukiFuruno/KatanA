"""Manifest dependency snapshots and workspace inheritance."""

from __future__ import annotations

from downstream_dependency_records import (
    DependencyChange, DependencyDeclaration, DependencySnapshotError, DependencySource,
    changes, features, load, source, table, text,
)


def _entry_fields(scope: str, alias: str, value: object,
                  default_package: str, default_version: str | None) -> tuple[object, ...]:
    entry = table(value, f"{scope}.{alias}")
    package = text(entry.get("package", default_package), f"{scope}.{alias}.package")
    version = text(entry.get("version", default_version), f"{scope}.{alias}.version", optional=True)
    source_value = source(entry, f"{scope}.{alias}")
    feature_values = features(entry.get("features"), f"{scope}.{alias}.features")
    optional = entry.get("optional")
    default_features = entry.get("default-features")
    for field, field_value in (("optional", optional), ("default-features", default_features)):
        if field_value is not None and not isinstance(field_value, bool):
            raise DependencySnapshotError(f"{scope}.{alias}.{field} must be boolean")
    path_only = "path" in entry and not any(key in entry for key in ("git", "registry"))
    return package, version, source_value, feature_values, optional, default_features, path_only


def _workspace_dependencies(document: dict[str, object]) -> dict[str, object] | None:
    workspace = document.get("workspace")
    if workspace is None:
        return None
    dependencies = table(workspace, "workspace").get("dependencies")
    return None if dependencies is None else table(dependencies, "workspace.dependencies")


def _resolve_workspace_entry(alias: str, value: object,
                             workspace: dict[str, object] | None) -> object:
    if not isinstance(value, dict) or "workspace" not in value:
        return value
    if value["workspace"] is not True:
        raise DependencySnapshotError(f"dependency {alias}.workspace must be true")
    allowed = {"workspace", "features", "optional", "default-features"}
    if set(value) - allowed:
        raise DependencySnapshotError(f"dependency {alias} overrides inherited fields")
    if workspace is None or alias not in workspace:
        raise DependencySnapshotError(f"workspace dependency {alias} is missing")
    inherited = workspace[alias]
    merged = {"version": inherited} if isinstance(inherited, str) else dict(
        table(inherited, f"workspace.dependencies.{alias}")
    )
    if merged.get("workspace") is not None:
        raise DependencySnapshotError(f"workspace dependency {alias} is unresolved")
    root_features = features(merged.get("features"), f"workspace.dependencies.{alias}.features")
    member_features = features(value.get("features"), f"{alias}.features")
    if root_features or member_features:
        merged["features"] = list(dict.fromkeys(root_features + member_features))
    for key in ("optional", "default-features"):
        if key in value:
            merged[key] = value[key]
    return merged


def _declaration(scope: str, alias: str, value: object,
                 internal_names: frozenset[str]) -> DependencyDeclaration | None:
    default_package, default_version = alias, None
    if scope == "replace" and ":" in alias:
        default_package, default_version = alias.rsplit(":", 1)
    if isinstance(value, str):
        package, version = default_package, text(value, f"{scope}.{alias}.version")
        source_value = DependencySource("registry", "")
        feature_values, optional, default_features, path_only = (), None, None, False
    else:
        entry = table(value, f"{scope}.{alias}")
        if entry.get("workspace") is not None:
            raise DependencySnapshotError(f"{scope}.{alias}.workspace requires inherited context")
        package, version, source_value, feature_values, optional, default_features, path_only = _entry_fields(
            scope, alias, entry, default_package, default_version
        )
    if not package.startswith("katana-") or (package in internal_names and path_only):
        return None
    return DependencyDeclaration(scope, alias, package, version, source_value,
                                 feature_values, optional, default_features)


def _dependency_tables(document: dict[str, object]) -> list[tuple[str, dict[str, object]]]:
    tables = []
    for scope in ("dependencies", "dev-dependencies", "build-dependencies"):
        value = document.get(scope)
        if value is not None:
            tables.append((scope, table(value, scope)))
    workspace = document.get("workspace")
    if workspace is not None:
        value = table(workspace, "workspace").get("dependencies")
        if value is not None:
            tables.append(("workspace.dependencies", table(value, "workspace.dependencies")))
    targets = document.get("target")
    if targets is not None:
        for target, target_value in table(targets, "target").items():
            target_table = table(target_value, f"target.{target}")
            for scope in ("dependencies", "dev-dependencies", "build-dependencies"):
                value = target_table.get(scope)
                if value is not None:
                    tables.append((f"target.{target}.{scope}", table(value, scope)))
    for patch_name, patch_value in table(document.get("patch", {}), "patch").items():
        tables.append((f"patch.{patch_name}", table(patch_value, f"patch.{patch_name}")))
    replace = document.get("replace")
    if replace is not None:
        tables.append(("replace", table(replace, "replace")))
    return tables


def manifest_snapshot(content: str | bytes, internal_names: frozenset[str],
                      workspace_manifest: str | bytes | None = None) -> tuple[DependencyDeclaration, ...]:
    document = load(content, "manifest")
    workspace_document = load(workspace_manifest, "workspace manifest") if workspace_manifest is not None else document
    workspace = _workspace_dependencies(workspace_document)
    declarations = []
    for scope, dependency_table in _dependency_tables(document):
        for alias, value in dependency_table.items():
            resolved = _resolve_workspace_entry(alias, value, workspace)
            declaration = _declaration(scope, alias, resolved, internal_names)
            if declaration is not None:
                declarations.append(declaration)
    return tuple(sorted(declarations))


def classify_manifest_changes(old: str | bytes, new: str | bytes,
                              internal_names: frozenset[str],
                              old_workspace_manifest: str | bytes | None = None,
                              new_workspace_manifest: str | bytes | None = None) -> tuple[DependencyChange, ...]:
    before = manifest_snapshot(old, internal_names, old_workspace_manifest)
    after = manifest_snapshot(new, internal_names, new_workspace_manifest)
    return changes(before, after, lambda item: f"{item.scope}:{item.alias}")
