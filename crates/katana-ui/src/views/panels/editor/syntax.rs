//! `MarkdownSyntaxHighlighter` — syntect-backed implementation of
//! `katana_core::editor::SyntaxHighlighter` for use in the egui editor panel.

use egui_commonmark::syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};
use katana_core::editor::{HighlightedSpan, HighlightedText, SyntaxHighlighter, TokenKind};

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
    fn highlight(&self, source: &str) -> HighlightedText {
        let syntax = self
            .ss
            .find_syntax_by_extension("md")
            .unwrap_or_else(|| self.ss.find_syntax_plain_text());

        let Some(theme) = self.ts.themes.values().next() else {
            return HighlightedText { spans: Vec::new() };
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

        HighlightedText { spans }
    }
}

fn style_to_token_kind(style: &egui_commonmark::syntect::highlighting::Style) -> TokenKind {
    if style.font_style.contains(FontStyle::BOLD) {
        TokenKind::Heading
    } else if style.font_style.contains(FontStyle::ITALIC) {
        TokenKind::Comment
    } else {
        TokenKind::Default
    }
}

fn append_ranges(
    ranges: &[(egui_commonmark::syntect::highlighting::Style, &str)],
    spans: &mut Vec<HighlightedSpan>,
    offset: &mut usize,
) {
    for (style, text) in ranges {
        spans.push(HighlightedSpan {
            range: (*offset)..(*offset + text.len()),
            token_kind: style_to_token_kind(style),
        });
        *offset += text.len();
    }
}
