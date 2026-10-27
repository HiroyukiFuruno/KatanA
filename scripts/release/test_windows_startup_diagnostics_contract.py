"""preflight discoveryから既存の診断テスト入口へ接続する。"""

import importlib.util
from pathlib import Path
import unittest


MAIN_TEST = Path(__file__).with_name("test-windows-startup-diagnostics.py")


def load_main_tests():
    spec = importlib.util.spec_from_file_location("windows_startup_diagnostics_main", MAIN_TEST)
    if spec is None or spec.loader is None:
        raise AssertionError(f"could not load {MAIN_TEST}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def load_tests(loader, standard_tests, pattern):
    main_tests = load_main_tests()
    standard_tests.addTests(loader.loadTestsFromModule(main_tests))
    return standard_tests


if __name__ == "__main__":
    unittest.main()
