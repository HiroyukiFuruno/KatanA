"""Real PNG and temporary Git contract; these records are not release evidence."""

import copy
import base64
from datetime import datetime, timezone
from io import BytesIO
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import shutil

from PIL import Image, PngImagePlugin

from document_host_io import HostEvidenceError, digest
from v047_canonical_reference import INPUTS, SCHEMA, COMMAND, source_inputs, verify, verify_png
from v047_evidence_paths import CANONICAL_PNG, CANONICAL_RECORD

ROOT = Path(__file__).resolve().parents[2]


def git(root, *args):
    return subprocess.run(["git", "-C", str(root), "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                           "-c", "tag.gpgSign=false", *args],
                          check=True, capture_output=True, text=True).stdout.strip()


class Canonical47Tests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name).resolve()
        git(self.root, "init", "-q")
        (self.root / ".gitignore").write_text("tmp/\n__pycache__/\n")
        (self.root / "Cargo.toml").write_text('[workspace]\n[workspace.package]\nversion="0.22.47"\n')
        lock = ('[[package]]\nname="katana-render-runtime"\nversion="0.4.23"\n'
                'source="registry+https://github.com/rust-lang/crates.io-index"\nchecksum="' + "a" * 64 + '"\n')
        for path in INPUTS:
            target = self.root / path
            target.parent.mkdir(parents=True, exist_ok=True)
            content = (ROOT / path).read_text() if path.endswith("source_identity.py") else (
                lock if path.endswith("Cargo.lock") else "unit fixture\n")
            target.write_text(content)
        git(self.root, "add", ".")
        git(self.root, "commit", "-qm", "synthetic source")
        self.source = git(self.root, "rev-parse", "HEAD")
        git(self.root, "tag", "v0.22.47")
        png = BytesIO()
        Image.new("RGB", (1280, 2400), "white").save(png, format="PNG")
        self.image = png.getvalue()
        self.hashes, self.renderer = source_inputs(self.root, self.source)
        self.geometry = {"schema_version": 1, "configured_font_size": 14, "pixels_per_point": 2,
                         "require_clean_interaction_state": True, "ui_pass_frame_nr": 2,
                         "preview_geometry": {"physical_crop": {"x": 88, "y": 268, "width": 2374, "height": 4450}, "scroll_y": 0,
                                              "content_top_y": 134, "viewport": {"x": 44, "y": 134, "width": 1187, "height": 2225},
                                              "full_screenshot": {"width": 2565, "height": 4774, "output_name": "sample-diagrams-full"}},
                         "interaction_state": {"completed_ui_frame_nr": 2, "pointer_inside_content": False,
                             "pointer_position": {"x": 1.0, "y": 1.0},
                             "active_editor_line": None, "markdown_section_renders": 1,
                             **{key: 0 for key in ("hovered_preview_line_count", "diagram_control_renders", "image_control_renders",
                                 "code_copy_control_renders", "active_markdown_ranges", "hovered_markdown_spans",
                                 "image_hover_background_renders", "local_image_hover_background_renders", "code_selection_renders")}}}
        self.record = {"schema": SCHEMA, "version": "v0.22.47", "source_commit": self.source,
                       "source_tree": git(self.root, "rev-parse", "HEAD^{tree}"), "capture_started_at": "2026-11-02T01:00:00Z",
                       "capture_finished_at": "2026-11-02T01:01:00Z", "command": COMMAND,
                       "input_sha256": self.hashes, "renderer": self.renderer,
                       "source_identity": {"schema_version": 1, "head": self.source},
                       "geometry": self.geometry, "png_sha256": digest(self.image)}
        self.release = {"published_at": "2026-11-02T00:00:00Z"}
        (self.root / CANONICAL_PNG).parent.mkdir(parents=True)
        (self.root / CANONICAL_PNG).write_bytes(self.image)
        self.save()

    def tearDown(self):
        self.temporary.cleanup()

    def save(self):
        (self.root / CANONICAL_RECORD).write_text(json.dumps(self.record))

    def test_valid_capture_and_rejects_source_geometry_hash_and_time_changes(self):
        self.assertEqual(self.record, verify(self.root, self.source, self.release))
        for field, value in (("source_commit", "0" * 40), ("source_tree", "0" * 40),
                             ("png_sha256", "0" * 64), ("capture_started_at", "2026-11-01T00:00:00Z"),
                             ("version", "v0.22.42"), ("command", "other")):
            original = self.record[field]
            self.record[field] = value
            self.save()
            with self.subTest(field=field), self.assertRaises((HostEvidenceError, ValueError)):
                verify(self.root, self.source, self.release)
            self.record[field] = original
        self.record["geometry"] = copy.deepcopy(self.geometry)
        self.record["geometry"]["interaction_state"]["pointer_inside_content"] = True
        self.save()
        with self.assertRaises(HostEvidenceError):
            verify(self.root, self.source, self.release)

    def test_geometry_rejects_unknown_nested_content(self):
        for path in (("preview_geometry",), ("interaction_state",), ("interaction_state", "pointer_position")):
            self.record["geometry"] = copy.deepcopy(self.geometry)
            target = self.record["geometry"]
            for key in path:
                target = target[key]
            target["private_path"] = "/private/example"
            self.save()
            with self.subTest(path=path), self.assertRaises(HostEvidenceError):
                verify(self.root, self.source, self.release)

    def test_real_png_decode_rejects_wrong_dimensions_and_truncation(self):
        wrong = BytesIO()
        Image.new("RGB", (1, 1)).save(wrong, format="PNG")
        metadata = PngImagePlugin.PngInfo()
        metadata.add_text("private_path", "/private/example")
        annotated = BytesIO()
        Image.new("RGB", (1280, 2400), "white").save(annotated, format="PNG", pnginfo=metadata)
        for value in (wrong.getvalue(), self.image[:80], annotated.getvalue()):
            with self.assertRaises((HostEvidenceError, OSError, ValueError)):
                verify_png(value)

    def test_real_collector_rejects_existing_copied_capture(self):
        (self.root / CANONICAL_PNG).unlink()
        (self.root / CANONICAL_RECORD).unlink()
        capture = self.root / "tmp/capture"
        (capture / "raw").mkdir(parents=True)
        (capture / "source-identity.json").write_text(json.dumps({"schema_version": 1, "head": self.source}))
        (capture / "raw/sample-diagrams-preview-geometry.json").write_text(json.dumps(self.geometry))
        (capture / "sample-diagrams-top-candidate.png").write_bytes(self.image)
        command = [sys.executable, str(ROOT / "scripts/release/v047_canonical_reference.py"), "--root", str(self.root),
                   "--capture-directory", str(capture), "--source", self.source, "--published-at", "2020-01-01T00:00:00Z"]
        result = subprocess.run(command, capture_output=True, text=True)
        self.assertNotEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertIn("capture directory must be new", result.stderr)
        (capture / "sample-diagrams-top-candidate.png").touch()
        again = subprocess.run(command, capture_output=True, text=True)
        self.assertNotEqual(0, again.returncode)

    def test_collector_runs_fresh_isolated_fixture_process(self):
        (self.root / CANONICAL_PNG).unlink()
        (self.root / CANONICAL_RECORD).unlink()
        ci = self.root / "scripts/ci"
        ci.mkdir(parents=True)
        for name in ("isolated-test-config.py", "cargo_target_lease.py"):
            shutil.copyfile(ROOT / "scripts/ci" / name, ci / name)
        generator = self.root / COMMAND
        generator.write_text(f'''#!{sys.executable}
import base64, json, os, pathlib, subprocess, sys
root = pathlib.Path(__file__).resolve().parents[2]
capture = pathlib.Path(sys.argv[1])
capture.mkdir()
(capture / "raw").mkdir()
config = pathlib.Path(os.environ["KATANA_CONFIG_DIR"])
assert config.is_dir() and not any(config.iterdir())
(capture / "config-observed.txt").write_text(str(config))
subprocess.run([sys.executable, str(root / "scripts/screenshot/source_identity.py"), "begin", "--repository", str(root), "--identity", str(capture / "source-identity.json")], check=True)
(capture / "raw/sample-diagrams-preview-geometry.json").write_text({json.dumps(json.dumps(self.geometry))})
(capture / "sample-diagrams-top-candidate.png").write_bytes(base64.b64decode({base64.b64encode(self.image).decode()!r}))
''')
        generator.chmod(0o755)
        git(self.root, "add", ".")
        git(self.root, "commit", "-qm", "synthetic process fixture")
        self.source = git(self.root, "rev-parse", "HEAD")
        git(self.root, "tag", "-f", "v0.22.47")
        capture = self.root / "tmp/capture-new"
        capture.parent.mkdir()
        caller_config = self.root / "tmp/caller-config"
        caller_config.mkdir()
        environment = dict(os.environ, KATANA_CONFIG_DIR=str(caller_config))
        command = [sys.executable, str(ROOT / "scripts/release/v047_canonical_reference.py"), "--root", str(self.root),
                   "--capture-directory", str(capture), "--source", self.source, "--published-at", "2020-01-01T00:00:00Z"]
        before = datetime.now(timezone.utc)
        result = subprocess.run(command, capture_output=True, text=True, env=environment)
        after = datetime.now(timezone.utc)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        record = verify(self.root, self.source, {"published_at": "2020-01-01T00:00:00Z"})
        self.assertLessEqual(before, datetime.fromisoformat(record["capture_started_at"]))
        self.assertLessEqual(datetime.fromisoformat(record["capture_finished_at"]), after)
        self.assertEqual(digest(self.image), record["png_sha256"])
        self.assertNotEqual(str(caller_config), (capture / "config-observed.txt").read_text())
        self.assertFalse(any(caller_config.iterdir()))


if __name__ == "__main__":
    unittest.main()
