# ============================================================
# Documentation / Analysis
# ============================================================

# Generate API documentation
doc:
    {{CARGO}} doc --workspace --no-deps

# Generate & open API documentation in browser
doc-open:
    {{CARGO}} doc --workspace --no-deps --open

# Binary size analysis (requires cargo-bloat)
bloat:
    {{CARGO}} bloat --release --bin KatanA

# Count lines of code (requires tokei)
loc:
    {{RTK_CMD}}tokei crates/

# Display dependency tree
tree:
    {{CARGO}} tree --workspace
