use crate::markdown::MarkdownError;

const BODY_FONT_SIZE: u32 = 16;
const CODE_FONT_SIZE: u32 = 14;
const LINE_SPACING: u32 = 10;
const TEXT_COLUMNS: usize = 82;

/* Marker characters used to survive the tag-stripping pipeline */
const HEADING_START: char = '\x01';
const HEADING_SEP: char = '\x02';
const HEADING_END: char = '\x03';
const CODE_FENCE: char = '\x04';

#[derive(Clone)]
pub(super) struct NativeTextLine {
    pub(super) text: String,
    pub(super) font_size: u32,
    pub(super) bold: bool,
    pub(super) is_code: bool,
    pub(super) spans: Vec<NativeTextSpan>,
}

#[derive(Clone)]
pub(super) struct NativeTextSpan {
    pub(super) text: String,
    pub(super) color: [u8; 3],
}

impl NativeTextLine {
    pub(super) fn body(text: String) -> Self {
        Self {
            text,
            font_size: BODY_FONT_SIZE,
            bold: false,
            is_code: false,
            spans: vec![],
        }
    }

    fn heading(text: String, level: u8) -> Self {
        let font_size = match level {
            1 => 28,
            2 => 22,
            3 => 18,
            _ => 16,
        };
        Self {
            text,
            font_size,
            bold: true,
            is_code: false,
            spans: vec![],
        }
    }

    fn code_plain(text: String) -> Self {
        Self {
            text,
            font_size: CODE_FONT_SIZE,
            bold: false,
            is_code: true,
            spans: vec![],
        }
    }

    fn code_highlighted(text: String, spans: Vec<NativeTextSpan>) -> Self {
        Self {
            text,
            font_size: CODE_FONT_SIZE,
            bold: false,
            is_code: true,
            spans,
        }
    }

    pub(super) fn line_height(&self) -> u32 {
        self.font_size + LINE_SPACING
    }

    pub(super) fn is_heading(&self) -> bool {
        self.bold && self.font_size > BODY_FONT_SIZE
    }
}

pub(super) fn extract_lines(
    html: &str,
    is_dark: bool,
) -> Result<Vec<NativeTextLine>, MarkdownError> {
    let body = body_content(html)?;
    let (body_no_code, code_blocks) = extract_code_blocks(&body, is_dark)?;
    let with_heading_marks = mark_headings(&body_no_code)?;
    let with_image_alt = replace_image_alt(&with_heading_marks)?;
    let without_scripts = remove_tag_blocks(&with_image_alt, "script")?;
    let without_style = remove_tag_blocks(&without_scripts, "style")?;
    let with_breaks = block_tags_to_breaks(&without_style)?;
    let without_tags = strip_tags(&with_breaks)?;
    let decoded = decode_entities(&without_tags);
    /* WHY: decode_entities() re-introduces < and > from HTML entities in code blocks.
    A second strip removes these decoded tag patterns. */
    let clean = strip_tags(&decoded)?;
    parse_typed_lines(&clean, &code_blocks)
}

fn body_content(html: &str) -> Result<String, MarkdownError> {
    let lower = html.to_ascii_lowercase();
    let start = lower
        .find("<body")
        .and_then(|pos| lower[pos..].find('>').map(|end| pos + end + 1))
        .unwrap_or(0);
    let end = lower.rfind("</body>").unwrap_or(html.len());
    Ok(html[start..end.max(start)].to_string())
}

