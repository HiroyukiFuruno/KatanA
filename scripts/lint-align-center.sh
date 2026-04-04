#!/usr/bin/env bash
# ============================================================
# lint-align-center.sh
# Detect ui.horizontal() usage in katana-ui views that should
# use AlignCenter instead for proper vertical centering.
#
# Allowed exceptions (annotated with #[allow(horizontal_layout)]):
#   Lines containing this annotation are skipped.
# ============================================================
set -euo pipefail

SEARCH_DIR="crates/katana-ui/src/views"
PATTERN='ui\.horizontal\s*\('
ALLOW_ANNOTATION='allow(horizontal_layout)'

errors=0

while IFS= read -r file; do
    line_num=0
    while IFS= read -r line; do
        line_num=$((line_num + 1))

        # Skip lines with the allow annotation
        if echo "$line" | grep -q "$ALLOW_ANNOTATION"; then
            continue
        fi

        if echo "$line" | grep -qE "$PATTERN"; then
            echo "error: Use AlignCenter instead of ui.horizontal() for vertical centering"
            echo "  --> ${file}:${line_num}"
            echo "  | ${line}"
            echo "  = help: Replace with crate::widgets::AlignCenter::new().left(...).show(ui)"
            echo "  = note: Add #[allow(horizontal_layout)] comment above this line to suppress"
            echo ""
            errors=$((errors + 1))
        fi
    done < "$file"
done < <(find "$SEARCH_DIR" -name '*.rs' -type f)

if [ $errors -gt 0 ]; then
    echo "Found $errors ui.horizontal() usage(s) that should use AlignCenter."
    echo "See: crates/katana-ui/src/widgets/align_center/"
    exit 1
fi

echo "✅ No ui.horizontal() violations found."
