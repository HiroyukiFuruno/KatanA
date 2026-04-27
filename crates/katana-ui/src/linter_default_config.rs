pub(crate) struct MarkdownLinterDefaultConfigOps;

impl MarkdownLinterDefaultConfigOps {
    const KATANA_CONFIG: &'static str = include_str!("../../../.markdownlint.json");

    pub(crate) fn load() -> katana_markdown_linter::MarkdownLintConfig {
        let raw = match serde_json::from_str(Self::KATANA_CONFIG) {
            Ok(raw) => raw,
            Err(err) => panic!("bundled KatanA markdownlint config must be valid JSON: {err}"),
        };
        let config = katana_markdown_linter::MarkdownLintConfig { raw };
        let errors = config.validate_cached_rules();
        if !errors.is_empty() {
            panic!("bundled KatanA markdownlint config must be valid: {errors:?}");
        }
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn katana_default_config_loads_repo_markdownlint_json() {
        let config = MarkdownLinterDefaultConfigOps::load();

        assert_eq!(config.raw["MD013"], false);
        assert_eq!(config.raw["MD048"]["style"], "consistent");
        assert!(
            config.raw["MD052"]["ignored_labels"]
                .as_array()
                .expect("ignored_labels must be an array")
                .iter()
                .any(|it| it == "?")
        );
    }
}
