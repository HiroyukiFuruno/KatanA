# ============================================================
# KatanA - Development Justfile
# ============================================================
# Stable local, CI/CD, and multi-platform task entrypoint.
# Usage:
#   just
#   just <recipe>
#   just VERSION=x.y.z release
# ============================================================

set shell := ["bash", "-uc"]

RTK := env_var_or_default("RTK", `command -v rtk 2> /dev/null || true`)
RTK_CMD := if RTK == "" { "" } else { RTK + " " }
JOBS := env_var_or_default("JOBS", "2")
export RUSTFLAGS := env_var_or_default("RUSTFLAGS", "-D warnings")

LLVM_AR := `if [ "$(uname -s)" = "Darwin" ]; then if [ -x /opt/homebrew/opt/llvm/bin/llvm-ar ]; then printf '%s' /opt/homebrew/opt/llvm/bin/llvm-ar; elif [ -x /usr/local/opt/llvm/bin/llvm-ar ]; then printf '%s' /usr/local/opt/llvm/bin/llvm-ar; fi; fi`
CARGO := if LLVM_AR == "" { RTK_CMD + "cargo" } else { "AR=" + LLVM_AR + " " + RTK_CMD + "cargo" }
DOCKER := RTK_CMD + "docker"

VERSION := env_var_or_default("VERSION", `grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/'`)
FORCE := env_var_or_default("FORCE", "")
T := env_var_or_default("T", "")

KML_VERSION := env_var_or_default("KML_VERSION", "0.16.1")
KML := env_var_or_default("KML", "kml")
KML_MCP := env_var_or_default("KML_MCP", "kml-mcp")
KML_INSTALL_FEATURES := env_var_or_default("KML_INSTALL_FEATURES", "cli,mcp,jsonc")
KML_INSTALL_FLAGS := env_var_or_default("KML_INSTALL_FLAGS", "")
KML_CONFIG := env_var_or_default("KML_CONFIG", ".markdownlint.json")
KML_SCOPE := env_var_or_default("KML_SCOPE", ".")
KML_EXCLUDE_ARGS := env_var_or_default("KML_EXCLUDE_ARGS", "--exclude \"openspec/changes/archive/**\" --exclude \"target/**\" --exclude \"scripts/screenshot/target/**\"")
KML_CHECK_ARGS := env_var_or_default("KML_CHECK_ARGS", "--config " + KML_CONFIG + " --include \"**/*.md\" --include \"**/*.markdown\" " + KML_EXCLUDE_ARGS + " " + KML_SCOPE)
OPENSPEC := env_var_or_default("OPENSPEC", "scripts/openspec")

BIOME_VERSION := env_var_or_default("BIOME_VERSION", "2.4.13")
BIOME := env_var_or_default("BIOME", "bunx @biomejs/biome@" + BIOME_VERSION)
BIOME_JS_TS_FILES := env_var_or_default("BIOME_JS_TS_FILES", "")
BIOME_JSON_FILES := env_var_or_default("BIOME_JSON_FILES", ".markdownlint.json .vscode/settings.json biome.jsonc crates/katana-ui/locales/*.json crates/katana-ui/resources/*.json scripts/screenshot/examples/*.json")

[private]
default: help

# Show this help
help:
    @just --list --unsorted

[private]
version-required:
    @test -n "{{ VERSION }}" || { echo "VERSION is required. Usage: just VERSION=x.y.z <recipe>"; exit 1; }

import 'just/setup.just'
import 'just/quality.just'
import 'just/tests.just'
import 'just/docs.just'
import 'just/release.just'
import 'just/maintenance.just'
