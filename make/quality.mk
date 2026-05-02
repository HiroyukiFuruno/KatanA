###################################
# Quality
###################################

.PHONY: fmt locale-json-fmt biome-js-fmt biome-json-fmt
fmt: biome-js-fmt biome-json-fmt ## Apply code formatting (rustfmt + Biome)
	$(RTK) cargo fmt --all

.PHONY: locale-json-fmt
locale-json-fmt:
	rtk err $(BIOME) format --indent-style space --indent-width 2 --write crates/katana-ui/locales/*.json

.PHONY: biome-js-fmt
biome-js-fmt: ## Apply Biome formatting to renderer JavaScript/TypeScript and Mermaid scripts
	$(BIOME) format --write $(BIOME_JS_TS_FILES)

.PHONY: biome-json-fmt
biome-json-fmt: ## Apply Biome formatting to repository JSON/JSONC files
	$(BIOME) format --write $(BIOME_JSON_FILES)

.PHONY: fmt-check
fmt-check: biome-js-fmt-check biome-json-fmt-check ## Check format differences (for CI)
	$(RTK) cargo fmt --all -- --check

.PHONY: biome-js-fmt-check
biome-js-fmt-check: ## Check Biome formatting for renderer JavaScript/TypeScript and Mermaid scripts
	$(BIOME) format $(BIOME_JS_TS_FILES)

.PHONY: biome-json-fmt-check
biome-json-fmt-check: ## Check Biome formatting for repository JSON/JSONC files
	$(BIOME) format $(BIOME_JSON_FILES)

.PHONY: lint biome-js-lint
lint: biome-js-lint ## Run Clippy (forces zero warnings)
	$(RTK) cargo clippy -j $(JOBS) --workspace -- -D warnings

.PHONY: biome-js-lint
biome-js-lint: ## Run Biome lint for renderer JavaScript and Mermaid scripts
	$(BIOME) lint --error-on-warnings $(BIOME_JS_TS_FILES)

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
