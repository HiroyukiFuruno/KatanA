"""Synthetic render artifacts for release-validator unit tests only."""

import json
import struct
import subprocess
import zlib


def png_bytes(width=1280, height=900):
    def chunk(kind, payload):
        return struct.pack(">I", len(payload)) + kind + payload + struct.pack(">I", zlib.crc32(kind + payload))
    header = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    pixels = (b"\x00" + b"\x00" * (width * 3)) * height
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(pixels)) + chunk(b"IEND", b"")


def bind_pair(root, comparison, run, target, renderer, module, html=False):
    contract_path = root / comparison["contract"]
    contract = json.loads(contract_path.read_text())
    for side in ("reference", "measured"):
        path_value = contract["reference_artifact"] if side == "reference" else comparison["measured_artifact"]
        path = root / path_value
        manifest = json.loads(path.read_text())
        render_path = path.with_suffix(".png")
        render_path.write_bytes(png_bytes())
        manifest.update(viewport=contract["viewport"], producer_mode=renderer if side == "reference" else "packaged_main",
                        render={"path": render_path.relative_to(root).as_posix(), "sha256": module.sha256_bytes(render_path.read_bytes()), "pixel_ratio": 1})
        if html:
            manifest.update(active_toc="#s15", visible_section_state="#s15")
        if side == "measured":
            manifest["run_identity"] = {field: run[field] for field in ("run_id", "main_sha256", "sidecar_sha256", "fixture_sha256")}
            manifest["run_identity"]["target"] = target
            manifest.update(navigation=comparison["navigation"]) if html else manifest.update(missing_elements=comparison["missing_elements"])
        path.write_text(json.dumps(manifest))
        digest = module.sha256_bytes(path.read_bytes())
        comparison[side + "_sha256"] = digest
        if side == "reference":
            contract["reference_sha256"] = digest
        else:
            run["render_output"] = dict(manifest["render"], metrics_path=path_value, metrics_sha256=digest)
    contract_path.write_text(json.dumps(contract))
    comparison["contract_sha256"] = module.sha256_bytes(contract_path.read_bytes())


def bind_resource_cycle_artifact(root, record, module):
    run = record["packaged_run"]
    cold, warm, final = run["cold_rss_bytes"], run["after_close_rss_bytes"], run["after_close_rss_bytes"] + 100
    def idle(rss):
        return {"rss_bytes": rss, "physical_footprint_bytes": rss, "worker_count": 0, "frame_count": 0, "texture_count": 0, "cache_count": 0}
    cycles = []
    for cycle in range(1, 11):
        html = {"fixture_sha256": module.ORIGINAL_HTML_SHA256, "session_id": f"html-{cycle}", "generation": 1, "close_completed": True, "close_ms": 100, "snapshot": idle(cold + 100)}
        office = {"fixture_sha256": run["fixture_sha256"], "session_id": f"office-{cycle}", "generation": 1, "close_completed": True, "close_ms": 100, "snapshot": idle(final if cycle == 10 else warm)}
        cycles.append({"cycle": cycle, "opened_generations": [{"session_id": f"html-{cycle}", "generation": 1}, {"session_id": f"office-{cycle}", "generation": 1}], "closed_generations": [{"session_id": f"html-{cycle}", "generation": 1, "close_ms": 100}, {"session_id": f"office-{cycle}", "generation": 1, "close_ms": 100}], "html": html, "office": office})
    artifact = {"schema_version": 1, "run_identity": {field: run[field] for field in ("run_id", "main_sha256", "sidecar_sha256", "fixture_sha256")} | {"target": record["packaged_target"]}, "cold_snapshot": idle(cold), "warm_snapshot": idle(warm), "final_snapshot": idle(final), "cycles": cycles}
    artifact_path = root / "evidence-artifacts" / f"resource-cycle-{run['run_id']}.json"
    artifact_path.write_text(json.dumps(artifact), encoding="utf-8")
    run["resource_cycle_artifact"] = artifact_path.relative_to(root).as_posix()
    run["resource_cycle_sha256"] = module.sha256_bytes(artifact_path.read_bytes())


def bind_render_artifacts(root, evidence, module):
    target = "macos-arm64"
    if target not in evidence["packaged_targets"]:
        target = sorted(evidence["packaged_targets"])[0]
    html = evidence["html"]
    html["packaged_target"] = target
    html["packaged_run"] = dict(evidence["packaged_targets"][target], run_id="synthetic-html-run", fixture_sha256=module.ORIGINAL_HTML_SHA256)
    bind_pair(root, html["comparison"], html["packaged_run"], target, "chromeHTML", module, html=True)
    for record in evidence["office_fixtures"]:
        bind_pair(root, record["fidelity"], record["packaged_run"], record["packaged_target"], "sourceOffice", module)
        bind_resource_cycle_artifact(root, record, module)
    contracts = root / "scripts/release/document-fidelity-contracts"
    subprocess.run(["git", "-C", str(root), "add", "-f", str(contracts)], check=True, capture_output=True, env=module.git_environment())
    evidence["source_tree_sha256"] = module.source_tree_sha256(root)
