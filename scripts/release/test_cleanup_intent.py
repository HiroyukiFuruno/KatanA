"""Filesystem contract tests for private cleanup intents."""

import hashlib
import json
import os
from pathlib import Path
import tempfile
import unittest

from cleanup_intent import read_intent, write_intent
from cleanup_intent_io import MAX_INTENT_BYTES
from release_cleanup_audit import CleanupRefused, git


class CleanupIntentTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        base = Path(self.temporary.name)
        self.root = base / "source"
        self.private = base / "private"
        self.root.mkdir()
        self.private.mkdir(mode=0o700)
        git(self.root, "init", "--initial-branch=master", "--object-format=sha1")
        git(self.root, "config", "user.email", "cleanup@example.invalid")
        git(self.root, "config", "user.name", "Cleanup fixture")
        git(self.root, "config", "commit.gpgsign", "false")
        git(self.root, "commit", "--allow-empty", "-m", "initial")
        tip = git(self.root, "rev-parse", "HEAD").decode().strip()
        git(self.root, "branch", "release/completed", tip)
        self.intent, self.digest = write_intent(self.private, self.root, "release/completed", tip)

    def _payload(self) -> dict[str, object]:
        return json.loads(self.intent.read_text(encoding="utf-8"))

    def _rewrite(self, payload: dict[str, object]) -> str:
        raw = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()
        self.intent.write_bytes(raw)
        return hashlib.sha256(raw).hexdigest()

    def _assert_rejected(self, digest: str | None = None) -> None:
        with self.assertRaises(CleanupRefused):
            read_intent(self.intent, digest or hashlib.sha256(self.intent.read_bytes()).hexdigest())

    def test_roundtrip_uses_absolute_path_and_preserves_full_oid(self) -> None:
        raw_path = Path(os.path.relpath(self.private, Path.cwd()))
        self.intent.unlink()
        tip = git(self.root, "rev-parse", "HEAD").decode().strip()
        path, digest = write_intent(raw_path, self.root, "release/completed", tip)
        self.assertTrue(path.is_absolute())
        self.assertEqual(read_intent(path, digest)["expected"], tip)

    def test_overwrite_is_rejected_and_original_bytes_remain(self) -> None:
        original = self.intent.read_bytes()
        tip = git(self.root, "rev-parse", "HEAD").decode().strip()
        with self.assertRaises(CleanupRefused):
            write_intent(self.private, self.root, "release/completed", tip)
        self.assertEqual(self.intent.read_bytes(), original)

    def test_duplicate_json_key_is_rejected_after_digest_recalculation(self) -> None:
        payload = self._payload()
        pairs = list(payload.items()) + [("branch", "release/completed")]
        raw = b"{" + b",".join(json.dumps(key).encode() + b":" + json.dumps(value).encode()
                                  for key, value in pairs) + b"}"
        self.intent.write_bytes(raw)
        self._assert_rejected(hashlib.sha256(raw).hexdigest())

    def test_unknown_schema_and_field_are_independent_rejections(self) -> None:
        payload = self._payload()
        original = payload.copy()
        payload["schema"] = 2
        self._assert_rejected(self._rewrite(payload))
        payload = original.copy()
        payload["unexpected"] = "field"
        self._assert_rejected(self._rewrite(payload))

    def test_nonce_branch_oid_and_common_directory_mismatches_are_rejected(self) -> None:
        original = self._payload()
        for field, value in (("nonce", "bad"), ("branch", "main"), ("expected", "0" * 39),
                             ("common_git_dir", str(self.root.parent))):
            with self.subTest(field=field):
                payload = original.copy()
                payload[field] = value
                self._assert_rejected(self._rewrite(payload))

    def test_symlink_file_and_unsafe_modes_are_rejected(self) -> None:
        target = self.private / "target.json"
        target.write_bytes(self.intent.read_bytes())
        self.intent.unlink()
        self.intent.symlink_to(target)
        self._assert_rejected(self.digest)
        self.intent.unlink()
        self.intent.write_bytes(target.read_bytes())
        os.chmod(self.intent, 0o644)
        self._assert_rejected(self.digest)
        os.chmod(self.intent, 0o600)
        os.chmod(self.private, 0o755)
        self._assert_rejected(self.digest)

    def test_oversize_payload_is_rejected_before_digest_can_authorize_it(self) -> None:
        payload = self.intent.read_bytes() + b"x" * (MAX_INTENT_BYTES + 1)
        self.intent.write_bytes(payload)
        self._assert_rejected(hashlib.sha256(payload).hexdigest())


if __name__ == "__main__":
    unittest.main()
