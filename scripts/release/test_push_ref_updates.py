"""Pure native ref parsing, separate from actual Git admission verification."""

import os
import subprocess
import unittest

from push_ref_updates import MAX_PUSH_INPUT_BYTES, PushInputError, parse_push_updates


class PushRefUpdateTests(unittest.TestCase):
    def test_multiple_updates_and_head_source_preserve_exact_data(self):
        raw = (f"HEAD {'1' * 40} refs/heads/release/a {'2' * 40}\n"
               f"refs/heads/release/b {'3' * 40} refs/heads/release/b {'0' * 40}\n").encode()
        updates = parse_push_updates(raw)
        self.assertEqual(len(updates), 2)
        self.assertEqual(updates[0].local_ref, "HEAD")
        self.assertFalse(updates[0].created)
        self.assertTrue(updates[1].created)

    def test_delete_and_sha256_metadata_are_preserved(self):
        update, = parse_push_updates(f"(delete) {'0' * 64} refs/tags/v1.0.0 {'1' * 64}\n".encode())
        self.assertTrue(update.deleted)
        self.assertFalse(update.created)
        self.assertEqual(update.remote_oid, "1" * 64)

    def test_empty_input_is_only_empty_selection(self):
        self.assertEqual(parse_push_updates(b""), ())

    def test_native_unicode_ref_characters_are_not_field_delimiters(self):
        environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        for character in ("\u3000", "\u2028", "\u0085"):
            reference = f"refs/heads/release/検証{character}結果"
            subprocess.run(["git", "check-ref-format", reference], env=environment,
                           check=True, capture_output=True)
            update, = parse_push_updates(f"HEAD {'1' * 40} {reference} {'2' * 40}\n".encode())
            self.assertEqual(update.remote_ref, reference)

    def test_malformed_duplicate_or_mixed_format_inputs_are_rejected(self):
        valid = f"HEAD {'1' * 40} refs/heads/release/a {'2' * 40}\n"
        for raw in ("missing fields", valid + valid,
                    f"HEAD {'1' * 40} refs/heads/release/a {'2' * 64}",
                    f"HEAD {'0' * 40} refs/heads/release/a {'2' * 40}",
                    f"(delete) {'0' * 40} refs/heads/release/a {'0' * 40}",
                    valid + f"HEAD {'3' * 64} refs/heads/release/b {'4' * 64}",
                    valid.replace("refs/heads/release/a", "--bad"),
                    valid.replace("HEAD", ""),
                    valid.replace("release/a", "release/a\x00"),
                    valid.replace("1" * 40, "F" * 40)):
            with self.subTest(raw=raw), self.assertRaises(PushInputError):
                parse_push_updates(raw.encode())

    def test_size_and_encoding_are_bounded(self):
        for raw in (b"x" * (MAX_PUSH_INPUT_BYTES + 1), b"\xff"):
            with self.assertRaises(PushInputError):
                parse_push_updates(raw)


if __name__ == "__main__":
    unittest.main()
