"""固定版のhost admission。strict全体受入および独立95達成とは別の判定を返す。"""

from __future__ import annotations

import subprocess
import re
from pathlib import Path

from document_host_io import decode, digest, fields, legacy, read, reference, require, sha
from document_host_transcript import verify as verify_transcript

VERSION = "v0.22.42"
SCOPE = "katana-host"
POLICY = "2026-10-24-katana-host-advance-publication"
APPROVAL_SHA256 = "42701122e21f9c3af53cf09d54eb03dd212677a832143b7371c3e090177a4bcd"
EVIDENCE_RELATIVE = ("openspec/changes/post-v0-22-41-document-fidelity-regressions/evidence/"
                     "document-host-acceptance-v0.22.42.json")
PUBLISHED_EVIDENCE_RELATIVE = ("openspec/changes/post-v0-22-41-document-fidelity-regressions/evidence/"
                               "document-host-published-acceptance-v0.22.42.json")
APPROVAL = ("openspec/changes/post-v0-22-41-document-fidelity-regressions/evidence/"
            "release-goal-2026-10-24.ja.md")
MANIFEST = {"schema", "version", "scope", "policy", "approval_sha256", "phase", "claim",
            "provenance", "candidate", "transcripts", "upstream_assessments", "other_targets",
            "producer_sha256", "publication"}
PROVENANCE = {"source_commit", "source_tree_sha256", "cargo_lock_sha256", "published_dependencies"}
ASSESSMENT = {"owner", "issue", "status", "score", "observation", "measurement"}
ISSUES = {
    "chrome_html": ("KRR", "https://github.com/HiroyukiFuruno/katana-render-runtime/issues/106"),
    # Officeの包括免除は作らない。採点未完は未完として残す。
    "office_independent_95": ("KDV", "https://github.com/HiroyukiFuruno/katana-document-viewer/issues/58"),
}


def provenance(root: Path, value: object) -> dict:
    record = fields(value, PROVENANCE, "provenance")
    helper = legacy()
    require(isinstance(record["source_commit"], str) and re.fullmatch(r"[0-9a-f]{40}", record["source_commit"]) is not None,
            "host evidence actual build commit is required")
    require(sha(record["source_tree_sha256"], "source tree") == helper.source_tree_sha256(root),
            "host evidence source tree is stale")
    recorded_object = subprocess.run(["git", "-C", str(root), "cat-file", "-e", record["source_commit"]+"^{commit}"],
                                     check=False, capture_output=True, env=helper.git_environment())
    require(recorded_object.returncode == 0, "recorded source commit object unavailable; supply actual build history before validation")
    same_committed_source = subprocess.run(["git", "-C", str(root), "diff", "--quiet", record["source_commit"],
                                           "HEAD", "--", *helper.SOURCE_ROOTS], check=False, env=helper.git_environment())
    require(same_committed_source.returncode == 0, "recorded source commit differs from current production source")
    require(sha(record["cargo_lock_sha256"], "Cargo.lock") == digest(read(root / "Cargo.lock")),
            "host evidence lockfile is stale")
    packages = helper.lock_packages(root)
    dependencies = record["published_dependencies"]
    require(isinstance(dependencies, dict) and set(dependencies) == helper.RENDERER_PACKAGES,
            "host evidence published registry graph is incomplete")
    for name, package in packages.items():
        expected = {key: package[key] for key in ("version", "source", "checksum")}
        require(dependencies[name] == expected, "host dependency version/source/checksum differs from lock")
    dirty = subprocess.run(["git", "-C", str(root), "diff", "--quiet", "HEAD", "--", *helper.SOURCE_ROOTS],
                           check=False, env=helper.git_environment())
    require(dirty.returncode == 0, "host acceptance requires unchanged production source")
    tracked = subprocess.run(["git", "-C", str(root), "ls-files", "--others", "--exclude-standard", "--", *helper.SOURCE_ROOTS],
                             check=True, capture_output=True, text=True, env=helper.git_environment())
    require(not any(not any(part in helper.EXCLUDED_PARTS for part in Path(path).parts)
                    for path in tracked.stdout.splitlines()), "host source contains uncommitted production files")
    return record


def same_source_content(left: dict, right: dict) -> bool:
    return all(left.get(key) == right.get(key) for key in PROVENANCE - {"source_commit"})


def assessments(root: Path, value: object) -> None:
    records = fields(value, set(ISSUES), "upstream assessments")
    for kind, (owner, issue) in ISSUES.items():
        record = fields(records[kind], ASSESSMENT, kind)
        require(record["owner"] == owner and record["issue"] == issue, "unknown owner/Issue cannot defer a host bug")
        status = record["status"]
        require(status in {"not_evaluated", "failed", "diverged"}, "host admission cannot manufacture a fidelity pass")
        require(record["observation"] in {"reference_unavailable", "comparison_failed", "measured_difference", "independent_95_incomplete"},
                "unknown upstream assessment")
        if status == "not_evaluated":
            require(record["measurement"] is None and record["score"] is None,
                    "unmeasured comparison must not synthesize a score/artifact")
            require(record["observation"] in {"reference_unavailable", "independent_95_incomplete"},
                    "unmeasured comparison cannot claim a measured difference")
        else:
            _, measured = reference(root, record["measurement"], "upstream measured artifact")
            fields(measured, {"schema", "assessment", "status", "score", "input_sha256", "reference_sha256", "measured_sha256"}, "upstream artifact")
            require(measured["schema"] == "katana-upstream-assessment-v1" and measured["assessment"] == kind
                    and measured["status"] == status and measured["score"] == record["score"],
                    "upstream measured artifact does not support the stated result")
            for field in ("input_sha256", "reference_sha256", "measured_sha256"):
                sha(measured[field], f"upstream {field}")
            require(record["observation"] == ("comparison_failed" if status == "failed" else "measured_difference"),
                    "comparison failed and measured divergence are different states")
            if record["score"] is not None:
                from document_host_io import number
                require(number(record["score"], "upstream score") < 95, "independent 95 claim needs strict evidence")


