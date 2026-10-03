#!/usr/bin/env python3
"""Real archive regressions for publication identity (no command mocks)."""

import hashlib
import importlib.util
import io
import stat
import subprocess
import sys
import tarfile
import tempfile
import unittest
import warnings
import zipfile
from pathlib import Path


SPEC = importlib.util.spec_from_file_location("publication_identity", Path(__file__).with_name("check-published-artifact-identity.py"))
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class PublicationIdentityTests(unittest.TestCase):
    def archives(self, root, changed=None):
        main, sidecar = b"accepted real main bytes", b"accepted real sidecar bytes"
        targets = {target: {"main_sha256": hashlib.sha256(main).hexdigest(), "sidecar_sha256": hashlib.sha256(sidecar).hexdigest()} for target in ("macos-arm64", "macos-x86_64", "linux-x86_64", "windows-x86_64")}
        for filename, prefix, suffix in (("KatanA-macOS.zip", "KatanA Desktop.app/Contents/MacOS/", ""), ("KatanA-windows-x86_64.zip", "", ".exe")):
            with zipfile.ZipFile(root / filename, "w") as archive:
                for name, data in (("KatanA", main), ("kdv-office-worker", sidecar)):
                    archive.writestr(prefix + name + suffix, b"unaccepted CI rebuild" if changed == (filename, name) else data)
        filename = "KatanA-linux-x86_64.tar.gz"
        with tarfile.open(root / filename, "w:gz") as archive:
            for name, data in (("KatanA", main), ("kdv-office-worker", sidecar)):
                data = b"unaccepted CI rebuild" if changed == (filename, name) else data
                entry = tarfile.TarInfo(name)
                entry.size = len(data)
                archive.addfile(entry, io.BytesIO(data))
        return targets

    def test_downloaded_archives_match_all_four_accepted_targets(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            MODULE.check_portable(root, self.archives(root))

    def test_newly_built_main_or_worker_cannot_reuse_old_acceptance(self):
        for archive in ("KatanA-macOS.zip", "KatanA-windows-x86_64.zip", "KatanA-linux-x86_64.tar.gz"):
            for binary in ("KatanA", "kdv-office-worker"):
                with self.subTest(archive=archive, binary=binary), tempfile.TemporaryDirectory() as directory:
                    root = Path(directory)
                    targets = self.archives(root, (archive, binary))
                    with self.assertRaisesRegex(MODULE.ArtifactIdentityError, "does not match accepted"):
                        MODULE.check_portable(root, targets)

    def test_universal_zip_must_match_both_architecture_acceptances(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            targets = self.archives(root)
            targets["macos-x86_64"]["main_sha256"] = "f" * 64
            with self.assertRaises(MODULE.ArtifactIdentityError):
                MODULE.check_portable(root, targets)

    def test_duplicate_symlink_empty_and_missing_zip_payloads_fail(self):
        for kind in ("duplicate", "symlink", "empty", "missing"):
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as directory:
                archive_path = Path(directory) / "payload.zip"
                with warnings.catch_warnings(), zipfile.ZipFile(archive_path, "w") as archive:
                    warnings.simplefilter("ignore", UserWarning)
                    if kind == "duplicate":
                        archive.writestr("KatanA", b"first")
                        archive.writestr("KatanA", b"second")
                    elif kind == "symlink":
                        entry = zipfile.ZipInfo("KatanA")
                        entry.external_attr = (stat.S_IFLNK | 0o777) << 16
                        archive.writestr(entry, b"outside")
                    elif kind == "empty":
                        archive.writestr("KatanA", b"")
                with self.assertRaises(MODULE.ArtifactIdentityError):
                    MODULE.zip_digest(archive_path, "KatanA")

    def test_tar_symlink_duplicate_missing_and_empty_payloads_fail(self):
        for kind in ("symlink", "duplicate", "missing", "empty"):
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / "payload.tar.gz"
                with tarfile.open(path, "w:gz") as archive:
                    if kind != "missing":
                        for _ in range(2 if kind == "duplicate" else 1):
                            entry = tarfile.TarInfo("KatanA")
                            if kind == "symlink":
                                entry.type, entry.linkname = tarfile.SYMTYPE, "outside"
                                archive.addfile(entry)
                            else:
                                data = b"" if kind == "empty" else b"main"
                                entry.size = len(data)
                                archive.addfile(entry, io.BytesIO(data))
                with self.assertRaises(MODULE.ArtifactIdentityError):
                    MODULE.tar_digest(path, "KatanA")

    def test_publish_checks_downloaded_payloads_before_release_mutation(self):
        root = Path(__file__).resolve().parents[2]
        workflow = (root / ".github/workflows/build-and-release.yml").read_text()
        publish = workflow.split("  publish:", 1)[1]
        self.assertLess(publish.index("collect-artifacts.sh"), publish.index("check-published-artifact-identity.py"))
        self.assertLess(publish.index("check-published-artifact-identity.py"), publish.index("Create or Update GitHub Release"))
        self.assertIn("check_dmg(dmgs[0]", (root / "scripts/release/check-published-artifact-identity.py").read_text())
        self.assertIn("check_msi(assets /", (root / "scripts/release/check-published-artifact-identity.py").read_text())

    def test_unexpected_version_never_reports_success(self):
        script = Path(__file__).with_name("check-published-artifact-identity.py")
        result = subprocess.run([sys.executable, str(script), "--root", ".", "--assets", ".", "--version", "v0.22.43"], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("unsupported publication acceptance version", result.stderr)

    def test_argument_version_cannot_override_workspace_version(self):
        script = Path(__file__).with_name("check-published-artifact-identity.py").resolve()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text('[workspace.package]\nversion = "0.22.43"\n')
            result = subprocess.run([sys.executable, str(script), "--root", directory, "--assets", directory, "--version", "v0.22.42"], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("does not match Cargo.toml", result.stderr)


if __name__ == "__main__":
    unittest.main()
