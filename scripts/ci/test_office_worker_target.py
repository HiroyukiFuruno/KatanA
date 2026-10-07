"""実際のCargo出力形式によるWindows/Unixパスの回帰検査。"""

import unittest

from office_worker_target import normalize_target


class OfficeWorkerTargetTests(unittest.TestCase):
    def test_windows_drive_path(self):
        self.assertEqual(normalize_target(r"D:\a\KatanA\KatanA\target", True),
                         "D:/a/KatanA/KatanA/target")

    def test_windows_unc_path(self):
        self.assertEqual(normalize_target(r"\\server\share\target", True),
                         "//server/share/target")

    def test_unix_absolute_path(self):
        self.assertEqual(normalize_target("/work dir/target", False), "/work dir/target")

    def test_rejects_relative_and_drive_relative_paths(self):
        for path, windows in [("target", False), ("target", True),
                              (r"D:target", True), (r"\target", True)]:
            with self.subTest(path=path, windows=windows), self.assertRaises(ValueError):
                normalize_target(path, windows)


if __name__ == "__main__":
    unittest.main()
