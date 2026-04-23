use crate::rules::markdown::BrokenLinkRule;
use crate::rules::markdown::HeadingStructureRule;

use crate::rules::markdown::rules::blockquote::*;
use crate::rules::markdown::rules::content::*;
use crate::rules::markdown::rules::content_ext::*;
use crate::rules::markdown::rules::heading::*;
use crate::rules::markdown::rules::heading_ext::*;
use crate::rules::markdown::rules::heading_style::*;
use crate::rules::markdown::rules::image::*;
use crate::rules::markdown::rules::list::*;
use crate::rules::markdown::rules::list_ext::*;
use crate::rules::markdown::rules::style::*;
use crate::rules::markdown::rules::table::*;
use crate::rules::markdown::rules::whitespace::*;
use crate::rules::markdown::stubs_regex::*;
use crate::rules::markdown::{MarkdownDiagnostic, MarkdownRule};

pub struct MarkdownLinterOps;

impl MarkdownLinterOps {
    pub fn evaluate_all(
        file_path: &std::path::Path,
        content: &str,
        enabled: bool,
        severity_map: &std::collections::HashMap<
            String,
            Option<crate::rules::markdown::DiagnosticSeverity>,
        >,
    ) -> Vec<MarkdownDiagnostic> {
        let mut diagnostics = Vec::new();

        let rules = Self::get_official_rules();

        if !enabled {
            return diagnostics;
        }

        for rule in rules {
            let rule_id = rule.id();
            let sev_opt = severity_map
                .get(rule_id)
                .copied()
                .unwrap_or(Some(crate::rules::markdown::DiagnosticSeverity::Warning));
            if let Some(severity) = sev_opt {
                let mut diags = rule.evaluate(file_path, content);
                for d in &mut diags {
                    d.severity = severity;
                }
                diagnostics.extend(diags);
            }
        }

        diagnostics
    }

    pub fn get_official_rules() -> Vec<Box<dyn MarkdownRule>> {
        vec![
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
            /* WHY: Blockquote rules */
            Box::new(NoBlanksBlockquoteRule),    // MD028
            Box::new(SingleTrailingNewlineRule), // MD047
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
        ]
    }

    /* WHY: User-configurable rules are those with official_meta (i.e., not internal-only).
     * This includes both actively evaluated rules AND stub rules (official rules that are not
     * yet fully implemented). All are shown in the settings UI so the user can configure
     * severity for when they become active. Rules are deduplicated by ID and sorted. */
    pub fn get_user_configurable_rules() -> Vec<Box<dyn MarkdownRule>> {
        use crate::rules::markdown::stubs::*;
        let mut all: Vec<Box<dyn MarkdownRule>> = Self::get_official_rules()
            .into_iter()
            .filter(|r| r.official_meta().is_some())
            .collect();

        /* WHY: Add stub rules that represent official markdownlint rules not yet implemented.
         * These are shown in the settings UI for forward-compatibility with .markdownlint.json. */
        let stubs: Vec<Box<dyn MarkdownRule>> = vec![
            Box::new(RuleMD001),
            Box::new(RuleMD003),
            Box::new(RuleMD004),
            Box::new(RuleMD005),
            Box::new(RuleMD007),
            Box::new(RuleMD011),
            Box::new(RuleMD012),
            Box::new(RuleMD013),
            Box::new(RuleMD014),
            Box::new(RuleMD020),
            Box::new(RuleMD021),
            Box::new(RuleMD022),
            Box::new(RuleMD023),
            Box::new(RuleMD024),
            Box::new(RuleMD025),
            Box::new(RuleMD026),
            Box::new(RuleMD027),
            Box::new(RuleMD028),
            Box::new(RuleMD029),
            Box::new(RuleMD030),
            Box::new(RuleMD031),
            Box::new(RuleMD032),
            Box::new(RuleMD033),
            Box::new(RuleMD034),
            Box::new(RuleMD035),
            Box::new(RuleMD036),
            Box::new(RuleMD040),
            Box::new(RuleMD041),
            Box::new(RuleMD042),
            Box::new(RuleMD043),
            Box::new(RuleMD044),
            Box::new(RuleMD045),
            Box::new(RuleMD046),
            Box::new(RuleMD047),
            Box::new(RuleMD048),
            Box::new(RuleMD049),
            Box::new(RuleMD050),
            Box::new(RuleMD051),
            Box::new(RuleMD052),
            Box::new(RuleMD053),
            Box::new(RuleMD054),
            Box::new(RuleMD055),
            Box::new(RuleMD056),
            Box::new(RuleMD058),
            Box::new(RuleMD059),
            Box::new(TableColumnStyleRule),
        ];

        /* WHY: Collect existing IDs first to avoid duplicating rules already in get_official_rules(). */
        let existing_ids: std::collections::HashSet<&str> = all.iter().map(|r| r.id()).collect();
        for stub in stubs {
            if !existing_ids.contains(stub.id()) {
                all.push(stub);
            }
        }

        all.sort_by_key(|r| r.id());
        all
    }
}
