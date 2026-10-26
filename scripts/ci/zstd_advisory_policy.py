"""Exact, time-limited RUSTSEC-2026-0403 scope within the ZIP policy."""

from __future__ import annotations

import datetime as dt
import re

import zip_advisory_policy as zip_policy
from zip_advisory_policy import EXPIRES_ON


ADVISORY_ID = "RUSTSEC-2026-0403"
REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"
SELECTED_SAFE = "5.0.2+zstd.1.5.2"
SELECTED_ZSTD = "0.11.2+zstd.1.5.2"
ZIP_PARENTS = {("ironcalc", "0.8.3"), ("office2pdf", "0.8.1")}
REASON = (
    "KatanA issue https://github.com/HiroyukiFuruno/KatanA/issues/354; "
    "source audit of registry KDV 0.5.12 -> ironcalc 0.8.3 / office2pdf 0.8.1 -> "
    "zip 0.6.6 -> zstd 0.11.2+zstd.1.5.2 -> zstd-safe 5.0.2+zstd.1.5.2 "
    "found read::Decoder and write::Encoder, no affected write::Decoder caller; "
    "not proof of whole-program safety; "
    "openspec/changes/post-v0-22-41-document-fidelity-regressions/evidence/"
    "zstd-0403-risk-assessment-2026-10-26.ja.md; "
    f"expires {EXPIRES_ON.isoformat()} UTC, same existing operational deadline"
)


def _package_records(packages: list) -> dict:
    records = {}
    for package in packages:
        if (not isinstance(package, dict)
                or any(not isinstance(package.get(key), str) or not package[key]
                       for key in ("id", "name", "version"))
                or "source" not in package
                or (package["source"] is not None and not isinstance(package["source"], str))):
            raise ValueError("malformed package metadata")
        if package["id"] in records:
            raise ValueError("duplicate package ID")
        records[package["id"]] = package
    return records


def _node_edges(nodes: list, records: dict) -> dict:
    if not isinstance(nodes, list) or not nodes:
        raise ValueError("locked metadata resolve nodes are missing")
    edges = {}
    for node in nodes:
        if (not isinstance(node, dict) or not isinstance(node.get("id"), str)
                or node["id"] not in records or node["id"] in edges
                or not isinstance(node.get("deps"), list)):
            raise ValueError("duplicate or malformed resolve node")
        edges[node["id"]] = set()
        for dependency in node["deps"]:
            child = dependency.get("pkg") if isinstance(dependency, dict) else None
            if not isinstance(child, str) or child not in records:
                raise ValueError("dependency has no matching package")
            edges[node["id"]].add(child)
    if any(child not in edges for children in edges.values() for child in children):
        raise ValueError("dependency has no matching resolve node")
    return edges


def _workspace_roots(members: list, edges: dict) -> set[str]:
    if (not isinstance(members, list) or not members
            or any(not isinstance(member, str) or member not in edges for member in members)
            or len(set(members)) != len(members)):
        raise ValueError("malformed workspace members")
    return set(members)


def _graph(metadata: dict) -> tuple[dict, dict, set[str]]:
    if not isinstance(metadata, dict):
        raise ValueError("locked metadata must be an object")
    packages, resolve = metadata.get("packages"), metadata.get("resolve")
    if not isinstance(packages, list) or not packages or not isinstance(resolve, dict):
        raise ValueError("locked metadata packages or resolve are missing")
    records = _package_records(packages)
    edges = _node_edges(resolve.get("nodes"), records)
    return records, edges, _workspace_roots(metadata.get("workspace_members"), edges)


def _affected(version: str) -> bool:
    match = re.fullmatch(
        r"(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)"
        r"(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?", version
    )
    if not match:
        raise ValueError(f"unparseable zstd-safe version: {version}")
    return (4, 0, 0) <= tuple(map(int, match.groups())) < (8, 2, 0)


def _reachable(edges: dict, roots: set[str], excluded: set[str] = frozenset()) -> set[str]:
    seen, pending = roots - excluded, list(roots - excluded)
    while pending:
        for child in edges[pending.pop()]:
            if child not in seen and child not in excluded:
                seen.add(child)
                pending.append(child)
    return seen


