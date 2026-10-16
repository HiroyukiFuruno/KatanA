"""Validated observations remain author-writable diagnostics, not execution proof."""

import hashlib

from native_test_inventory_record import OID_RE, SHA256_RE
from native_test_run_recording import cargo_command
from native_test_run_identity import INPUT_ENVIRONMENT, LOCK_PATHS
from native_test_run_selection import selected_listing


def _strings(value: object, *, nonempty: bool = False) -> bool:
    return (isinstance(value, list) and (bool(value) or not nonempty)
            and all(isinstance(item, str) and bool(item) for item in value))


def artifact_key(artifact: dict) -> tuple:
    target = artifact["target"]
    required = (artifact.get("package_id"), artifact.get("manifest_path"),
                target.get("name"), target.get("src_path"), artifact.get("executable"))
    if (not all(isinstance(item, str) and bool(item) for item in required)
            or not _strings(target.get("kind"), nonempty=True)
            or not _strings(target.get("crate_types"), nonempty=True)
            or not _strings(artifact.get("features"))):
        raise ValueError("native artifact identity is incomplete")
    return (artifact["package_id"], target["name"], tuple(target["kind"]),
            target["src_path"], artifact["manifest_path"], tuple(target["crate_types"]))


def _source(run: dict) -> None:
    source = run["source_before"]
    if (run.get("source_stable") is not True or source != run.get("source_after")
            or source.get("clean") is not True or set(source) != {
                "head", "tree", "clean", "status_sha256", "locks"}):
        raise ValueError("source was not stable and clean during the observed invocation")
    if (not isinstance(source.get("head"), str) or not OID_RE.fullmatch(source["head"])
            or not isinstance(source.get("tree"), str) or not OID_RE.fullmatch(source["tree"])
            or not isinstance(source.get("status_sha256"), str)
            or source["status_sha256"] != hashlib.sha256(b"").hexdigest()):
        raise ValueError("source identity is incomplete")
    locks = source.get("locks")
    if not isinstance(locks, dict) or set(locks) != set(LOCK_PATHS) or any(
            not isinstance(key, str) or not isinstance(value, str)
            or not SHA256_RE.fullmatch(value) for key, value in locks.items()):
        raise ValueError("source lock identities are incomplete")


def _execution(run: dict, kind: str) -> list[str]:
    if (type(run.get("schema")) is not int or run["schema"] != 1 or run.get("kind") != kind
            or not isinstance(run.get("run_id"), str) or not run["run_id"]
            or type(run.get("exit_code")) is not int or run.get("complete") is not False
            or run.get("reuse_authorized") is not False):
        raise ValueError("observed invocation schema/kind/status is invalid")
    requested, executed = run["requested_argv"], run["executed_argv"]
    if not _strings(requested, nonempty=True) or not _strings(executed, nonempty=True):
        raise ValueError("observed Cargo argv is invalid")
    if cargo_command(requested, kind) != executed:
        raise ValueError("observed Cargo argv does not match the requested invocation")
    return executed


def _listings(run: dict, argv: list[str]) -> tuple[set, dict]:
    cargo = run["cargo"]
    if (cargo.get("native_artifacts_complete") is not True or cargo.get("reasons") != []
            or run.get("listing_reasons") != []):
        raise ValueError("native artifact or listing collection is incomplete")
    emitted, records = cargo["artifacts"], run["native_listings"]
    if (not isinstance(emitted, list) or not emitted or not isinstance(records, list)
            or len(emitted) > 4096):
        raise ValueError("native artifact collection is invalid or oversized")
    by_key, shallow = {}, set()
    for artifact in emitted:
        key = artifact_key(artifact)
        if key[:3] in shallow:
            raise ValueError("duplicate or conflicting native artifact identity")
        shallow.add(key[:3])
        by_key[key] = artifact
    return _bound_listings(by_key, records, argv)


def _bound_listings(emitted: dict, records: list, argv: list[str]) -> tuple[set, dict]:
    candidates, observations = set(), {}
    for record in records:
        artifact = record["artifact"]
        key = artifact_key(artifact)
        if key in observations or emitted.get(key) != artifact:
            raise ValueError("native listing is duplicated or not bound to the Cargo artifact")
        executable = artifact["executable"]
        if (record["all"].get("argv") != [executable, "--list"]
                or record["ignored"].get("argv") != [executable, "--list", "--ignored"]):
            raise ValueError("native listing argv is not bound to its artifact and mode")
        digest = record.get("binary_sha256_at_listing")
        if not isinstance(digest, str) or not SHA256_RE.fullmatch(digest):
            raise ValueError("native binary listing digest is missing")
        candidates.update((key, name, kind) for name, kind in selected_listing(record, argv))
        observations[key] = _listing_observation(record, artifact, digest)
    if set(observations) != set(emitted):
        raise ValueError("native listing is missing for an emitted Cargo artifact")
    return candidates, observations


def _listing_observation(record: dict, artifact: dict, digest: str) -> dict:
    return {"features": artifact["features"], "binary_sha256_at_listing": digest,
            "executable": artifact["executable"], "ignored": record["ignored"]["cases"],
            "all_raw_sha256": record["all"]["raw_sha256"],
            "all_footer": record["all"]["footer"], "all_cases": record["all"]["cases"],
            "ignored_raw_sha256": record["ignored"]["raw_sha256"],
            "ignored_footer": record["ignored"]["footer"]}


def observe_run(run: dict, kind: str) -> tuple[set, dict]:
    argv = _execution(run, kind)
    _source(run)
    _inputs(run.get("inputs"))
    return _listings(run, argv)


def _inputs(inputs: dict) -> None:
    if not isinstance(inputs, dict) or set(inputs) != {
            "os", "machine", "rustc", "cargo", "environment", "worker_sha256"}:
        raise ValueError("observed invocation conditions are missing or unsupported")
    if any(not isinstance(inputs[key], str) or not inputs[key]
           for key in ("os", "machine", "rustc", "cargo")):
        raise ValueError("observed platform/toolchain conditions are incomplete")
    environment = inputs["environment"]
    if (not isinstance(environment, dict) or set(environment) != set(INPUT_ENVIRONMENT) or any(
            not isinstance(key, str) or not (value is None or isinstance(value, str))
            for key, value in environment.items())):
        raise ValueError("observed environment conditions are invalid")
    worker = inputs["worker_sha256"]
    if worker is not None and (not isinstance(worker, str) or not SHA256_RE.fullmatch(worker)):
        raise ValueError("observed worker digest is invalid")
