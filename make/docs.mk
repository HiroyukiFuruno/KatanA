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
