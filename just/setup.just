# ============================================================
# Setup / Run / Build
# ============================================================

# Bootstrap the development environment interactively
init:
    scripts/setup/setup.sh

# Run the application in release mode
run-release: sweep
    {{CARGO}} run --bin KatanA --release

# Run in release mode with FPS monitor logging
run-performance: sweep
    RUST_LOG=warn {{CARGO}} run --bin KatanA --release

# Run the application (KatanA)
run: sweep build
    {{CARGO}} run --bin KatanA

# Watch file changes & auto check (requires cargo-watch)
watch: sweep
    {{CARGO}} watch -x 'check --workspace' -x 'test --workspace'

# Watch file changes & auto restart (requires cargo-watch)
watch-run: sweep
    {{CARGO}} watch -x 'run --bin KatanA'

# Build the entire workspace (debug)
build: sweep
    {{CARGO}} build --workspace

# Release build (optimized)
build-release: sweep
    {{CARGO}} build --workspace --release
