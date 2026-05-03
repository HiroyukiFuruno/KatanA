# ============================================================
# Mermaid / Draw.io
# ============================================================

# Install Playwright Chromium for official Mermaid reference rendering
mermaid-diagram-browser-install:
    {{PLAYWRIGHT}} install chromium

# Split source Markdown into one-file-per-diagram Mermaid parts
mermaid-parts-split lang:
    @case "{{lang}}" in en|ja) ;; *) echo "lang must be en or ja"; exit 1;; esac
    @source_path=$(case "{{lang}}" in en) printf '%s' "{{MERMAID_EN_SOURCE}}" ;; ja) printf '%s' "{{MERMAID_JA_SOURCE}}" ;; esac); \
      bun run scripts/mermaid/split-markdown-fixtures.ts --input "$source_path" --output "{{MERMAID_PARTS_ROOT}}/{{lang}}"

# Render official Mermaid reference images for Mermaid parts
mermaid-parts-reference lang theme='dark': (mermaid-parts-split lang)
    @case "{{theme}}" in light|dark) ;; *) echo "theme must be light or dark"; exit 1;; esac
    bun run scripts/mermaid/diagram-update.ts --fixtures "{{MERMAID_PARTS_ROOT}}/{{lang}}" --output "{{MERMAID_PARTS_TMP_PREFIX}}-{{lang}}-{{theme}}-official" --markdown-output "{{MERMAID_PARTS_ROOT}}/{{lang}}/official-{{theme}}" --theme "{{theme}}" --mermaid-js "{{MERMAID_JS}}" --skip-errors

# Render KatanA Mermaid images for Mermaid parts
mermaid-parts-katana lang theme='dark': (mermaid-parts-split lang)
    @case "{{theme}}" in light|dark) ;; *) echo "theme must be light or dark"; exit 1;; esac
    {{CARGO}} run -p katana-core --example render_mermaid_fixtures -- --fixtures "{{MERMAID_PARTS_ROOT}}/{{lang}}" --output "{{MERMAID_PARTS_TMP_PREFIX}}-{{lang}}-{{theme}}-katana" --theme "{{theme}}" --skip-errors

# Compare official Mermaid reference images with KatanA renderer images
mermaid-parts-compare lang theme='dark' baseline='': (mermaid-parts-reference lang theme) (mermaid-parts-katana lang theme)
    @case "{{theme}}" in light|dark) ;; *) echo "theme must be light or dark"; exit 1;; esac
    bun run scripts/mermaid/rasterize-svg-dir.ts --input "{{MERMAID_PARTS_TMP_PREFIX}}-{{lang}}-{{theme}}-official" --output "{{MERMAID_PARTS_TMP_PREFIX}}-{{lang}}-{{theme}}-official-browser" --theme "{{theme}}"
    bun run scripts/mermaid/rasterize-svg-dir.ts --input "{{MERMAID_PARTS_TMP_PREFIX}}-{{lang}}-{{theme}}-katana" --output "{{MERMAID_PARTS_TMP_PREFIX}}-{{lang}}-{{theme}}-katana-browser" --theme "{{theme}}"
    bun run scripts/mermaid/reference-compare.ts --official "{{MERMAID_PARTS_TMP_PREFIX}}-{{lang}}-{{theme}}-official-browser" --katana "{{MERMAID_PARTS_TMP_PREFIX}}-{{lang}}-{{theme}}-katana-browser" --output "{{MERMAID_PARTS_TMP_PREFIX}}-{{lang}}-{{theme}}-comparison" --theme "{{theme}}" --min-score "{{MERMAID_MIN_SCORE}}" $(if [ -n "{{baseline}}" ]; then echo "--baseline {{baseline}}"; fi)

# Compare Mermaid parts in both light and dark themes by language
mermaid-parts-compare-en: (mermaid-parts-compare "en" "light") (mermaid-parts-compare "en" "dark")
mermaid-parts-compare-ja:
    just mermaid-parts-compare-en
    just mermaid-parts-compare "ja" "light" "{{MERMAID_PARTS_TMP_PREFIX}}-en-light-comparison/scores.json"
    just mermaid-parts-compare "ja" "dark" "{{MERMAID_PARTS_TMP_PREFIX}}-en-dark-comparison/scores.json"

# Legacy aliases now point to the English Mermaid parts flow
mermaid-diagram-update: (mermaid-parts-reference "en" "dark")
mermaid-diagum-update: mermaid-diagram-update

# Render KatanA Mermaid fixture images without browser screenshots
mermaid-diagram-katana-update: (mermaid-parts-katana "en" "dark")

# Compare official Mermaid reference images with KatanA renderer images
mermaid-diagram-compare: (mermaid-parts-compare-en)

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
