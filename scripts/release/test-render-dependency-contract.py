#!/usr/bin/env python3
"""Regression tests for the release renderer dependency graph contract."""

from __future__ import annotations

import importlib.util
import unittest
from itertools import product
from pathlib import Path
from typing import Any


SCRIPT_PATH = Path(__file__).with_name("check-render-dependency-contract.py")
SPEC = importlib.util.spec_from_file_location("render_dependency_contract", SCRIPT_PATH)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def package(
    name: str,
    version: str,
    *,
    dependencies: list[dict[str, Any]] | None = None,
    source: str | None = "registry+https://github.com/rust-lang/crates.io-index",
) -> dict[str, Any]:
    return {
        "id": f"{name}@{version}",
        "name": name,
        "version": version,
        "source": source,
        "dependencies": dependencies or [],
    }


def dependency(
    name: str,
    requirement: str,
    *,
    source: str | None = "registry+https://github.com/rust-lang/crates.io-index",
    path: str | None = None,
) -> dict[str, Any]:
    return {"name": name, "req": requirement, "source": source, "path": path}


def valid_metadata() -> dict[str, Any]:
    return {
        "workspace_members": ["katana-core@0.22.42", "katana-ui@0.22.42"],
        "packages": [
            package(
                "katana-core",
                "0.22.42",
                source=None,
                dependencies=[dependency("katana-render-runtime", "=0.4.19")],
            ),
            package(
                "katana-ui",
                "0.22.42",
                source=None,
                dependencies=[dependency("katana-document-viewer", "=0.5.6")],
            ),
            package("katana-document-viewer", "0.5.6"),
            package("katana-render-runtime", "0.4.19"),
            package("v8", "152.2.0"),
        ]
    }


class RenderDependencyContractTests(unittest.TestCase):
    def test_accepts_single_registry_v8_and_exact_renderer_dependencies(self) -> None:
        self.assertEqual(MODULE.validate(valid_metadata()), [])

    def test_rejects_duplicate_v8_versions(self) -> None:
        metadata = valid_metadata()
        metadata["packages"].append(package("v8", "150.0.0"))

        errors = MODULE.validate(metadata)

        self.assertTrue(any("exactly one v8" in error for error in errors))
        self.assertTrue(any("150.0.0, 152.2.0" in error for error in errors))

    def test_rejects_path_override(self) -> None:
        metadata = valid_metadata()
        metadata["packages"][1]["dependencies"][0] = dependency(
            "katana-document-viewer",
            "=0.5.6",
            source=None,
            path="../katana-document-viewer",
        )

        errors = MODULE.validate(metadata)

        self.assertTrue(any("path override" in error for error in errors))
        self.assertTrue(any("crates.io registry" in error for error in errors))

    def test_rejects_non_exact_requirement(self) -> None:
        metadata = valid_metadata()
        metadata["packages"][0]["dependencies"][0]["req"] = "0.4"

        errors = MODULE.validate(metadata)

        self.assertTrue(any("must be exact" in error for error in errors))

    def test_rejects_git_resolved_renderer(self) -> None:
        metadata = valid_metadata()
        metadata["packages"][2]["source"] = (
            "git+https://github.com/HiroyukiFuruno/katana-document-viewer"
        )

        errors = MODULE.validate(metadata)

        self.assertTrue(any("must resolve from crates.io" in error for error in errors))

    def test_rejects_non_registry_renderer_hidden_by_same_name(self) -> None:
        cases = product(
            sorted(MODULE.RENDER_PACKAGES),
            (None, "git+https://github.com/example/renderer"),
            (True, False),
        )
        for name, source, first in cases:
            with self.subTest(name=name, source=source, first=first):
                metadata = valid_metadata()
                extra = package(name, "0.0.1", source=source)
                if first:
                    metadata["packages"].insert(0, extra)
                else:
                    metadata["packages"].append(extra)

                errors = MODULE.validate(metadata)

                self.assertTrue(
                    any(
                        name in error and "must resolve from crates.io" in error
                        for error in errors
                    )
                )

    def test_rejects_alternate_registry_renderer(self) -> None:
        metadata = valid_metadata()
        metadata["packages"][2]["source"] = (
            "registry+https://registry.example.invalid/index"
        )

        errors = MODULE.validate(metadata)

        self.assertTrue(any("must resolve from crates.io" in error for error in errors))

    def test_rejects_alternate_registry_dependency_edge(self) -> None:
        metadata = valid_metadata()
        metadata["packages"][0]["dependencies"][0]["source"] = (
            "registry+https://registry.example.invalid/index"
        )

        errors = MODULE.validate(metadata)

        self.assertTrue(any("must use the crates.io registry" in error for error in errors))

    def test_rejects_crates_io_prefix_with_deceptive_suffix(self) -> None:
        metadata = valid_metadata()
        metadata["packages"][2]["source"] = (
            "registry+https://github.com/rust-lang/crates.io-index-attacker"
        )

        errors = MODULE.validate(metadata)

        self.assertTrue(any("must resolve from crates.io" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
