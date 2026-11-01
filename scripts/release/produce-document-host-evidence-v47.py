#!/usr/bin/env python3
"""Produce v47 published host evidence from observed private runs."""

import argparse
from pathlib import Path
import sys

from document_host_acceptance import APPROVAL, POLICY, PUBLISHED_V47_EVIDENCE_RELATIVE, assessments, provenance, verify_payload, verify_v47_timing
from document_host_io import decode, digest, fields, read, require, write_new
from document_host_publication import verify_download_receipt
from generated_asset_identity import module
from document_host_boundaries import validate_case


def produce(root: Path, run_map: Path, output: Path, receipt: Path) -> None:
    helper = module("_host_v47_pair_producer", "produce-document-host-evidence.py")
    mapping = fields(decode(read(run_map)), {"schema", "provenance", "bundle", "runs", "upstream_assessments"}, "private run map")
    require(mapping["schema"] == "katana-host-private-run-map-v1", "wrong private run-map schema")
    source = provenance(root, mapping["provenance"])
    bundle = fields(mapping["bundle"], {"target", "main_path", "worker_path", "main_sha256", "sidecar_sha256", "archive_sha256", "provenance"}, "actual bundle")
    require(bundle["provenance"] == source, "actual bundle source differs")
    require(helper.input_digest(Path(bundle["main_path"])) == bundle["main_sha256"]
            and helper.input_digest(Path(bundle["worker_path"])) == bundle["sidecar_sha256"], "actual executable bytes changed")
    require(output.is_relative_to(root) and "evidence" in output.relative_to(root).parts
            and "tmp" not in output.relative_to(root).parts and not output.exists(), "new public evidence directory required")
    require(receipt.is_relative_to(root), "public release receipt required")
    require(not (root / PUBLISHED_V47_EVIDENCE_RELATIVE).exists(), "refusing to overwrite existing v47 evidence")
    receipt_ref = {"path": receipt.relative_to(root).as_posix(), "sha256": digest(read(receipt))}
    verify_download_receipt(root, {"status": "published_macos_host_accepted", "receipt": receipt_ref},
                            source, bundle, "v0.22.47")
    assessments(root, mapping["upstream_assessments"])
    require(isinstance(mapping["runs"], list) and len(mapping["runs"]) == 7, "seven actual pair runs required")
    producer_sha = digest(read(Path(__file__)))
    transcripts = [helper.pair(root, item, source, bundle, producer_sha) for item in mapping["runs"]]
    for entry, transcript in zip(mapping["runs"], transcripts, strict=True):
        validate_boundaries(helper, entry, transcript)
    validate_batch(transcripts, helper.legacy().SUPPLIED_OFFICE_FIXTURES)
    write_evidence(root, output, receipt_ref, mapping, bundle, source, producer_sha, transcripts)


def validate_boundaries(helper, entry: dict, transcript: dict) -> None:
    run = helper.PrivateRun(Path(entry["directory"]))
    for case in transcript["cases"]:
        label = f"{entry['prefix']}-cycle{case['cycle']:02}-{case['kind']}"
        validate_case(run, label, case, transcript)
    transcript["raw_artifacts"] = sorted(set(transcript["raw_artifacts"]) | run.inventory)


def validate_batch(transcripts: list[dict], fixtures: dict) -> None:
    inputs = {item["input_sha256"] for item in transcripts}
    require(inputs == set(fixtures) and len(inputs) == 7, "actual Office inputs missing or duplicated")
    require(len({item["run_id"] for item in transcripts}) == 7, "actual run reused")
    require(len({(item["pid"], item["start_identity"]) for item in transcripts}) == 7, "actual process reused")
    operations = []
    for item in transcripts:
        verify_v47_timing(item)
        ops = item["operations"]
        name = {"docx": "page", "xlsx": "sheet", "pptx": "slide"}[fixtures[item["input_sha256"]]]
        require(any(op["kind"] == "office" and op["name"] in {name, "single_item"} for op in ops),
                "Office navigation operation missing")
        operations.extend(ops)
    require({"scroll", "reload", "dirty_source"}.issubset({op["name"] for op in operations if op["kind"] == "html"}),
            "HTML operations missing")


def write_evidence(root: Path, output: Path, receipt_ref: dict, mapping: dict, bundle: dict,
                   source: dict, producer_sha: str, transcripts: list[dict]) -> None:
    helper = module("_host_v47_output_producer", "produce-document-host-evidence.py")
    refs = []
    for index, transcript in enumerate(transcripts, 1):
        path = output / f"run-{index:02}.json"
        ref = write_new(path, transcript)
        ref["path"] = path.relative_to(root).as_posix()
        refs.append(ref)
    bundle_public = {key: bundle[key] for key in ("target", "main_sha256", "sidecar_sha256", "archive_sha256", "provenance")}
    bundle_public["schema"] = "katana-host-bundle-v1"
    bundle_ref = write_new(output / "bundle.json", bundle_public)
    bundle_ref["path"] = (output / "bundle.json").relative_to(root).as_posix()
    evidence = {"schema": "katana-host-acceptance-v1", "version": "v0.22.47", "scope": "katana-host", "policy": POLICY,
                "approval_sha256": digest(read(root / APPROVAL)), "phase": "postpublication", "claim": "published_macos_host_accepted",
                "provenance": source, "candidate": {"target": bundle["target"], "main_sha256": bundle["main_sha256"],
                "sidecar_sha256": bundle["sidecar_sha256"], "archive_sha256": bundle["archive_sha256"], "bundle_manifest": bundle_ref},
                "transcripts": refs, "upstream_assessments": mapping["upstream_assessments"],
                "other_targets": {target: "document_qa_not_run" for target in helper.legacy().SUPPORTED_TARGETS - {bundle["target"]}},
                "producer_sha256": producer_sha, "publication": {"status": "published_macos_host_accepted", "receipt": receipt_ref}}
    provenance(root, source)
    require(helper.input_digest(Path(bundle["main_path"])) == bundle["main_sha256"]
            and helper.input_digest(Path(bundle["worker_path"])) == bundle["sidecar_sha256"], "actual executable bytes changed")
    canonical = root / PUBLISHED_V47_EVIDENCE_RELATIVE
    require(canonical.resolve() == canonical and not canonical.is_symlink(), "canonical path escapes repository")
    verify_payload(root, evidence, phase="postpublication", publication_version="v0.22.47")
    write_new(canonical, evidence)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--run-map", type=Path, required=True)
    parser.add_argument("--output-directory", type=Path, required=True)
    parser.add_argument("--publication-receipt", type=Path, required=True)
    args = parser.parse_args()
    try:
        produce(args.root.resolve(), args.run_map, args.output_directory.resolve(), args.publication_receipt.resolve())
    except (ValueError, RuntimeError, OSError, KeyError, TypeError) as error:
        print(f"ERROR: v47 host evidence producer: {error}", file=sys.stderr)
        return 1
    print("OK: actual v47 public host evidence validated")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
