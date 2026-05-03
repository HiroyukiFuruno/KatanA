use super::native_text::{
    CODE_FENCE_MARKER, HEADING_END_MARKER, HEADING_LEVEL_1, HEADING_LEVEL_6, HEADING_SEP_MARKER,
    HEADING_START_MARKER, NativeTextLine, NativeTextSpan,
};
use crate::markdown::MarkdownError;

pub(super) fn body_content(html: &str) -> Result<String, MarkdownError> {
    let lower = html.to_ascii_lowercase();
    let start = lower
        .find("<body")
        .and_then(|pos| lower[pos..].find('>').map(|end| pos + end + 1))
        .unwrap_or(0);
    let end = lower.rfind("</body>").unwrap_or(html.len());
    Ok(html[start..end.max(start)].to_string())
}

/* Extract <pre><code> blocks before tag stripping, replace with CODE_FENCE markers */
pub(super) fn extract_code_blocks(
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
        result.push_str(&format!("\n{CODE_FENCE_MARKER}{idx}{CODE_FENCE_MARKER}\n"));
        code_blocks.push(lines);
        last_end = full.end();
    }
    result.push_str(&html[last_end..]);
    Ok((result, code_blocks))
}

fn extract_code_language(attrs: &str) -> Option<String> {
    let regex = regex::Regex::new(r#"(?i)class="language-([^"\s]+)""#).ok()?;
    regex
        .captures(attrs)
        .and_then(|c| c.get(1).map(|capture| capture.as_str().to_lowercase()))
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
pub(super) fn mark_headings(html: &str) -> Result<String, MarkdownError> {
    let mut result = html.to_string();
    for level in HEADING_LEVEL_1..=HEADING_LEVEL_6 {
        let pattern = format!(r"(?is)<h{level}\b[^>]*>(.*?)</h{level}>");
        let regex =
            regex::Regex::new(&pattern).map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        result = regex
            .replace_all(&result, |caps: &regex::Captures| {
                let content = caps.get(1).map(|capture| capture.as_str()).unwrap_or("");
                format!(
                    "\n{HEADING_START_MARKER}h{level}{HEADING_SEP_MARKER}{content}{HEADING_END_MARKER}\n",
                )
            })
            .to_string();
    }
    Ok(result)
}

pub(super) fn replace_image_alt(html: &str) -> Result<String, MarkdownError> {
    let regex = regex::Regex::new(r#"(?is)<img\b[^>]*\balt="([^"]*)"[^>]*>"#)
        .map_err(|error| MarkdownError::ExportFailed(error.to_string()))?;
    Ok(regex
        .replace_all(html, |captures: &regex::Captures| {
            let image_text = captures.get(1).map_or("", |capture| capture.as_str());
            format!("\n[image: {}]\n", decode_entities(image_text))
        })
        .to_string())
}

pub(super) fn remove_tag_blocks(html: &str, tag: &str) -> Result<String, MarkdownError> {
    let pattern = format!(r"(?is)<{tag}\b[^>]*>.*?</{tag}>");
    let regex = regex::Regex::new(&pattern)
        .map_err(|error| MarkdownError::ExportFailed(error.to_string()))?;
    Ok(regex.replace_all(html, "\n").to_string())
}

pub(super) fn block_tags_to_breaks(html: &str) -> Result<String, MarkdownError> {
    let regex = regex::Regex::new(
        r"(?is)</?(p|div|section|article|header|footer|li|tr|table|pre|blockquote|br|hr)\b[^>]*>",
    )
    .map_err(|error| MarkdownError::ExportFailed(error.to_string()))?;
    Ok(regex.replace_all(html, "\n").to_string())
}

pub(super) fn strip_tags(html: &str) -> Result<String, MarkdownError> {
    let regex = regex::Regex::new(r"(?is)<[^>]+>")
        .map_err(|error| MarkdownError::ExportFailed(error.to_string()))?;
    Ok(regex.replace_all(html, " ").to_string())
}

pub(super) fn decode_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}
