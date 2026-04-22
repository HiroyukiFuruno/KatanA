use crate::rules::markdown::BrokenLinkRule;
use crate::rules::markdown::HeadingStructureRule;

use crate::rules::markdown::stubs_regex::*;
use crate::rules::markdown::{MarkdownDiagnostic, MarkdownRule};

pub struct MarkdownLinterOps;

impl MarkdownLinterOps {
    pub fn evaluate_all(
        file_path: &std::path::Path,
        content: &str,
        disabled_rules: &std::collections::HashSet<String>,
    ) -> Vec<MarkdownDiagnostic> {
        let mut diagnostics = Vec::new();

        let rules: Vec<Box<dyn MarkdownRule>> = vec![
            Box::new(HeadingStructureRule),
            Box::new(BrokenLinkRule),
            Box::new(RuleMD009),
            Box::new(RuleMD010),
            Box::new(RuleMD018),
            Box::new(RuleMD019),
            Box::new(RuleMD037),
            Box::new(RuleMD038),
            Box::new(RuleMD039),
        ];

        for rule in rules {
            if !disabled_rules.contains(rule.id()) {
                diagnostics.extend(rule.evaluate(file_path, content));
            }
        }

        diagnostics
    }
}
