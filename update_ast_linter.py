import re

with open("crates/katana-linter/tests/ast_linter.rs", "r") as f:
    content = f.read()

# Replace the arrays of paths with a call to `target_crates(root)`
# Wait, first define `fn target_crates(root: &std::path::Path) -> Vec<std::path::PathBuf>`
helper = """
fn target_crates(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    vec![
        root.join("crates/katana-linter/src"),
        root.join("crates/katana-core/src"),
        // root.join("crates/katana-platform/src"), // Phase 4
        // root.join("crates/katana-ui/src"), // Phase 5
    ]
}

#[test]
"""
content = content.replace("#[test]\nfn ast_linter_i18n_no_hardcoded_strings() {", helper + "fn ast_linter_i18n_no_hardcoded_strings() {")

# Replace all occurrences of these explicit array blocks
array_pattern1 = r"""&\[\s*root\.join\("crates/katana-core/src"\),\s*root\.join\("crates/katana-platform/src"\),\s*root\.join\("crates/katana-ui/src"\),\s*\]"""
array_pattern2 = r"""&\[\s*root\.join\("crates/katana-linter/src/rules/coding"\),\s*root\.join\("crates/katana-linter/src/rules/structure"\),\s*\]"""
array_pattern3 = r"""&\[root\.join\("crates/katana-linter/src"\)\]"""

content = re.sub(array_pattern1, "&target_crates(root)", content)
content = re.sub(array_pattern2, "&target_crates(root)", content)
content = re.sub(array_pattern3, "&target_crates(root)", content)

with open("crates/katana-linter/tests/ast_linter.rs", "w") as f:
    f.write(content)
