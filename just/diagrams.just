# ============================================================
# Mermaid / Draw.io
# ============================================================

# Install Playwright Chromium for official Mermaid reference rendering
mermaid-diagram-browser-install:
    {{PLAYWRIGHT}} install chromium

# Update official Mermaid reference images and fixture image links
mermaid-diagram-update:
    bun run scripts/mermaid/diagram-update.ts --fixtures {{MERMAID_FIXTURES}} --output {{MERMAID_REFERENCE_DIR}} --mermaid-js {{MERMAID_JS}}

# Alias for mermaid-diagram-update
mermaid-diagum-update: mermaid-diagram-update

# Render KatanA Mermaid fixture images without browser screenshots
mermaid-diagram-katana-update:
    cargo run -p katana-core --example render_mermaid_fixtures -- --fixtures {{MERMAID_FIXTURES}} --output {{MERMAID_KATANA_DIR}}

# Compare official Mermaid reference images with KatanA renderer images
mermaid-diagram-compare: mermaid-diagram-katana-update
    bun run scripts/mermaid/rasterize-svg-dir.ts --input {{MERMAID_REFERENCE_DIR}} --output {{MERMAID_REFERENCE_BROWSER_DIR}}
    bun run scripts/mermaid/rasterize-svg-dir.ts --input {{MERMAID_KATANA_DIR}} --output {{MERMAID_KATANA_BROWSER_DIR}}
    bun run scripts/mermaid/reference-compare.ts --official {{MERMAID_REFERENCE_BROWSER_DIR}} --katana {{MERMAID_KATANA_BROWSER_DIR}} --min-score {{MERMAID_MIN_SCORE}}

# Split assets/fixtures/mermaid.md into temporary single-diagram fixtures
mermaid-sample-fixtures:
    bun run scripts/mermaid/split-markdown-fixtures.ts --input {{MERMAID_SAMPLE}} --output {{MERMAID_SAMPLE_FIXTURES_DIR}}

# Render official Mermaid reference images for assets/fixtures/mermaid.md
mermaid-sample-reference-update: mermaid-sample-fixtures
    bun run scripts/mermaid/diagram-update.ts --fixtures {{MERMAID_SAMPLE_FIXTURES_DIR}} --output {{MERMAID_SAMPLE_REFERENCE_DIR}} --mermaid-js {{MERMAID_JS}} --no-write-md --skip-errors

# Render KatanA images for assets/fixtures/mermaid.md
mermaid-sample-katana-update: mermaid-sample-fixtures
    cargo run -p katana-core --example render_mermaid_fixtures -- --fixtures {{MERMAID_SAMPLE_FIXTURES_DIR}} --output {{MERMAID_SAMPLE_KATANA_DIR}} --skip-errors

# Compare assets/fixtures/mermaid.md with the same scoring path
mermaid-sample-compare: mermaid-sample-reference-update mermaid-sample-katana-update
    bun run scripts/mermaid/rasterize-svg-dir.ts --input {{MERMAID_SAMPLE_REFERENCE_DIR}} --output {{MERMAID_SAMPLE_REFERENCE_BROWSER_DIR}}
    bun run scripts/mermaid/rasterize-svg-dir.ts --input {{MERMAID_SAMPLE_KATANA_DIR}} --output {{MERMAID_SAMPLE_KATANA_BROWSER_DIR}}
    bun run scripts/mermaid/reference-compare.ts --official {{MERMAID_SAMPLE_REFERENCE_BROWSER_DIR}} --katana {{MERMAID_SAMPLE_KATANA_BROWSER_DIR}} --output {{MERMAID_SAMPLE_COMPARISON_DIR}} --min-score {{MERMAID_MIN_SCORE}}

# Install Playwright Chromium for official Draw.io reference rendering
drawio-diagram-browser-install:
    {{PLAYWRIGHT}} install chromium

# Update official Draw.io reference images
drawio-diagram-update:
    bun run scripts/drawio/diagram-update.ts --fixtures {{DRAWIO_FIXTURES}} --output {{DRAWIO_REFERENCE_DIR}} --drawio-js {{DRAWIO_JS}}

# Render KatanA Draw.io fixture images without browser screenshots
drawio-diagram-katana-update:
    cargo run -p katana-core --example render_drawio_fixtures -- --fixtures {{DRAWIO_FIXTURES}} --output {{DRAWIO_KATANA_DIR}}

# Compare official Draw.io reference images with KatanA renderer images
drawio-diagram-compare: drawio-diagram-katana-update
    bun run scripts/mermaid/rasterize-svg-dir.ts --input {{DRAWIO_KATANA_DIR}} --output {{DRAWIO_KATANA_BROWSER_DIR}}
    bun run scripts/drawio/reference-compare.ts --official {{DRAWIO_REFERENCE_DIR}} --katana {{DRAWIO_KATANA_BROWSER_DIR}} --output {{DRAWIO_COMPARISON_DIR}} --min-score {{DRAWIO_MIN_SCORE}}

# Update embedded Draw.io resource manifest
drawio-resource-update:
    bun run scripts/drawio/resource-update.ts --resources {{DRAWIO_RESOURCE_DIR}} --manifest {{DRAWIO_RESOURCE_MANIFEST}}

# Verify fixture Draw.io external resources are embedded
drawio-resource-audit:
    bun run scripts/drawio/resource-audit.ts --fixtures {{DRAWIO_RESOURCE_FIXTURES}} --resources {{DRAWIO_RESOURCE_DIR}} --manifest {{DRAWIO_RESOURCE_MANIFEST}}
