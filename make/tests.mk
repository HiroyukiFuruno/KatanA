###################################
# Tests / Checks
###################################

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
check-light: sweep fmt-check biome-js-lint lint-impacted ## Quick verification (skip slow fixture tests; ast-lint runs inside cargo test)
	$(RTK) scripts/runner/impacted.py test -- --skip fixture
	@echo "✅ Light checks passed"

.PHONY: check
check: sweep fmt-check biome-js-lint lint-impacted test check-platforms ## Fast impacted verification (local default)
	@echo "✅ All checks passed"

.PHONY: check-local
check-local: sweep fmt lint test-integration coverage check-platforms ## Full local verification incl. cross-platform checks
	@echo "✅ All checks passed"

.PHONY: pre-push
pre-push: check-full ## Pre-push hook equivalent checks

.PHONY: test-impacted
test-impacted: ## Run tests for impacted packages only
	$(RTK) scripts/runner/impacted.py test -- --skip fixture

.PHONY: lint-impacted
lint-impacted: ## Run Clippy for impacted packages only
	$(RTK) scripts/runner/impacted.py clippy

.PHONY: check-full
check-full: fmt-check lint test-integration coverage check-platforms ## Full verification (fmt + clippy + fixture IT + 100% coverage; ast-lint runs inside coverage)
	@echo "✅ All checks passed"