def verify(root: Path, evidence_path: Path | None = None, *, phase: str = "prepublish") -> dict:
    require(phase in {"prepublish", "postpublication"}, "unknown host acceptance phase")
    expected_path = root / (EVIDENCE_RELATIVE if phase == "prepublish" else PUBLISHED_EVIDENCE_RELATIVE)
    path = evidence_path or expected_path
    require(path.resolve() == expected_path.resolve() and not path.is_symlink(), "host schema has its own canonical path")
    evidence = fields(decode(read(path)), MANIFEST, "host admission")
    require(evidence["schema"] == "katana-host-acceptance-v1" and evidence["version"] == VERSION
            and evidence["scope"] == SCOPE and evidence["policy"] == POLICY, "host approval is fixed to v0.22.42 katana-host")
    require(evidence["approval_sha256"] == APPROVAL_SHA256 == digest(read(root / APPROVAL)), "host approval text changed")
    require(evidence["phase"] == phase, "candidate QA cannot satisfy post-publication acceptance")
    expected_claim = "candidate_host_accepted_postpublication_pending" if phase == "prepublish" else "published_macos_host_accepted"
    require(evidence["claim"] == expected_claim, "host evidence cannot claim full fidelity/goal completion")
    source = provenance(root, evidence["provenance"])
    helper = legacy()
    bundle = fields(evidence["candidate"], {"target", "bundle_manifest", "main_sha256", "sidecar_sha256", "archive_sha256"}, "host bundle")
    require(bundle["target"] in {"macos-arm64", "macos-x86_64"}, "host evidence must name its real Mac target")
    main_sha, worker_sha = sha(bundle["main_sha256"], "main"), sha(bundle["sidecar_sha256"], "worker")
    _, bundle_manifest = reference(root, bundle["bundle_manifest"], "host actual bundle manifest")
    fields(bundle_manifest, {"schema", "target", "provenance", "main_sha256", "sidecar_sha256", "archive_sha256"}, "host bundle manifest")
    require(bundle_manifest.get("schema") == "katana-host-bundle-v1"
            and bundle_manifest.get("provenance") == source and bundle_manifest.get("main_sha256") == main_sha
            and bundle_manifest.get("sidecar_sha256") == worker_sha and bundle_manifest.get("target") == bundle["target"]
            and bundle_manifest.get("archive_sha256") == sha(bundle["archive_sha256"], "host archive"),
            "candidate bundle identity/source not bound")
    producer_sha = sha(evidence["producer_sha256"], "producer")
    require(producer_sha == digest(read(root / "scripts/release/produce-document-host-evidence.py")), "producer hash stale")
    records = evidence["transcripts"]
    require(isinstance(records, list) and len(records) == len(helper.SUPPLIED_OFFICE_FIXTURES), "exactly seven Office pair transcripts required")
    seen_inputs, seen_runs, seen_processes, seen_refs = set(), set(), set(), set()
    operations = []
    for ref in records:
        raw, transcript = reference(root, ref, "host public transcript")
        require(digest(raw) not in seen_refs, "same transcript aliased across fixtures")
        seen_refs.add(digest(raw))
        run = verify_transcript(transcript, fixtures=helper.SUPPLIED_OFFICE_FIXTURES,
                                html_sha=helper.ORIGINAL_HTML_SHA256, provenance=source,
                                main_sha=main_sha, worker_sha=worker_sha, producer_sha=producer_sha)
        require(run["target"] == bundle["target"], "Mac measurements cannot be transferred to another target")
        require(run["input_sha256"] not in seen_inputs and run["run_id"] not in seen_runs
                and (run["pid"], run["start_identity"]) not in seen_processes, "fixture/run/process identity reused")
        seen_inputs.add(run["input_sha256"])
        seen_runs.add(run["run_id"])
        seen_processes.add((run["pid"], run["start_identity"]))
        operations.extend(run["operations"])
        name = {"docx": "page", "xlsx": "sheet", "pptx": "slide"}[helper.SUPPLIED_OFFICE_FIXTURES[run["input_sha256"]]]
        require(any(item["kind"] == "office" and item["name"] in {name, "single_item"} for item in run["operations"]),
                "Office input lacks its actual navigation operation")
    require(seen_inputs == set(helper.SUPPLIED_OFFICE_FIXTURES), "missing fixed Office input")
    require({"scroll", "reload", "dirty_source"}.issubset({item["name"] for item in operations if item["kind"] == "html"}),
            "HTML scroll/reload/dirty source was not observed")
    assessments(root, evidence["upstream_assessments"])
    other = evidence["other_targets"]
    require(isinstance(other, dict) and set(other) == helper.SUPPORTED_TARGETS - {bundle["target"]}
            and all(value == "document_qa_not_run" for value in other.values()), "unmeasured targets cannot claim host QA success")
    if phase == "prepublish":
        require(evidence["publication"] == {"status": "postpublication_pending"}, "candidate does not establish final Release asset acceptance")
    else:
        from document_host_publication import verify_download_receipt
        verify_download_receipt(root, evidence["publication"], source, bundle)
    return evidence
