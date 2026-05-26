//! Neutral editor interface.
//!
//! `katana-ui` supplies `MarkdownSyntaxHighlighter` (syntect-based);
//! at kle intake the Floem-based editor will implement `EditorWidget` instead.
use std::ops::Range;

/// Coarse token kind for syntax highlighting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Keyword,
    String,
    Comment,
    Number,
    Operator,
    Heading,
    Code,
    Default,
}

/// A highlighted token span within the source text (byte offsets).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightedSpan {
    pub range: Range<usize>,
    pub token_kind: TokenKind,
}

/// A collection of highlighted text spans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightedText {
    pub spans: Vec<HighlightedSpan>,
}

/// Language-agnostic syntax highlighter.
///
/// `katana-ui` implements this with `MarkdownSyntaxHighlighter` (syntect).
/// kle may supply its own implementation.
pub trait SyntaxHighlighter: Send + Sync {
    fn highlight(&self, source: &str) -> HighlightedText;
}

/// Shared editor configuration.
const DEFAULT_EDITOR_FONT_SIZE: f32 = 14.0;
const DEFAULT_EDITOR_THEME_IS_DARK: bool = false;

pub struct EditorConfig {
    pub syntax_highlighter: Box<dyn SyntaxHighlighter>,
    pub font_size: f32,
    pub theme_is_dark: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            syntax_highlighter: Box::new(NoopSyntaxHighlighter),
            font_size: DEFAULT_EDITOR_FONT_SIZE,
            theme_is_dark: DEFAULT_EDITOR_THEME_IS_DARK,
        }
    }
}

/// Internal no-op highlighter used by default so `EditorWidget` can be
/// constructed without a mandatory syntax implementation.
#[derive(Debug)]
struct NoopSyntaxHighlighter;

impl SyntaxHighlighter for NoopSyntaxHighlighter {
    fn highlight(&self, _source: &str) -> HighlightedText {
        HighlightedText { spans: Vec::new() }
    }
}

/// Egui-agnostic editor widget interface.
///
/// kle provides a Floem-based implementation of this trait.
/// The current egui-based editor in `katana-ui` provides trait-compatible
/// adapter shims until Floem migration is complete.
pub trait EditorWidget: Send + Sync {
    fn config(&self) -> &EditorConfig;
    fn apply_config(&mut self, config: EditorConfig);
    fn set_config(&mut self, config: EditorConfig) {
        self.apply_config(config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_config_default_is_safe_for_widget_consumers() {
        let config = EditorConfig::default();
        assert!(config.font_size > 0.0);
        assert!(!config.theme_is_dark);
    }

    #[test]
    fn noop_highlighter_defaults_to_empty_spans() {
        let highlighter = NoopSyntaxHighlighter;
        let highlighted = highlighter.highlight("");
        assert!(highlighted.spans.is_empty());
    }

    #[test]
    fn editor_widget_set_config_delegates_to_apply_config() {
        #[derive(Default)]
        struct DummyWidget {
            config: Option<EditorConfig>,
        }

        impl EditorWidget for DummyWidget {
            fn config(&self) -> &EditorConfig {
                self.config
                    .as_ref()
                    .expect("config should have been initialized")
            }

            fn apply_config(&mut self, config: EditorConfig) {
                self.config = Some(config);
            }
        }

        let mut widget = DummyWidget::default();
        let config = EditorConfig::default();
        widget.set_config(config);
        assert_eq!(widget.config().font_size, DEFAULT_EDITOR_FONT_SIZE);
    }
}