/* Extract <pre><code> blocks before tag stripping, replace with CODE_FENCE markers */
fn extract_code_blocks(
    html: &str,
    is_dark: bool,
) -> Result<(String, Vec<Vec<NativeTextLine>>), MarkdownError> {
    let regex = regex::Regex::new(r#"(?is)<pre\b[^>]*><code\b([^>]*)>(.*?)</code></pre>"#)
        .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
    let mut code_blocks: Vec<Vec<NativeTextLine>> = Vec::new();
    let mut result = String::new();
    let mut last_end = 0;

    for caps in regex.captures_iter(html) {
        let full = caps.get(0).unwrap();
        result.push_str(&html[last_end..full.start()]);

        let attrs = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let raw_code = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let code = decode_entities(raw_code);
        let language = extract_code_language(attrs);
        let lines = highlight_code(&code, language.as_deref(), is_dark);

        let idx = code_blocks.len();
        result.push_str(&format!("\n{CODE_FENCE}{idx}{CODE_FENCE}\n"));
        code_blocks.push(lines);
        last_end = full.end();
    }
    result.push_str(&html[last_end..]);
    Ok((result, code_blocks))
}

fn extract_code_language(attrs: &str) -> Option<String> {
    let regex = regex::Regex::new(r#"(?i)class="language-([^"\s]+)""#).ok()?;
    regex.captures(attrs).map(|c| c[1].to_lowercase())
}

fn highlight_code(code: &str, language: Option<&str>, is_dark: bool) -> Vec<NativeTextLine> {
    use std::sync::LazyLock;
    use syntect::easy::HighlightLines;
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;
    use syntect::util::LinesWithEndings;

    static PS: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
    static TS: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

    let theme_name = if is_dark {
        "base16-ocean.dark"
    } else {
        "InspiredGitHub"
    };
    let Some(theme) = TS.themes.get(theme_name) else {
        return code
            .lines()
            .map(|l| NativeTextLine::code_plain(l.to_string()))
            .collect();
    };

    let syntax = language
        .and_then(|lang| PS.find_syntax_by_token(lang))
        .unwrap_or_else(|| PS.find_syntax_plain_text());

    let mut h = HighlightLines::new(syntax, theme);
    let mut lines = Vec::new();

    for line_str in LinesWithEndings::from(code) {
        let text = line_str.trim_end_matches(['\n', '\r']).to_string();
        match h.highlight_line(line_str, &PS) {
            Ok(ranges) => {
                let spans: Vec<NativeTextSpan> = ranges
                    .iter()
                    .filter(|(_, t)| !t.is_empty() && *t != "\n" && *t != "\r\n")
                    .map(|(style, t)| NativeTextSpan {
                        text: t.trim_end_matches(['\n', '\r']).to_string(),
                        color: [style.foreground.r, style.foreground.g, style.foreground.b],
                    })
                    .filter(|s| !s.text.is_empty())
                    .collect();
                lines.push(NativeTextLine::code_highlighted(text, spans));
            }
            Err(_) => lines.push(NativeTextLine::code_plain(text)),
        }
    }
    lines
}

/* Mark <h1>…</h6> with control-char markers that survive tag stripping.
regex crate does not support backreferences, so iterate per heading level. */
fn mark_headings(html: &str) -> Result<String, MarkdownError> {
    let mut result = html.to_string();
    for level in 1u8..=6 {
        let pattern = format!(r"(?is)<h{level}\b[^>]*>(.*?)</h{level}>");
        let regex =
            regex::Regex::new(&pattern).map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        result = regex
            .replace_all(&result, |caps: &regex::Captures| {
                let content = &caps[1];
                format!("\n{HEADING_START}h{level}{HEADING_SEP}{content}{HEADING_END}\n")
            })
            .to_string();
    }
    Ok(result)
}

fn replace_image_alt(html: &str) -> Result<String, MarkdownError> {
    let regex = regex::Regex::new(r#"(?is)<img\b[^>]*\balt="([^"]*)"[^>]*>"#)
        .map_err(|error| MarkdownError::ExportFailed(error.to_string()))?;
    Ok(regex
        .replace_all(html, |captures: &regex::Captures| {
            format!("\n[image: {}]\n", decode_entities(&captures[1]))
        })
        .to_string())
}

fn remove_tag_blocks(html: &str, tag: &str) -> Result<String, MarkdownError> {
    let pattern = format!(r"(?is)<{tag}\b[^>]*>.*?</{tag}>");
    let regex = regex::Regex::new(&pattern)
        .map_err(|error| MarkdownError::ExportFailed(error.to_string()))?;
    Ok(regex.replace_all(html, "\n").to_string())
}

fn block_tags_to_breaks(html: &str) -> Result<String, MarkdownError> {
    let regex = regex::Regex::new(
        r"(?is)</?(p|div|section|article|header|footer|li|tr|table|pre|blockquote|br|hr)\b[^>]*>",
    )
    .map_err(|error| MarkdownError::ExportFailed(error.to_string()))?;
    Ok(regex.replace_all(html, "\n").to_string())
}

fn strip_tags(html: &str) -> Result<String, MarkdownError> {
    let regex = regex::Regex::new(r"(?is)<[^>]+>")
        .map_err(|error| MarkdownError::ExportFailed(error.to_string()))?;
    Ok(regex.replace_all(html, " ").to_string())
}

fn decode_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn parse_typed_lines(
    text: &str,
    code_blocks: &[Vec<NativeTextLine>],
) -> Result<Vec<NativeTextLine>, MarkdownError> {
    let mut result = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        /* Code block placeholder: \x04N\x04 */
        if line.starts_with(CODE_FENCE) && line.ends_with(CODE_FENCE) && line.len() > 2 {
            let inner = &line[CODE_FENCE.len_utf8()..line.len() - CODE_FENCE.len_utf8()];
            if let Ok(idx) = inner.parse::<usize>()
                && let Some(code_lines) = code_blocks.get(idx)
            {
                result.extend(code_lines.iter().cloned());
                continue;
            }
        }

        /* Heading marker: \x01hN\x02content\x03 */
        if line.starts_with(HEADING_START) {
            let rest = &line[HEADING_START.len_utf8()..];
            if let Some(sep_pos) = rest.find(HEADING_SEP) {
                let level_str = &rest[..sep_pos];
                let after_sep = &rest[sep_pos + HEADING_SEP.len_utf8()..];
                let content_raw = after_sep.trim_end_matches(HEADING_END).trim();
                let level: u8 = match level_str {
                    "h1" => 1,
                    "h2" => 2,
                    "h3" => 3,
                    "h4" => 4,
                    "h5" => 5,
                    "h6" => 6,
                    _ => 0,
                };
                if level > 0 {
                    /* heading inner HTML may still have tags – strip them */
                    let clean = strip_tags(content_raw)?;
                    let text = decode_entities(&clean);
                    let cols = heading_columns(level);
                    for wrapped in wrap_line_n(&text, cols) {
                        result.push(NativeTextLine::heading(wrapped, level));
                    }
                    continue;
                }
            }
        }

        /* Normal body text */
        let normalized = line.split_whitespace().collect::<Vec<_>>().join(" ");
        for wrapped in wrap_line_n(&normalized, TEXT_COLUMNS) {
            result.push(NativeTextLine::body(wrapped));
        }
    }

    Ok(result)
}

fn heading_columns(level: u8) -> usize {
    match level {
        1 => 47,
        2 => 60,
        3 => 73,
        _ => TEXT_COLUMNS,
    }
}

fn wrap_line_n(line: &str, columns: usize) -> Vec<String> {
    let mut rows = Vec::new();
    let mut current = String::new();
    for word in line.split_whitespace() {
        if current.chars().count() + word.chars().count() + 1 > columns && !current.is_empty() {
            rows.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        rows.push(current);
    }
    rows
}

pub(super) fn is_dark_background(color: &str) -> bool {
    parse_hex_rgb(color)
        .map(|[r, g, b]| {
            /* perceptual luminance */
            0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32) < 128.0
        })
        .unwrap_or(false)
}

fn parse_hex_rgb(color: &str) -> Option<[u8; 3]> {
    let hex = color.trim().strip_prefix('#')?;
    let n = u32::from_str_radix(hex, 16).ok()?;
    match hex.len() {
        6 => Some([(n >> 16) as u8, (n >> 8) as u8, n as u8]),
        3 => {
            let r = ((n >> 8) & 0xf) as u8;
            let g = ((n >> 4) & 0xf) as u8;
            let b = (n & 0xf) as u8;
            Some([r << 4 | r, g << 4 | g, b << 4 | b])
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_content_extracts_from_full_html() {
        let html = "<!DOCTYPE html><html><head><title>Doc</title></head><body><h1>Hello</h1></body></html>";
        assert_eq!(body_content(html).unwrap(), "<h1>Hello</h1>");
    }

    #[test]
    fn body_content_strips_head_when_no_closing_body() {
        let html = "<!DOCTYPE html><html><head><title>Exported Document</title></head><body><h1>Hello</h1>";
        let result = body_content(html).unwrap();
        assert!(
            !result.contains("Exported Document"),
            "head content must not leak"
        );
        assert!(result.contains("<h1>Hello</h1>"));
    }

    #[test]
    fn body_content_strips_tail_when_no_opening_body() {
        let html = "<p>More text</p></body></html>";
        assert_eq!(body_content(html).unwrap(), "<p>More text</p>");
    }

    #[test]
    fn body_content_returns_as_is_when_no_body_tags() {
        let html = "<p>Middle content</p>";
        assert_eq!(body_content(html).unwrap(), "<p>Middle content</p>");
    }

    #[test]
    fn extract_lines_does_not_include_html_title() {
        let html = r#"<!DOCTYPE html><html><head><title>Exported Document</title><style>body{color:red}</style></head><body><h1>Title</h1><p>Text</p></body></html>"#;
        let lines = extract_lines(html, false).unwrap();
        assert!(!lines.iter().any(|l| l.text.contains("Exported Document")));
        assert!(lines.iter().any(|l| l.text.contains("Title")));
        assert!(lines.iter().any(|l| l.text.contains("Text")));
    }

    #[test]
    fn extract_lines_headings_get_larger_font_sizes() {
        let html = r#"<body><h1>Big Title</h1><h2>Sub</h2><p>body text</p></body>"#;
        let lines = extract_lines(html, false).unwrap();
        let h1 = lines.iter().find(|l| l.text.contains("Big Title")).unwrap();
        let h2 = lines.iter().find(|l| l.text.contains("Sub")).unwrap();
        let body = lines.iter().find(|l| l.text.contains("body text")).unwrap();
        assert_eq!(h1.font_size, 28);
        assert!(h1.bold);
        assert_eq!(h2.font_size, 22);
        assert!(h2.bold);
        assert_eq!(body.font_size, 16);
        assert!(!body.bold);
    }

    #[test]
    fn extract_lines_code_block_html_does_not_leak_tags() {
        let html = r#"<!DOCTYPE html><html><head><style>body{color:red}</style></head><body>
<h1>Title</h1>
<p>See this HTML:</p>
<pre><code class="language-html">&lt;h1&gt;Hello&lt;/h1&gt;
&lt;p&gt;World&lt;/p&gt;
</code></pre>
<p>End.</p>
</body></html>"#;
        let lines = extract_lines(html, true).unwrap();
        /* HTML tags are valid source code inside code blocks. The important guarantee is
        that they do NOT leak into body (non-code) lines as structural HTML. */
        let body_joined = lines
            .iter()
            .filter(|l| !l.is_code)
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !body_joined.contains("<h1>") && !body_joined.contains("</h1>"),
            "HTML tags from code blocks must not leak into body text: {:?}",
            body_joined
        );
        assert!(lines.iter().any(|l| l.text.contains("Title")));
        assert!(
            lines
                .iter()
                .any(|l| l.text.contains("Hello")
                    || l.spans.iter().any(|s| s.text.contains("Hello")))
        );
        assert!(lines.iter().any(|l| l.text.contains("End")));
    }

    #[test]
    fn code_block_lines_are_marked_is_code() {
        let html = r#"<body><pre><code class="language-rust">fn main() {}</code></pre></body>"#;
        let lines = extract_lines(html, true).unwrap();
        assert!(
            lines.iter().any(|l| l.is_code),
            "code block lines must have is_code=true"
        );
    }
}
