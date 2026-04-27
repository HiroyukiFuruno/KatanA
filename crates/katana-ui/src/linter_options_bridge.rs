use std::path::Path;

use katana_markdown_linter::{LintOptions, RuleConfig, rules::markdown::DocumentContext};
use katana_platform::settings::types::{LinterSettings, RuleSeverity};

pub(crate) struct MarkdownLinterOptionsBridgeOps;

impl MarkdownLinterOptionsBridgeOps {
    const MD013_RULE_ID: &'static str = "MD013";

    pub(crate) fn load_effective_options(
        state: &crate::app_state::AppState,
        path: &Path,
    ) -> LintOptions {
        let mut options =
            crate::linter_config_bridge::MarkdownLinterConfigOps::load_options_for_path(
                state, path,
            );
        Self::apply_katana_settings(&mut options, &state.config.settings.settings().linter);
        options
    }

    pub(crate) fn disable_unsafe_multibyte_md013(options: &mut LintOptions, content: &str) {
        if !Self::md013_has_unsafe_boundary(options, content) {
            return;
        }
        if let Some(rule_config) = options.rules.get_mut(Self::MD013_RULE_ID) {
            rule_config.enabled = false;
        }
    }

    fn apply_katana_settings(options: &mut LintOptions, settings: &LinterSettings) {
        for (rule_id, severity) in &settings.rule_severity {
            if matches!(severity, RuleSeverity::Ignore) {
                let rule_config = options.rules.entry(rule_id.clone()).or_default();
                rule_config.enabled = false;
            }
        }
    }

    fn md013_has_unsafe_boundary(options: &LintOptions, content: &str) -> bool {
        let Some(rule_config) = options.rules.get(Self::MD013_RULE_ID) else {
            return false;
        };
        if !rule_config.enabled {
            return false;
        }
        let ctx = DocumentContext::new(Path::new("<memory>"), content);
        let md013_options = Md013Options::from_rule_config(rule_config);
        ctx.lines().iter().enumerate().any(|(line_index, line)| {
            md013_options
                .limit_for(&ctx, line_index)
                .is_some_and(|limit| line.text.len() > limit && !line.text.is_char_boundary(limit))
        })
    }
}

struct Md013Options {
    line_length: usize,
    code_block_line_length: usize,
    heading_line_length: usize,
    code_blocks: bool,
    headings: bool,
    tables: bool,
}

impl Md013Options {
    const DEFAULT_LIMIT: usize = 80;

    fn from_rule_config(rule_config: &RuleConfig) -> Self {
        Self {
            line_length: Self::usize_property(rule_config, "line_length"),
            code_block_line_length: Self::usize_property(rule_config, "code_block_line_length"),
            heading_line_length: Self::usize_property(rule_config, "heading_line_length"),
            code_blocks: Self::bool_property(rule_config, "code_blocks", true),
            headings: Self::bool_property(rule_config, "headings", true),
            tables: Self::bool_property(rule_config, "tables", true),
        }
    }

    fn limit_for(&self, ctx: &DocumentContext<'_>, line_index: usize) -> Option<usize> {
        if ctx.is_code_line(line_index) {
            return self.code_blocks.then_some(self.code_block_line_length);
        }
        if ctx
            .headings()
            .iter()
            .any(|heading| heading.line == line_index)
        {
            return self.headings.then_some(self.heading_line_length);
        }
        if ctx
            .tables()
            .iter()
            .any(|table| (table.start_line..=table.end_line).contains(&line_index))
        {
            return self.tables.then_some(self.line_length);
        }
        Some(self.line_length)
    }

    fn usize_property(rule_config: &RuleConfig, key: &str) -> usize {
        rule_config
            .properties
            .get(key)
            .and_then(|value| value.parse().ok())
            .unwrap_or(Self::DEFAULT_LIMIT)
    }

    fn bool_property(rule_config: &RuleConfig, key: &str, default: bool) -> bool {
        rule_config
            .properties
            .get(key)
            .and_then(|value| value.parse().ok())
            .unwrap_or(default)
    }
}
