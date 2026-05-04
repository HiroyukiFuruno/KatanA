//! Neutral editor interface.
//!
//! `katana-ui` supplies `MarkdownSyntaxHighlighter` (syntect-based); at kle
//! intake the Floem-based editor will implement `EditorWidget` instead.

/// Coarse token type for syntax highlighting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    Keyword,
    String,
    Comment,
    Code,
    Heading,
    Default,
}

/// A highlighted token span within the source text (byte offsets).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenSpan {
    pub start: usize,
    pub end: usize,
    pub token_type: TokenType,
}

/// Language-aware syntax highlighter.
///
/// `katana-ui` implements this with `MarkdownSyntaxHighlighter` (syntect).
/// kle may supply its own implementation.
pub trait SyntaxHighlighter: Send + Sync {
    fn highlight(&self, source: &str, language: &str) -> Vec<TokenSpan>;
}

const DEFAULT_TAB_SIZE: u32 = 4;

/// Editor configuration shared between `EditorWidget` and `SyntaxHighlighter`.
#[derive(Debug, Clone)]
pub struct EditorConfig {
    /// File extension used to select the syntax definition (e.g. `"md"`).
    pub language: String,
    pub tab_size: u32,
    pub soft_wrap: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            language: "md".to_string(),
            tab_size: DEFAULT_TAB_SIZE,
            soft_wrap: true,
        }
    }
}

/// Egui-agnostic editor widget interface.
///
/// kle provides a Floem-based implementation of this trait.
/// The current egui-based editor in `katana-ui` does not yet implement it
/// (Floem migration is planned for Phase 1 of the UI framework transition).
pub trait EditorWidget: Send + Sync {
    fn config(&self) -> &EditorConfig;
    fn set_config(&mut self, config: EditorConfig);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_config_default_has_markdown_language_and_standard_settings() {
        let config = EditorConfig::default();
        assert_eq!(config.language, "md");
        assert_eq!(config.tab_size, DEFAULT_TAB_SIZE);
        assert!(config.soft_wrap);
    }
}
