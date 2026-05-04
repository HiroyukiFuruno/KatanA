//! `MarkdownSyntaxHighlighter` — syntect-backed implementation of
//! `katana_core::editor::SyntaxHighlighter` for use in the egui editor panel.

use egui_commonmark::syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};
use katana_core::editor::{SyntaxHighlighter, TokenSpan, TokenType};

/// Syntax highlighter that uses the bundled syntect `SyntaxSet` / `ThemeSet`.
///
/// This wraps syntect independently of `CommonMarkCache` so that it can be
/// composed into any `EditorWidget` implementation without egui-specific
/// dependencies.
pub struct MarkdownSyntaxHighlighter {
    ss: SyntaxSet,
    ts: ThemeSet,
}

impl MarkdownSyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            ss: SyntaxSet::load_defaults_newlines(),
            ts: ThemeSet::load_defaults(),
        }
    }
}

impl Default for MarkdownSyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxHighlighter for MarkdownSyntaxHighlighter {
    fn highlight(&self, source: &str, language: &str) -> Vec<TokenSpan> {
        let syntax = self
            .ss
            .find_syntax_by_extension(language)
            .or_else(|| self.ss.find_syntax_by_extension("md"))
            .unwrap_or_else(|| self.ss.find_syntax_plain_text());

        let Some(theme) = self.ts.themes.values().next() else {
            return vec![];
        };

        let mut h = HighlightLines::new(syntax, theme);
        let mut spans = Vec::new();
        let mut offset = 0usize;

        for line in LinesWithEndings::from(source) {
            match h.highlight_line(line, &self.ss) {
                Ok(ranges) => append_ranges(&ranges, &mut spans, &mut offset),
                Err(_) => offset += line.len(),
            }
        }

        spans
    }
}

fn style_to_token_type(style: &egui_commonmark::syntect::highlighting::Style) -> TokenType {
    if style.font_style.contains(FontStyle::BOLD) {
        TokenType::Heading
    } else if style.font_style.contains(FontStyle::ITALIC) {
        TokenType::Comment
    } else {
        TokenType::Default
    }
}

fn append_ranges(
    ranges: &[(egui_commonmark::syntect::highlighting::Style, &str)],
    spans: &mut Vec<TokenSpan>,
    offset: &mut usize,
) {
    for (style, text) in ranges {
        spans.push(TokenSpan {
            start: *offset,
            end: *offset + text.len(),
            token_type: style_to_token_type(style),
        });
        *offset += text.len();
    }
}
