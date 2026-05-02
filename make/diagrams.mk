###################################
# Mermaid / Draw.io
###################################

.PHONY: mermaid-diagram-browser-install
mermaid-diagram-browser-install: ## Install Playwright Chromium for official Mermaid reference rendering
	$(PLAYWRIGHT) install chromium

.PHONY: mermaid-diagram-update mermaid-diagum-update
mermaid-diagram-update: ## Update official Mermaid reference images and fixture image links
	bun run scripts/mermaid/diagram-update.ts --fixtures $(MERMAID_FIXTURES) --output $(MERMAID_REFERENCE_DIR) --mermaid-js $(MERMAID_JS)

mermaid-diagum-update: mermaid-diagram-update ## Alias for mermaid-diagram-update

.PHONY: mermaid-diagram-katana-update
mermaid-diagram-katana-update: ## Render KatanA Mermaid fixture images without browser screenshots
	cargo run -p katana-core --example render_mermaid_fixtures -- --fixtures $(MERMAID_FIXTURES) --output $(MERMAID_KATANA_DIR)

.PHONY: mermaid-diagram-compare
mermaid-diagram-compare: mermaid-diagram-katana-update ## Compare official Mermaid reference images with KatanA renderer images
	bun run scripts/mermaid/rasterize-svg-dir.ts --input $(MERMAID_REFERENCE_DIR) --output $(MERMAID_REFERENCE_BROWSER_DIR)
	bun run scripts/mermaid/rasterize-svg-dir.ts --input $(MERMAID_KATANA_DIR) --output $(MERMAID_KATANA_BROWSER_DIR)
	bun run scripts/mermaid/reference-compare.ts --official $(MERMAID_REFERENCE_BROWSER_DIR) --katana $(MERMAID_KATANA_BROWSER_DIR) --min-score $(MERMAID_MIN_SCORE)

.PHONY: mermaid-sample-fixtures
mermaid-sample-fixtures: ## Split assets/fixtures/mermaid.md into temporary single-diagram fixtures
	bun run scripts/mermaid/split-markdown-fixtures.ts --input $(MERMAID_SAMPLE) --output $(MERMAID_SAMPLE_FIXTURES_DIR)

.PHONY: mermaid-sample-reference-update
mermaid-sample-reference-update: mermaid-sample-fixtures ## Render official Mermaid reference images for assets/fixtures/mermaid.md
	bun run scripts/mermaid/diagram-update.ts --fixtures $(MERMAID_SAMPLE_FIXTURES_DIR) --output $(MERMAID_SAMPLE_REFERENCE_DIR) --mermaid-js $(MERMAID_JS) --no-write-md --skip-errors

.PHONY: mermaid-sample-katana-update
mermaid-sample-katana-update: mermaid-sample-fixtures ## Render KatanA images for assets/fixtures/mermaid.md
	cargo run -p katana-core --example render_mermaid_fixtures -- --fixtures $(MERMAID_SAMPLE_FIXTURES_DIR) --output $(MERMAID_SAMPLE_KATANA_DIR) --skip-errors

.PHONY: mermaid-sample-compare
mermaid-sample-compare: ## Compare assets/fixtures/mermaid.md with the same scoring path
	$(MAKE) mermaid-sample-reference-update
	$(MAKE) mermaid-sample-katana-update
	bun run scripts/mermaid/rasterize-svg-dir.ts --input $(MERMAID_SAMPLE_REFERENCE_DIR) --output $(MERMAID_SAMPLE_REFERENCE_BROWSER_DIR)
	bun run scripts/mermaid/rasterize-svg-dir.ts --input $(MERMAID_SAMPLE_KATANA_DIR) --output $(MERMAID_SAMPLE_KATANA_BROWSER_DIR)
	bun run scripts/mermaid/reference-compare.ts --official $(MERMAID_SAMPLE_REFERENCE_BROWSER_DIR) --katana $(MERMAID_SAMPLE_KATANA_BROWSER_DIR) --output $(MERMAID_SAMPLE_COMPARISON_DIR) --min-score $(MERMAID_MIN_SCORE)

.PHONY: drawio-diagram-browser-install
drawio-diagram-browser-install: ## Install Playwright Chromium for official Draw.io reference rendering
	$(PLAYWRIGHT) install chromium

.PHONY: drawio-diagram-update
drawio-diagram-update: ## Update official Draw.io reference images
	bun run scripts/drawio/diagram-update.ts --fixtures $(DRAWIO_FIXTURES) --output $(DRAWIO_REFERENCE_DIR) --drawio-js $(DRAWIO_JS)

.PHONY: drawio-diagram-katana-update
drawio-diagram-katana-update: ## Render KatanA Draw.io fixture images without browser screenshots
	cargo run -p katana-core --example render_drawio_fixtures -- --fixtures $(DRAWIO_FIXTURES) --output $(DRAWIO_KATANA_DIR)

.PHONY: drawio-diagram-compare
drawio-diagram-compare: drawio-diagram-katana-update ## Compare official Draw.io reference images with KatanA renderer images
	bun run scripts/mermaid/rasterize-svg-dir.ts --input $(DRAWIO_KATANA_DIR) --output $(DRAWIO_KATANA_BROWSER_DIR)
	bun run scripts/drawio/reference-compare.ts --official $(DRAWIO_REFERENCE_DIR) --katana $(DRAWIO_KATANA_BROWSER_DIR) --output $(DRAWIO_COMPARISON_DIR) --min-score $(DRAWIO_MIN_SCORE)

.PHONY: drawio-resource-update
drawio-resource-update: ## Update embedded Draw.io resource manifest
	bun run scripts/drawio/resource-update.ts --resources $(DRAWIO_RESOURCE_DIR) --manifest $(DRAWIO_RESOURCE_MANIFEST)

.PHONY: drawio-resource-audit
drawio-resource-audit: ## Verify fixture Draw.io external resources are embedded
	bun run scripts/drawio/resource-audit.ts --fixtures $(DRAWIO_RESOURCE_FIXTURES) --resources $(DRAWIO_RESOURCE_DIR) --manifest $(DRAWIO_RESOURCE_MANIFEST)
