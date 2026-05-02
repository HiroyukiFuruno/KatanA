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
