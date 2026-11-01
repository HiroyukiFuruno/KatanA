from __future__ import annotations

import datetime as dt
import re
import tomllib
from pathlib import Path

EXPIRES_ON = dt.date(2026, 11, 7)
ADVISORIES = ("RUSTSEC-2026-0367", "RUSTSEC-2026-0368", "RUSTSEC-2026-0369")
OLD_ZIP_PARENTS = {
    ("0.6.6", "ironcalc", "0.8.3"), ("0.6.6", "office2pdf", "0.8.1"),
    ("2.6.1", "umya-spreadsheet", "2.3.3"), ("8.6.0", "docx-rs", "0.4.22"),
    ("8.6.0", "katana-document-viewer", "0.5.12"),
}
SAFE_ZIP = ("9.0.5", "katana-core", "0.22.42")
SAFE_ZIP_EDGES = frozenset(
    (
        SAFE_ZIP,
        ("9.1.2", "katana-core", "0.22.42"),
        ("9.1.3", "katana-core", "0.22.43"),
        ("9.1.3", "katana-core", "0.22.44"),
        ("9.1.3", "katana-core", "0.22.45"),
        ("9.2.0", "katana-core", "0.22.45"),
        ("9.2.0", "katana-core", "0.22.46"),
        ("9.2.1", "katana-core", "0.22.47"),
    )
)
SAFE_ZIP_MANIFEST = (
    Path(__file__).resolve().parents[2] / "crates" / "katana-core" / "Cargo.toml"
).resolve()
REASON = ("KatanA issue https://github.com/HiroyukiFuruno/KatanA/issues/354 source audit found only in-memory XML/asset reads and no vulnerable ZipArchive::extract or ZipStreamReader::extract use in the checked KDV/parent crates; not proof of whole-program safety; openspec/changes/post-v0-22-41-document-fidelity-regressions/evidence/zip-risk-acceptance-2026-10-24.ja.md; expires 2026-11-07 UTC")


def policy_expired(today: dt.date) -> bool:
    return today >= EXPIRES_ON


def validate_policy_ids(ids: tuple[str, ...]) -> None:
    if ids != ADVISORIES:
        raise ValueError(f"unexpected advisory IDs: {ids}")


def _packages(metadata: dict) -> dict[str, tuple[str, str]]:
    packages = metadata.get("packages")
    if not isinstance(packages, list):
        raise ValueError("locked metadata packages must be a list")
    result = {}
    for package in packages:
        if not isinstance(package, dict) or not all(k in package for k in ("id", "name", "version")):
            raise ValueError("malformed package metadata")
        if package["id"] in result:
            raise ValueError(f"duplicate package ID: {package['id']}")
        result[package["id"]] = (package["name"], package["version"])
    return result


def _graph_shape(metadata: dict) -> tuple[dict, list, set[str]]:
    if not metadata.get("packages") or not metadata.get("resolve", {}).get("nodes"):
        raise ValueError("locked metadata is missing packages or resolve nodes")
    packages, resolve = _packages(metadata), metadata["resolve"]
    nodes = resolve["nodes"]
    if not isinstance(nodes, list):
        raise ValueError("locked metadata resolve.nodes must be a list")
    node_ids = {node.get("id") for node in nodes if isinstance(node, dict)}
    if len(node_ids) != len(nodes) or not node_ids <= packages.keys():
        raise ValueError("duplicate or malformed resolve node")
    members = metadata.get("workspace_members")
    if not isinstance(members, list) or not members or not set(members) <= node_ids:
        raise ValueError("workspace member is missing from resolve nodes")
    return packages, nodes, node_ids


def _collect_edges(metadata: dict, packages: dict, nodes: list, node_ids: set[str]) -> tuple[set, dict, dict]:
    parents, edges = set(), {node_id: set() for node_id in node_ids}
    zip_edges, records = {}, {p["id"]: p for p in metadata["packages"]}
    for node in nodes:
        parent = packages.get(node.get("id"))
        dependencies = node.get("deps")
        if not parent or not isinstance(dependencies, list):
            raise ValueError(f"malformed resolve node: {node.get('id')!r}")
        for dependency in dependencies:
            child_id = dependency.get("pkg") if isinstance(dependency, dict) else None
            if child_id not in node_ids:
                raise ValueError(f"dependency has no matching node: {dependency!r}")
            child = packages[child_id]
            edges[node["id"]].add(child_id)
            if child[0] == "zip":
                if records[child_id].get("source") != "registry+https://github.com/rust-lang/crates.io-index":
                    raise ValueError(f"ZIP package is not from crates.io: {child_id}")
                item = (child[1], parent[0], parent[1])
                parents.add(item); zip_edges[(node["id"], child_id)] = item
                if item in OLD_ZIP_PARENTS and records[node["id"]].get("source") != "registry+https://github.com/rust-lang/crates.io-index":
                    raise ValueError(f"ZIP parent is not from crates.io: {node['id']}")
    return parents, edges, zip_edges


