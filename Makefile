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

# Force all warnings to be treated as errors for every cargo command run via make
export RUSTFLAGS=-D warnings

VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')

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
run-release: ## Run the application in release mode
	cargo run --bin KatanA --release

.PHONY: run-performance
run-performance: ## Run in release mode with FPS monitor logging
	RUST_LOG=warn cargo run --bin KatanA --release

.PHONY: run
run: build ## Run the application (KatanA)
	cargo run --bin KatanA

.PHONY: watch
watch: ## Watch file changes & auto check (requires cargo-watch)
	cargo watch -x 'check --workspace' -x 'test --workspace'

.PHONY: watch-run
watch-run: ## Watch file changes & auto restart (requires cargo-watch)
	cargo watch -x 'run --bin KatanA'

###################################
# Build
###################################

.PHONY: build
build: ## Build the entire workspace (debug)
	cargo build --workspace

.PHONY: build-release
build-release: ## Release build (optimized)
	cargo build --workspace --release

###################################
# Quality
###################################

.PHONY: fmt
fmt: ## Apply code formatting (rustfmt)
	cargo fmt --all

.PHONY: fmt-check
fmt-check: ## Check format differences (for CI)
	cargo fmt --all -- --check

.PHONY: lint
lint: ## Run Clippy (forces zero warnings)
	cargo clippy --workspace -- -D warnings

.PHONY: lint-fix
lint-fix: ## Run Clippy and apply automatic fixes
	cargo clippy --workspace --fix --allow-dirty --allow-staged -- -D warnings

.PHONY: ast-lint
ast-lint: ## Run AST-based custom linters (comment style, etc.)
	cargo test -p katana-linter ast_linter -- --nocapture

.PHONY: typecheck
typecheck: ## cargo check (type check only, fast)
	cargo check --workspace

.PHONY: test
test: ## Run all unit tests
	cargo test --workspace

.PHONY: test-core
test-core: ## Run tests for katana-core only
	cargo test -p katana-core

.PHONY: test-ui
test-ui: ## Run tests for katana-ui only
	cargo test -p katana-ui

.PHONY: test-verbose
test-verbose: ## Run tests with verbose output
	cargo test --workspace -- --nocapture

.PHONY: test-specific
test-specific: ## Run a specific test (e.g., make test-specific T=test_name)
	cargo test --workspace -- $(T)

.PHONY: test-integration
test-integration: ## Run integration tests (UI tests, semantic assertions only) (requires: egui_kittest)
	cargo test -q --workspace --test integration_tests -- --test-threads=1

.PHONY: check-linux
check-linux: ## Verify test execution in isolated Linux environment
	docker compose -f platforms/linux/ci/compose.yml run --rm -e RUSTFLAGS="$(RUSTFLAGS) -C link-arg=-fuse-ld=lld" ubuntu-test cargo test --workspace

.PHONY: check-windows
check-windows: ## Verify Windows cross-compilation without running tests
	docker compose -f platforms/windows/ci/compose.yml run --rm windows-test cargo xwin check --workspace --target x86_64-pc-windows-msvc --tests

.PHONY: check-platforms
check-platforms: check-linux check-windows ## Verify test/compilation across all target platforms (Linux, Windows)

.PHONY: coverage
coverage: ## Run tests and verify 100% test coverage (requires cargo-llvm-cov)
	scripts/ci/coverage.sh

.PHONY: check-light
check-light: fmt-check lint ast-lint ## Quick verification (skip slow fixture tests)
	cargo test --workspace -- --skip fixture
	@echo "✅ Light checks passed"


.PHONY: check
check: fmt-check lint ast-lint test-integration coverage check-platforms ## Full verification (fmt + clippy + AST lint + IT + 100% coverage enforced)
	@echo "✅ All checks passed"

.PHONY: check-local
check-local: fmt lint ast-lint test-integration coverage check-platforms ## Full verification including cross-platform checks (Windows, Linux)
	@echo "✅ All checks passed"

.PHONY: pre-push
pre-push: check ## Pre-push hook equivalent checks

###################################
# Documentation / Analysis
###################################

.PHONY: doc
doc: ## Generate API documentation
	cargo doc --workspace --no-deps

.PHONY: doc-open
doc-open: ## Generate & open API documentation in browser
	cargo doc --workspace --no-deps --open

.PHONY: bloat
bloat: ## Binary size analysis (requires cargo-bloat)
	cargo bloat --release --bin KatanA

.PHONY: loc
loc: ## Count lines of code (requires tokei)
	tokei crates/

.PHONY: tree
tree: ## Display dependency tree
	cargo tree --workspace

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
release: ## Trigger the release workflow on GitHub Actions (usage: make release VERSION=x.y.z FORCE=1)
ifndef VERSION
	$(error VERSION is required. Usage: make release VERSION=x.y.z)
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

.PHONY: clean
clean: ## Remove build artifacts
	cargo clean

.PHONY: update-safe
update-safe: ## Update dependency crates safely (respects Cargo.toml SemVer)
	cargo update

.PHONY: update
update: ## Upgrade ALL dependencies to absolute latest versions (including breaking changes)
	cargo upgrade -i
	cargo update

.PHONY: outdated
outdated: ## List outdated dependencies (requires cargo-outdated)
	@cp Cargo.toml Cargo.toml.bak
	@sed -e '/^\[patch\.crates-io\]/,$$d' Cargo.toml.bak > Cargo.toml
	@cargo outdated --workspace || (mv Cargo.toml.bak Cargo.toml && exit 1)
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
	./platforms/linux/init.sh

.PHONY: linux-down
linux-down: ## Stop the Linux verification environment
	./platforms/linux/down.sh
