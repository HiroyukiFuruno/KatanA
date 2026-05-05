# ============================================================
# Quality
# ============================================================

# Apply code formatting (rustfmt + Biome)
fmt: biome-js-fmt biome-json-fmt
    {{CARGO}} fmt --all

locale-json-fmt:
    rtk err {{BIOME}} format --indent-style space --indent-width 2 --write crates/katana-ui/locales/*.json

# Apply Biome formatting to JavaScript/TypeScript files when configured
biome-js-fmt:
    @if [ -n "{{BIOME_JS_TS_FILES}}" ]; then {{BIOME}} format --write {{BIOME_JS_TS_FILES}}; fi

# Apply Biome formatting to repository JSON/JSONC files
biome-json-fmt:
    {{BIOME}} format --write {{BIOME_JSON_FILES}}

# Check format differences (for CI)
fmt-check: biome-js-fmt-check biome-json-fmt-check
    {{CARGO}} fmt --all -- --check

# Check Biome formatting for JavaScript/TypeScript files when configured
biome-js-fmt-check:
    @if [ -n "{{BIOME_JS_TS_FILES}}" ]; then {{BIOME}} format {{BIOME_JS_TS_FILES}}; fi

# Check Biome formatting for repository JSON/JSONC files
biome-json-fmt-check:
    {{BIOME}} format {{BIOME_JSON_FILES}}

# Run Clippy (forces zero warnings)
lint: biome-js-lint
    {{CARGO}} clippy -j {{JOBS}} --workspace -- -D warnings

# Run Biome lint for JavaScript/TypeScript files when configured
biome-js-lint:
    @if [ -n "{{BIOME_JS_TS_FILES}}" ]; then {{BIOME}} lint --error-on-warnings {{BIOME_JS_TS_FILES}}; fi

# Run Clippy and apply automatic fixes
lint-fix:
    {{CARGO}} clippy --workspace --fix --allow-dirty --allow-staged -- -D warnings

# Install KML CLI and MCP server (katana-markdown-linter)
kml-install:
    {{CARGO}} install katana-markdown-linter --version {{KML_VERSION}} --features {{KML_INSTALL_FEATURES}} --force {{KML_INSTALL_FLAGS}}

kml-require:
    @command -v {{KML}} >/dev/null || { echo "kml not found. Run: just kml-install"; exit 127; }

kml-mcp-require:
    @command -v {{KML_MCP}} >/dev/null || { echo "kml-mcp not found. Run: just kml-install"; exit 127; }

# Run KML markdown lint without modifying files
kml-check: kml-require
    {{KML}} check {{KML_CHECK_ARGS}}

# Run KML markdown lint and print JSON diagnostics
kml-check-json: kml-require
    {{KML}} check --output json {{KML_CHECK_ARGS}}

# Apply KML safe Markdown fixes
kml-fix: kml-require
    {{KML}} fix {{KML_CHECK_ARGS}}

# Start the KML MCP server over stdio
kml-mcp: kml-mcp-require
    {{KML_MCP}}

# Verify OpenSpec CLI through the repo wrapper
openspec-require:
    @{{OPENSPEC}} --version >/dev/null

# List active OpenSpec changes
openspec-list: openspec-require
    {{OPENSPEC}} list