def _validate_ancestry(metadata: dict, packages: dict, edges: dict, zip_edges: dict, parents: set) -> None:
    for item in sorted(parents):
        if item not in SAFE_ZIP_EDGES and item not in OLD_ZIP_PARENTS:
            raise ValueError(f"unapproved ZIP edge: {item}")
    records = {p["id"]: p for p in metadata["packages"]}
    kdv = [p_id for p_id, (name, version) in packages.items() if name == "katana-document-viewer" and version == "0.5.12"]
    if any(records[p_id].get("source") != "registry+https://github.com/rust-lang/crates.io-index" for p_id in kdv):
        raise ValueError("KDV 0.5.12 is not from crates.io")
    if any(edge in OLD_ZIP_PARENTS for edge in parents) and not kdv:
        raise ValueError("old ZIP edge lacks KDV 0.5.12 ancestry")
    roots = set(metadata["workspace_members"])
    reachable, frontier = set(roots), list(roots)
    while frontier:
        for child in edges.get(frontier.pop(), ()):
            if child not in reachable:
                reachable.add(child); frontier.append(child)
    if any(kdv_id not in reachable for kdv_id in kdv):
        raise ValueError("KDV 0.5.12 is not reachable from workspace members")
    blocked, frontier = roots - set(kdv), list(roots - set(kdv))
    while frontier:
        for child in edges.get(frontier.pop(), ()):
            if child not in blocked and child not in kdv:
                blocked.add(child); frontier.append(child)
    for (parent_id, child_id), item in zip_edges.items():
        if item in SAFE_ZIP_EDGES:
            parent = records[parent_id]
            manifest_path = parent.get("manifest_path")
            if (parent.get("source") is not None or not isinstance(manifest_path, str)
                    or Path(manifest_path).resolve() != SAFE_ZIP_MANIFEST):
                raise ValueError("approved ZIP edge parent is not the local katana-core manifest")
            if parent_id not in reachable or child_id not in reachable:
                raise ValueError("approved ZIP edge is orphaned from workspace roots")
        if item not in OLD_ZIP_PARENTS:
            continue
        if parent_id not in reachable or child_id not in reachable:
            raise ValueError("old ZIP route is orphaned from workspace roots")
        if parent_id in blocked or child_id in blocked:
            raise ValueError("old ZIP route bypasses KDV")


def validate_metadata(metadata: dict) -> None:
    packages, nodes, node_ids = _graph_shape(metadata)
    for name, version in {"katana-document-viewer": "0.5.12", "katana-render-runtime": "0.4.23"}.items():
        found = {v for n, v in packages.values() if n == name}
        if found and found != {version}:
            raise ValueError(f"unexpected {name} version(s): {sorted(found)}")
    parents, edges, zip_edges = _collect_edges(metadata, packages, nodes, node_ids)
    _validate_ancestry(metadata, packages, edges, zip_edges, parents)


def overlay_config(source: str) -> str:
    validate_policy_ids(ADVISORIES)
    try:
        tomllib.loads(source)
    except tomllib.TOMLDecodeError as error:
        raise ValueError(f"invalid deny.toml: {error}") from error
    section = re.search(r"(?ms)^\[advisories\]\s*(.*?)(?=^\[|\Z)", source)
    if not section:
        raise ValueError("[advisories] section is missing")
    match = re.search(r"(?ms)^ignore\s*=\s*\[.*?^\]", section.group(1))
    if not match:
        raise ValueError("deny.toml advisories.ignore list is missing")
    entries = "\n".join(f'  {{ id = "{advisory}", reason = "{REASON}" }},' for advisory in ADVISORIES)
    end = section.start(1) + match.end()
    return source[: end - 1] + "\n" + entries + "\n" + source[end - 1 :]
