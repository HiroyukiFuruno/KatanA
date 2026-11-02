"""Bind the v47 in-process canonical image to the real published Git source."""

import argparse
from datetime import datetime, timezone
from io import BytesIO
from pathlib import Path
import subprocess
import sys
import tomllib

from document_host_io import decode, digest, fields, integer, moment, read, require, sha
from postpublication_controller_scope import git
from published_git_source import git_workspace_version
from v047_evidence_paths import CANONICAL_PNG, CANONICAL_RECORD

INPUTS = ("assets/fixtures/sample_diagrams.md", "scripts/screenshot/examples/sample-diagrams-canonical-capture.json",
          "scripts/screenshot/generate-sample-diagrams-reference.sh", "scripts/screenshot/run.sh",
          "scripts/screenshot/source_identity.py", "Cargo.lock", "scripts/screenshot/Cargo.lock")
SCHEMA = "katana-v47-canonical-reference-v1"
COMMAND = "scripts/screenshot/generate-sample-diagrams-reference.sh"


def source_inputs(root: Path, source: str) -> tuple[dict, dict]:
    values = {path: git(root, "show", f"{source}:{path}") for path in INPUTS}
    identities = []
    for path in ("Cargo.lock", "scripts/screenshot/Cargo.lock"):
        lock = tomllib.loads(values[path].decode("utf-8"))
        packages = [item for item in lock["package"] if item.get("name") == "katana-render-runtime"]
        require(len(packages) == 1 and packages[0].get("source") == "registry+https://github.com/rust-lang/crates.io-index",
                "canonical renderer must be a unique registry package")
        identities.append({"version": packages[0]["version"], "checksum": sha(packages[0]["checksum"], "renderer checksum")})
    require(identities[0] == identities[1], "canonical renderer locks differ")
    return {path: digest(raw) for path, raw in values.items()}, identities[0]


def verify_geometry(value: object) -> None:
    geometry = fields(value, {"schema_version", "configured_font_size", "pixels_per_point", "preview_geometry",
                              "require_clean_interaction_state", "ui_pass_frame_nr", "interaction_state"}, "capture geometry")
    require(geometry["schema_version"] == 1 and geometry["require_clean_interaction_state"] is True
            and geometry["configured_font_size"] == 14 and geometry["pixels_per_point"] == 2,
            "canonical geometry schema or clean interaction differs")
    preview = fields(geometry["preview_geometry"], {"physical_crop", "scroll_y", "content_top_y", "viewport", "full_screenshot"}, "preview geometry")
    require(preview.get("physical_crop") == {"x": 88, "y": 268, "width": 2374, "height": 4450}
            and preview.get("scroll_y") == 0 and preview.get("content_top_y") == 134
            and preview.get("viewport") == {"x": 44, "y": 134, "width": 1187, "height": 2225}
            and preview.get("full_screenshot") == {"width": 2565, "height": 4774, "output_name": "sample-diagrams-full"},
            "canonical crop, viewport, or scroll differs")
    counters = ("hovered_preview_line_count", "diagram_control_renders", "image_control_renders",
                "code_copy_control_renders", "active_markdown_ranges", "hovered_markdown_spans",
                "image_hover_background_renders", "local_image_hover_background_renders", "code_selection_renders")
    state = fields(geometry["interaction_state"], set(counters) | {"completed_ui_frame_nr", "pointer_inside_content",
                   "active_editor_line", "markdown_section_renders", "pointer_position"}, "interaction state")
    pointer = fields(state["pointer_position"], {"x", "y"}, "pointer position")
    require(all(type(pointer[k]) in (int, float) and -100000 < pointer[k] < 100000 for k in ("x", "y")), "invalid pointer position")
    require(integer(geometry["ui_pass_frame_nr"], "capture UI frame", minimum=1)
            == state.get("completed_ui_frame_nr") and state.get("pointer_inside_content") is False
            and state.get("active_editor_line") is None and integer(state.get("markdown_section_renders"), "rendered sections", minimum=1),
            "canonical capture was not a completed clean frame")
    for key in counters:
        require(type(state.get(key)) is int and state[key] == 0, "canonical interaction is not clean")


def verify_png(raw: bytes) -> None:
    from PIL import Image
    require(raw.startswith(b"\x89PNG\r\n\x1a\n"), "canonical PNG signature differs")
    offset, chunks = 8, []
    while offset + 12 <= len(raw):
        size = int.from_bytes(raw[offset:offset + 4], "big")
        kind = raw[offset + 4:offset + 8]
        require(offset + 12 + size <= len(raw) and kind in (b"IHDR", b"IDAT", b"IEND"), "unexpected PNG chunk")
        chunks.append(kind)
        offset += 12 + size
        if kind == b"IEND":
            break
    require(offset == len(raw) and len(chunks) >= 3 and chunks[0] == b"IHDR" and chunks[-1] == b"IEND"
            and all(kind == b"IDAT" for kind in chunks[1:-1]), "canonical PNG chunk sequence differs")
    with Image.open(BytesIO(raw)) as image:
        require(image.format == "PNG" and image.size == (1280, 2400), "canonical PNG dimensions differ")
        image.verify()
    with Image.open(BytesIO(raw)) as image:
        image.load()


