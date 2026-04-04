use crate::Violation;
use std::path::Path;

pub mod discovery;
pub mod structure;

pub struct MarkdownOps;

impl MarkdownOps {
    pub fn lint(root: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();
        for pair in discovery::MarkdownDiscoveryOps::collect_markdown_pairs(root) {
            violations
                .extend(structure::MarkdownStructureOps::compare_markdown_heading_structure(&pair));
        }
        violations
    }
}
