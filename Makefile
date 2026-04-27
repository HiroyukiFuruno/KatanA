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

KML_VERSION ?= 0.11.1
KML ?= kml
KML_MCP ?= kml-mcp
KML_INSTALL_FEATURES ?= cli,mcp,jsonc
KML_INSTALL_FLAGS ?=
KML_CONFIG ?= .markdownlint.json
KML_SCOPE ?= .
KML_EXCLUDE_ARGS ?= --exclude "openspec/changes/archive/**" --exclude "target/**" --exclude "scripts/screenshot/target/**"
KML_CHECK_ARGS ?= --config $(KML_CONFIG) --include "**/*.md" --include "**/*.markdown" $(KML_EXCLUDE_ARGS) $(KML_SCOPE)
OPENSPEC ?= scripts/openspec

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

###################################
# Setup
###################################

.PHONY: init
init: ## Bootstrap the development environment interactively
	scripts/setup/setup.sh

###################################
# Run / Watch
###################################

.PHONY: run-release
run-release: sweep ## Run the application in release mode
	$(RTK) cargo run --bin KatanA --release

.PHONY: run-performance
run-performance: sweep ## Run in release mode with FPS monitor logging
	RUST_LOG=warn $(RTK) cargo run --bin KatanA --release

.PHONY: run
run: sweep build ## Run the application (KatanA)
	$(RTK) cargo run --bin KatanA

.PHONY: watch
watch: sweep ## Watch file changes & auto check (requires cargo-watch)
	$(RTK) cargo watch -x 'check --workspace' -x 'test --workspace'

.PHONY: watch-run
watch-run: sweep ## Watch file changes & auto restart (requires cargo-watch)
	$(RTK) cargo watch -x 'run --bin KatanA'

###################################
# Build
###################################

.PHONY: build
build: sweep ## Build the entire workspace (debug)
	$(RTK) cargo build --workspace

.PHONY: build-release
build-release: sweep ## Release build (optimized)
	$(RTK) cargo build --workspace --release

###################################
# Quality
###################################

.PHONY: fmt locale-json-fmt
fmt: ## Apply code formatting (rustfmt)
	$(RTK) cargo fmt --all

