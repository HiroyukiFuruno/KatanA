# ============================================================
# Maintenance
# ============================================================

# Sweep old build artifacts locally (older than 7 days)
sweep:
    @{{CARGO}} sweep --time 7 || true

# Remove build artifacts
clean: sweep
    cargo clean

# Update dependency crates safely (respects Cargo.toml SemVer)
update-safe:
    {{CARGO}} update

# Upgrade ALL dependencies to absolute latest versions (including breaking changes)
update:
    {{CARGO}} upgrade -i
    {{CARGO}} update

# List outdated dependencies (requires cargo-outdated)
outdated:
    @cp Cargo.toml Cargo.toml.bak
    @sed -e '/^\[patch\.crates-io\]/,$d' Cargo.toml.bak > Cargo.toml
    @{{CARGO}} outdated --workspace || (mv Cargo.toml.bak Cargo.toml && exit 1)
    @mv Cargo.toml.bak Cargo.toml

# Start the Linux verification environment
linux-up:
    bash platforms/linux/init.sh

# Stop the Linux verification environment
linux-down:
    bash platforms/linux/down.sh

# Run tests for impacted packages only
test-impacted:
    {{RTK_CMD}}scripts/runner/impacted.py test -- --skip fixture

# Run Clippy for impacted packages only
lint-impacted:
    {{RTK_CMD}}scripts/runner/impacted.py clippy

# Full verification (fmt + clippy + fixture IT + 100% coverage; ast-lint runs inside coverage)
check-full: fmt-check lint test-integration coverage check-platforms
    @echo "✅ All checks passed"
