# ============================================================
# KatanA — Development Makefile
# ============================================================
# Usage: make <target>
#   make help   — Show a list of available commands
# ============================================================

.DEFAULT_GOAL := help

###################################
# Shared Settings
###################################

# Job limit for parallel execution (defaults to 2 as requested to reduce load during checks)
JOBS ?= 2

# Force all warnings to be treated as errors for every cargo command run via make
export RUSTFLAGS=-D warnings

# AI context-aware CLI proxy (mandatory for agents)
RTK := $(shell command -v rtk 2> /dev/null || echo "")

VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')

KML_VERSION ?= 0.16.1
KML ?= kml
KML_MCP ?= kml-mcp
KML_INSTALL_FEATURES ?= cli,mcp,jsonc
KML_INSTALL_FLAGS ?=
KML_CONFIG ?= .markdownlint.json
KML_SCOPE ?= .
KML_EXCLUDE_ARGS ?= --exclude "openspec/changes/archive/**" --exclude "target/**" --exclude "scripts/screenshot/target/**"
KML_CHECK_ARGS ?= --config $(KML_CONFIG) --include "**/*.md" --include "**/*.markdown" $(KML_EXCLUDE_ARGS) $(KML_SCOPE)
OPENSPEC ?= scripts/openspec
MERMAID_FIXTURES ?= assets/fixtures/mermaid_all
MERMAID_REFERENCE_DIR ?= $(MERMAID_FIXTURES)/official
MERMAID_KATANA_DIR ?= tmp/mermaid-katana-rendered
MERMAID_REFERENCE_BROWSER_DIR ?= tmp/mermaid-reference-browser
MERMAID_KATANA_BROWSER_DIR ?= tmp/mermaid-katana-browser
MERMAID_SAMPLE ?= assets/fixtures/mermaid.md
MERMAID_SAMPLE_FIXTURES_DIR ?= tmp/mermaid-sample-fixtures
MERMAID_SAMPLE_REFERENCE_DIR ?= tmp/mermaid-sample-official
MERMAID_SAMPLE_KATANA_DIR ?= tmp/mermaid-sample-katana
MERMAID_SAMPLE_REFERENCE_BROWSER_DIR ?= tmp/mermaid-sample-official-browser
MERMAID_SAMPLE_KATANA_BROWSER_DIR ?= tmp/mermaid-sample-katana-browser
MERMAID_SAMPLE_COMPARISON_DIR ?= tmp/mermaid-sample-comparison
MERMAID_JS ?= $(HOME)/.local/katana/mermaid.min.js
MERMAID_MIN_SCORE ?= 99
DRAWIO_FIXTURES ?= assets/fixtures/drawio/basic
DRAWIO_REFERENCE_DIR ?= $(DRAWIO_FIXTURES)/official
DRAWIO_KATANA_DIR ?= tmp/drawio-katana-rendered
DRAWIO_KATANA_BROWSER_DIR ?= tmp/drawio-katana-browser
DRAWIO_COMPARISON_DIR ?= tmp/drawio-official-comparison
DRAWIO_JS ?= $(HOME)/.local/katana/drawio.min.js
DRAWIO_MIN_SCORE ?= 99
DRAWIO_RESOURCE_DIR ?= crates/katana-core/src/markdown/drawio_renderer/js_runtime/resources
DRAWIO_RESOURCE_MANIFEST ?= $(DRAWIO_RESOURCE_DIR)/drawio-resource-manifest.json
DRAWIO_RESOURCE_FIXTURES ?= assets/fixtures/drawio
PLAYWRIGHT ?= playwright
BIOME_VERSION ?= 2.4.13
BIOME ?= bunx @biomejs/biome@$(BIOME_VERSION)
# Biome対象はKatanAが所有するJS/TSに限定する。
# Draw.io公式リソース配下は、公式ファイルをそのまま保持するためformat/lint対象外。
BIOME_JS_TS_FILES := \
	crates/katana-core/src/markdown/mermaid_renderer/js_runtime/*.js \
	crates/katana-core/src/markdown/drawio_renderer/js_runtime/*.js \
	scripts/mermaid/*.ts \
	scripts/drawio/*.ts
BIOME_JSON_FILES := \
	.markdownlint.json \
	.vscode/settings.json \
	biome.jsonc \
	crates/katana-ui/locales/*.json \
	crates/katana-ui/resources/*.json \
	$(DRAWIO_RESOURCE_MANIFEST) \
	scripts/screenshot/examples/*.json

# Suppress "ar: illegal option -- D" warning on macOS by using llvm-ar if available
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
  LLVM_AR_ARM := /opt/homebrew/opt/llvm/bin/llvm-ar
  LLVM_AR_INTEL := /usr/local/opt/llvm/bin/llvm-ar
  ifneq ($(wildcard $(LLVM_AR_ARM)),)
    export AR=$(LLVM_AR_ARM)
  else ifneq ($(wildcard $(LLVM_AR_INTEL)),)
    export AR=$(LLVM_AR_INTEL)
  endif
endif

include make/setup.mk
include make/quality.mk
include make/tests.mk
include make/diagrams.mk
include make/docs.mk
include make/release.mk
include make/maintenance.mk
include make/help.mk
