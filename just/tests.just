# ============================================================
# Tests / Checks
# ============================================================

# Run AST-based custom linters (comment style, etc.)
ast-lint:
    {{CARGO}} test -j {{JOBS}} -p katana-linter ast_linter -- --nocapture

# cargo check (type check only, fast)
type-check:
    {{CARGO}} check --workspace

# Run impacted unit tests (local default)
test: test-impacted

# Run tests for katana-core only
test-core:
    {{CARGO}} test -p katana-core

# Run tests for katana-ui only
test-ui:
    {{CARGO}} test -p katana-ui

# Run tests with verbose output
test-verbose:
    {{CARGO}} test --workspace -- --nocapture

# Run a specific test (e.g., just T=test_name test-specific)
test-specific:
    {{CARGO}} test --workspace -- {{T}}

# Run integration tests - fixture tests only (requires: egui_kittest)
test-integration:
    {{CARGO}} test -j {{JOBS}} -q --workspace --test ui_integration_fixture -- --test-threads={{JOBS}} fixture

# Verify test execution in isolated Linux environment
check-linux:
    {{DOCKER}} compose -f platforms/linux/ci/compose.yml run --rm -e RUSTFLAGS="{{RUSTFLAGS}} -C link-arg=-fuse-ld=lld" ubuntu-test bash -c "cargo sweep --time 7 && cargo test -q --workspace"

# Verify Windows cross-compilation without running tests
check-windows:
    {{DOCKER}} compose -f platforms/windows/ci/compose.yml run --rm windows-test bash -c "cargo sweep --time 7 && cargo xwin check -q --workspace --target x86_64-pc-windows-msvc --tests"

# Verify test/compilation across all target platforms (Linux, Windows)
check-platforms: check-linux check-windows

# Run tests and verify 100% test coverage (requires cargo-llvm-cov)
coverage:
    JOBS={{JOBS}} {{RTK_CMD}}scripts/ci/coverage.sh

# Quick verification (skip slow fixture tests; ast-lint runs inside cargo test)
check-light: sweep fmt-check biome-js-lint lint-impacted
    {{RTK_CMD}}scripts/runner/impacted.py test -- --skip fixture
    @echo "✅ Light checks passed"

# Fast impacted verification (local default)
check: sweep fmt-check biome-js-lint lint-impacted test check-platforms
    @echo "✅ All checks passed"

# Full local verification incl. cross-platform checks
check-local: sweep fmt lint test-integration coverage check-platforms
    @echo "✅ All checks passed"

# Pre-push hook equivalent checks
pre-push: check-full