def _affected_packages(records: dict) -> list:
    affected = []
    for package in records.values():
        if package["name"] == "zstd-safe":
            if package["source"] != REGISTRY:
                raise ValueError("zstd-safe package is not from crates.io")
            if _affected(package["version"]):
                affected.append(package)
    return affected


def _selected_legacy(records: dict, affected: list) -> tuple[str, str, str]:
    if len(affected) != 1 or affected[0]["version"] != SELECTED_SAFE:
        raise ValueError("affected zstd-safe is not the exact audited version")
    safe_id = affected[0]["id"]
    zstd = [pid for pid, p in records.items()
            if (p["name"], p["version"], p["source"]) == ("zstd", SELECTED_ZSTD, REGISTRY)]
    kdv = [pid for pid, p in records.items()
           if (p["name"], p["version"], p["source"]) == ("katana-document-viewer", "0.5.12", REGISTRY)]
    if len(zstd) != 1 or len(kdv) != 1:
        raise ValueError("audited registry zstd or KDV is missing or ambiguous")
    return safe_id, zstd[0], kdv[0]


def _parent_edges(records: dict, edges: dict) -> dict:
    parents = {pid: set() for pid in records}
    for parent, children in edges.items():
        for child in children:
            parents[child].add(parent)
    return parents


def _audited_route(records: dict, parents: dict, safe_id: str,
                   zstd_id: str, kdv_id: str) -> set[str]:
    if parents[safe_id] != {zstd_id}:
        raise ValueError("affected zstd-safe has an unapproved caller")
    zip_ids = parents[zstd_id]
    if not zip_ids or any(
            (records[pid]["name"], records[pid]["version"], records[pid]["source"])
            != ("zip", "0.6.6", REGISTRY) for pid in zip_ids):
        raise ValueError("old zstd has an unapproved ZIP caller")
    route = {safe_id, zstd_id, *zip_ids}
    for zip_id in zip_ids:
        if not parents[zip_id]:
            raise ValueError("old ZIP lacks the audited parent")
        for parent_id in parents[zip_id]:
            parent = records[parent_id]
            if ((parent["name"], parent["version"]) not in ZIP_PARENTS
                    or parent["source"] != REGISTRY or parents[parent_id] != {kdv_id}):
                raise ValueError("old ZIP has an unapproved parent or KDV caller")
            route.add(parent_id)
    return route


def validate_metadata(metadata: dict, today: dt.date) -> bool:
    """Reject any unaudited legacy route; return whether 0403 is needed."""
    if zip_policy.policy_expired(today):
        raise ValueError("ZIP/zstd advisory exception policy expired")
    records, edges, roots = _graph(metadata)
    zip_policy.validate_metadata(metadata)
    affected = _affected_packages(records)
    if not affected:
        return False
    safe_id, zstd_id, kdv_id = _selected_legacy(records, affected)
    parents = _parent_edges(records, edges)
    route = _audited_route(records, parents, safe_id, zstd_id, kdv_id)
    if route & roots:
        raise ValueError("affected zstd route is a direct workspace root")
    if not route <= _reachable(edges, roots):
        raise ValueError("affected zstd route is orphaned from workspace roots")
    if route & _reachable(edges, roots, {kdv_id}):
        raise ValueError("affected zstd route bypasses KDV")
    return True


def overlay_config(source: str, metadata: dict, today: dt.date) -> str:
    needed = validate_metadata(metadata, today)
    config = zip_policy.overlay_config(source)
    if not needed:
        return config
    section = re.search(r"(?ms)^\[advisories\]\s*(.*?)(?=^\[|\Z)", config)
    match = re.search(r"(?ms)^ignore\s*=\s*\[.*?^\]", section.group(1)) if section else None
    if not section or not match:
        raise ValueError("validated advisory ignore section is missing")
    end = section.start(1) + match.end() - 1
    entry = f'\n  {{ id = "{ADVISORY_ID}", reason = "{REASON}" }},\n'
    return config[:end] + entry + config[end:]