def verify(root: Path, source: str, release: dict) -> dict:
    record = fields(decode(read(root / CANONICAL_RECORD)), {"schema", "version", "source_commit", "source_tree",
                    "capture_started_at", "capture_finished_at", "command", "input_sha256", "renderer",
                    "source_identity", "geometry", "png_sha256"}, "canonical reference")
    require(record["schema"] == SCHEMA and record["version"] == "v0.22.47" and record["source_commit"] == source
            and git_workspace_version(root, source) == "v0.22.47", "canonical tag/source differs")
    require(record["source_tree"] == git(root, "rev-parse", source + "^{tree}").decode().strip(), "canonical tree differs")
    require(record["source_identity"] == {"schema_version": 1, "head": source}, "canonical clean source identity differs")
    require(record["command"] == COMMAND, "canonical generator differs")
    require(moment(release["published_at"], "publication") <= moment(record["capture_started_at"], "capture start")
            <= moment(record["capture_finished_at"], "capture end"), "canonical capture predates publication")
    hashes, renderer = source_inputs(root, source)
    require(record["input_sha256"] == hashes and record["renderer"] == renderer, "canonical input or renderer differs")
    verify_geometry(record["geometry"])
    image = read(root / CANONICAL_PNG)
    require(record["png_sha256"] == digest(image), "canonical PNG hash differs")
    verify_png(image)
    return record


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--capture-directory", type=Path, required=True)
    parser.add_argument("--source", required=True)
    parser.add_argument("--published-at", required=True)
    args = parser.parse_args()
    try:
        root, capture = args.root.resolve(), args.capture_directory.resolve()
        require(capture.is_relative_to(root / "tmp") and capture != root / "tmp", "ignored private capture directory required")
        subprocess.run(["git", "-C", str(root), "check-ignore", "--quiet", "--", str(capture)], check=True)
        require(not git(root, "status", "--porcelain=v1", "--untracked-files=normal"), "canonical capture source is not clean")
        require(git(root, "rev-parse", "HEAD").decode().strip() == args.source, "capture HEAD differs")
        require(git(root, "rev-parse", "v0.22.47^{commit}").decode().strip() == args.source, "capture tag differs")
        require(not capture.exists() and capture.parent.is_dir() and capture.parent.resolve() == capture.parent,
                "capture directory must be new")
        start = datetime.now(timezone.utc).isoformat()
        require(moment(args.published_at, "publication") <= moment(start, "capture start"), "publication is in the future")
        subprocess.run([sys.executable, str(root / "scripts/ci/isolated-test-config.py"),
                        str(root / COMMAND), str(capture)], cwd=root, check=True)
        end = datetime.now(timezone.utc).isoformat()
        subprocess.run([sys.executable, str(root / "scripts/screenshot/source_identity.py"), "verify",
                        "--repository", str(root), "--identity", str(capture / "source-identity.json")], check=True)
        identity = decode(read(capture / "source-identity.json"))
        geometry = decode(read(capture / "raw/sample-diagrams-preview-geometry.json"))
        image_path = capture / "sample-diagrams-top-candidate.png"
        image = read(image_path)
        hashes, renderer = source_inputs(root, args.source)
        record = {"schema": SCHEMA, "version": "v0.22.47", "source_commit": args.source,
                  "source_tree": git(root, "rev-parse", args.source + "^{tree}").decode().strip(),
                  "capture_started_at": start, "capture_finished_at": end, "command": COMMAND,
                  "input_sha256": hashes, "renderer": renderer, "source_identity": identity,
                  "geometry": geometry, "png_sha256": digest(image)}
        require(moment(args.published_at, "publication") <= moment(start, "capture start") <= moment(end, "capture end"),
                "canonical capture predates publication")
        verify_geometry(geometry)
        verify_png(image)
        from document_host_io import write_new
        output = root / CANONICAL_PNG
        require(not output.exists() and not (root / CANONICAL_RECORD).exists()
                and output.resolve() == output, "canonical reference path already exists or escapes repository")
        output.parent.mkdir(parents=True, exist_ok=True)
        with output.open("xb") as stream:
            stream.write(image)
        write_new(root / CANONICAL_RECORD, record)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(f"ERROR: v47 canonical reference: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