.PHONY: locale-json-fmt
locale-json-fmt:
	rtk err bunx biome format --indent-style space --indent-width 2 --write crates/katana-ui/locales/*.json

.PHONY: fmt-check
fmt-check: ## Check format differences (for CI)
	$(RTK) cargo fmt --all -- --check

.PHONY: lint
lint: ## Run Clippy (forces zero warnings)
	$(RTK) cargo clippy -j $(JOBS) --workspace -- -D warnings

.PHONY: lint-fix
lint-fix: ## Run Clippy and apply automatic fixes
	$(RTK) cargo clippy --workspace --fix --allow-dirty --allow-staged -- -D warnings

.PHONY: kml-install
kml-install: ## Install KML CLI and MCP server (katana-markdown-linter)
	$(RTK) cargo install katana-markdown-linter --version $(KML_VERSION) --features $(KML_INSTALL_FEATURES) --force $(KML_INSTALL_FLAGS)

.PHONY: kml-require
kml-require:
	@command -v $(KML) >/dev/null || { echo "kml not found. Run: make kml-install"; exit 127; }

.PHONY: kml-mcp-require
kml-mcp-require:
	@command -v $(KML_MCP) >/dev/null || { echo "kml-mcp not found. Run: make kml-install"; exit 127; }

.PHONY: kml-check
kml-check: kml-require ## Run KML markdown lint without modifying files
	$(KML) check $(KML_CHECK_ARGS)

.PHONY: kml-check-json
kml-check-json: kml-require ## Run KML markdown lint and print JSON diagnostics
	$(KML) check --output json $(KML_CHECK_ARGS)

.PHONY: kml-fix
kml-fix: kml-require ## Apply KML safe Markdown fixes
	$(KML) fix $(KML_CHECK_ARGS)

.PHONY: kml-mcp
kml-mcp: kml-mcp-require ## Start the KML MCP server over stdio
	$(KML_MCP)

.PHONY: openspec-require
openspec-require: ## Verify OpenSpec CLI through the repo wrapper
	@$(OPENSPEC) --version >/dev/null

.PHONY: openspec-list
openspec-list: openspec-require ## List active OpenSpec changes
	$(OPENSPEC) list

.PHONY: ast-lint
ast-lint: ## Run AST-based custom linters (comment style, etc.)
	$(RTK) cargo test -j $(JOBS) -p katana-linter ast_linter -- --nocapture

.PHONY: type-check
type-check: ## cargo check (type check only, fast)
	$(RTK) cargo check --workspace

.PHONY: test
test: test-impacted ## Run impacted unit tests (local default)


.PHONY: test-core
test-core: ## Run tests for katana-core only
	$(RTK) cargo test -p katana-core

.PHONY: test-ui
test-ui: ## Run tests for katana-ui only
	$(RTK) cargo test -p katana-ui

.PHONY: test-verbose
test-verbose: ## Run tests with verbose output
	$(RTK) cargo test --workspace -- --nocapture

.PHONY: test-specific
test-specific: ## Run a specific test (e.g., make test-specific T=test_name)
	$(RTK) cargo test --workspace -- $(T)

.PHONY: test-integration
test-integration: ## Run integration tests — fixture tests only (requires: egui_kittest)
	$(RTK) cargo test -j $(JOBS) -q --workspace --test ui_integration_fixture -- --test-threads=$(JOBS) fixture

.PHONY: check-linux
check-linux: ## Verify test execution in isolated Linux environment
	$(RTK) docker compose -f platforms/linux/ci/compose.yml run --rm -e RUSTFLAGS="$(RUSTFLAGS) -C link-arg=-fuse-ld=lld" ubuntu-test bash -c "cargo sweep --time 7 && cargo test -q --workspace"

.PHONY: check-windows
check-windows: ## Verify Windows cross-compilation without running tests
	$(RTK) docker compose -f platforms/windows/ci/compose.yml run --rm windows-test bash -c "cargo sweep --time 7 && cargo xwin check -q --workspace --target x86_64-pc-windows-msvc --tests"

.PHONY: check-platforms
check-platforms: check-linux check-windows ## Verify test/compilation across all target platforms (Linux, Windows)

.PHONY: coverage
coverage: ## Run tests and verify 100% test coverage (requires cargo-llvm-cov)
	JOBS=$(JOBS) $(RTK) scripts/ci/coverage.sh

.PHONY: check-light
check-light: sweep fmt-check lint-impacted ## Quick verification (skip slow fixture tests; ast-lint runs inside cargo test)
	$(RTK) scripts/runner/impacted.py test -- --skip fixture
	@echo "✅ Light checks passed"


.PHONY: check
check: sweep fmt-check lint-impacted test check-platforms ## Fast impacted verification (local default)
	@echo "✅ All checks passed"

.PHONY: check-local
check-local: sweep fmt lint test-integration coverage check-platforms ## Full local verification incl. cross-platform checks


	@echo "✅ All checks passed"

.PHONY: pre-push
pre-push: check-full ## Pre-push hook equivalent checks

###################################
# Documentation / Analysis
###################################

.PHONY: doc
doc: ## Generate API documentation
	$(RTK) cargo doc --workspace --no-deps

.PHONY: doc-open
doc-open: ## Generate & open API documentation in browser
	$(RTK) cargo doc --workspace --no-deps --open

.PHONY: bloat
bloat: ## Binary size analysis (requires cargo-bloat)
	$(RTK) cargo bloat --release --bin KatanA

.PHONY: loc
loc: ## Count lines of code (requires tokei)
	$(RTK) tokei crates/

.PHONY: tree
tree: ## Display dependency tree
	$(RTK) cargo tree --workspace

###################################
# Release / Packaging
###################################

.PHONY: package-mac
package-mac: ## Build macOS .app bundle (release)
	scripts/build/package-mac.sh $(VERSION)

.PHONY: package-linux
package-linux: ## Build Linux zip artifact
	scripts/build/package-linux.sh

.PHONY: package-windows
package-windows: ## Build Windows MSI and ZIP artifacts
	scripts/build/package-windows.sh

.PHONY: dmg
dmg: package-mac ## Build macOS .dmg installer from .app bundle
	@FORCE=$(FORCE) scripts/build/dmg.sh $(VERSION)

.PHONY: release
release: ## Prepare for release by bumping version in all files (usage: make release VERSION=x.y.z)
ifndef VERSION
	$(error VERSION is required. Usage: make release VERSION=x.y.z)
endif
	@scripts/release/bump-version.sh $(VERSION)
	@echo ""
	@echo "✅ Version bump completed!"
	@echo "   Next steps:"
	@echo "   1. Review changes (git diff)"
	@echo "   2. Commit with -S (Verified Commit): git commit -S -m \"release: v$(VERSION)\""
	@echo "   3. Create a PR to master"

.PHONY: release-trigger
release-trigger: ## Manually trigger the release workflow on GitHub Actions (emergency use only)
ifndef VERSION
	$(error VERSION is required. Usage: make release-trigger VERSION=x.y.z)
endif
	@echo "Triggering GitHub Actions release workflow for v$(VERSION)..."
	@BOOL_FORCE="false"; \
	if [ "$(FORCE)" = "1" ]; then \
		BOOL_FORCE="true"; \
		echo "⚠️ Force mode enabled (clobbering existing release)"; \
	fi; \
	gh workflow run Release -f version=$(VERSION) -f target=all -f force=$$BOOL_FORCE
	@echo ""
	@echo "✅ Workflow triggered! Monitor progress with:"
	@echo "   gh run watch --repo HiroyukiFuruno/KatanA"

.PHONY: release-preflight
release-preflight: ## Run preflight release checks without publishing (usage: make release-preflight VERSION=x.y.z)
ifndef VERSION
	$(error VERSION is required. Usage: make release-preflight VERSION=x.y.z)
endif
	@scripts/release/preflight.sh $(VERSION)

###################################
# Maintenance
###################################

.PHONY: sweep
sweep: ## Sweep old build artifacts locally (older than 7 days)
	@$(RTK) cargo sweep --time 7 || true

.PHONY: clean
clean: sweep ## Remove build artifacts
	cargo clean

.PHONY: update-safe
update-safe: ## Update dependency crates safely (respects Cargo.toml SemVer)
	$(RTK) cargo update

.PHONY: update
update: ## Upgrade ALL dependencies to absolute latest versions (including breaking changes)
	$(RTK) cargo upgrade -i
	$(RTK) cargo update

.PHONY: outdated
outdated: ## List outdated dependencies (requires cargo-outdated)
	@cp Cargo.toml Cargo.toml.bak
	@sed -e '/^\[patch\.crates-io\]/,$$d' Cargo.toml.bak > Cargo.toml
	@$(RTK) cargo outdated --workspace || (mv Cargo.toml.bak Cargo.toml && exit 1)
	@mv Cargo.toml.bak Cargo.toml

###################################
# Help
###################################

.PHONY: help
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'

###################################
# Linux Verification
###################################

.PHONY: linux-up
linux-up: ## Start the Linux verification environment
	bash platforms/linux/init.sh

.PHONY: linux-down
linux-down: ## Stop the Linux verification environment
	bash platforms/linux/down.sh

.PHONY: test-impacted
test-impacted: ## Run tests for impacted packages only
	$(RTK) scripts/runner/impacted.py test -- --skip fixture

.PHONY: lint-impacted
lint-impacted: ## Run Clippy for impacted packages only
	$(RTK) scripts/runner/impacted.py clippy

.PHONY: check-full
check-full: fmt-check lint test-integration coverage check-platforms ## Full verification (fmt + clippy + fixture IT + 100% coverage; ast-lint runs inside coverage)
	@echo "✅ All checks passed"
