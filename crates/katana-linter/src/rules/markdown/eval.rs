use crate::rules::markdown::BrokenLinkRule;
use crate::rules::markdown::HeadingStructureRule;

use crate::rules::markdown::rules::blockquote::*;
use crate::rules::markdown::rules::content::*;
use crate::rules::markdown::rules::heading::*;
use crate::rules::markdown::rules::heading_ext::*;
use crate::rules::markdown::rules::list::*;
use crate::rules::markdown::rules::style::*;
use crate::rules::markdown::rules::whitespace::*;
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
            /* WHY: MD001 — heading-increment (full impl in mod.rs) */
            Box::new(HeadingStructureRule),
            /* WHY: Internal-only broken link rule (hidden from user) */
            Box::new(BrokenLinkRule),
            /* WHY: Heading rules */
            Box::new(HeadingStyleRule),          // MD003
            Box::new(BlanksAroundHeadingsRule),  // MD022
            Box::new(HeadingStartLeftRule),      // MD023
            Box::new(SingleH1Rule),              // MD025
            Box::new(NoTrailingPunctuationRule), // MD026
            /* WHY: Regex-based rules */
            Box::new(RuleMD009), // trailing-spaces
            Box::new(RuleMD010), // hard-tabs
            Box::new(RuleMD018), // no-missing-space-atx
            Box::new(RuleMD019), // no-multiple-space-atx
            Box::new(RuleMD037), // no-space-in-emphasis
            Box::new(RuleMD038), // no-space-in-code
            Box::new(RuleMD039), // no-space-in-links
            /* WHY: Whitespace rules */
            Box::new(NoMultipleBlanksRule),          // MD012
            Box::new(NoMultipleSpaceBlockquoteRule), // MD027
            Box::new(NoBlanksBlockquoteRule),        // MD028
            Box::new(SingleTrailingNewlineRule),     // MD047
            /* WHY: Content rules */
            Box::new(NoInlineHtmlRule),       // MD033
            Box::new(FencedCodeLanguageRule), // MD040
            Box::new(FirstLineHeadingRule),   // MD041
            Box::new(NoEmptyLinksRule),       // MD042
            /* WHY: List rules */
            Box::new(UlStyleRule),           // MD004
            Box::new(OlPrefixRule),          // MD029
            Box::new(BlanksAroundListsRule), // MD032
            /* WHY: Style rules */
            Box::new(HrStyleRule),             // MD035
            Box::new(NoEmphasisAsHeadingRule), // MD036
            Box::new(NoAltTextRule),           // MD045
        ];

        for rule in rules {
            if !disabled_rules.contains(rule.id()) {
                diagnostics.extend(rule.evaluate(file_path, content));
            }
        }

        diagnostics
    }
}
