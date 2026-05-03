use super::native_text::{
    CODE_FENCE_MARKER, HEADING_COLUMN_SIZE_H1, HEADING_COLUMN_SIZE_H2, HEADING_COLUMN_SIZE_H3,
    HEADING_END_MARKER, HEADING_LEVEL_1, HEADING_LEVEL_2, HEADING_LEVEL_3, HEADING_LEVEL_4,
    HEADING_LEVEL_5, HEADING_LEVEL_6, HEADING_SEP_MARKER, HEADING_START_MARKER, NativeTextLine,
    TEXT_COLUMNS, WORD_SPACING,
};
use super::native_text_parser_html::{decode_entities, strip_tags};
use crate::markdown::MarkdownError;

pub(super) fn parse_typed_lines(
    text: &str,
    code_blocks: &[Vec<NativeTextLine>],
) -> Result<Vec<NativeTextLine>, MarkdownError> {
    let mut result = Vec::new();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(code_lines) = parse_code_fence(line, code_blocks) {
            result.extend(code_lines.iter().cloned());
            continue;
        }
        if let Some(wrapped) = parse_heading_line(line)? {
            result.extend(wrapped);
            continue;
        }

        append_wrapped_body_lines(line, &mut result);
    }
    Ok(result)
}

fn parse_code_fence<'a>(
    line: &'a str,
    code_blocks: &'a [Vec<NativeTextLine>],
) -> Option<&'a Vec<NativeTextLine>> {
    if !line.starts_with(CODE_FENCE_MARKER)
        || !line.ends_with(CODE_FENCE_MARKER)
        || line.len() <= CODE_FENCE_MARKER.len()
    {
        return None;
    }
    let inner = &line[CODE_FENCE_MARKER.len()..line.len() - CODE_FENCE_MARKER.len()];
    let Ok(idx) = inner.parse::<usize>() else {
        return None;
    };
    code_blocks.get(idx)
}

fn parse_heading_line(line: &str) -> Result<Option<Vec<NativeTextLine>>, MarkdownError> {
    if !line.starts_with(HEADING_START_MARKER) {
        return Ok(None);
    }
    let rest = &line[HEADING_START_MARKER.len()..];
    let Some(sep_pos) = rest.find(HEADING_SEP_MARKER) else {
        return Ok(None);
    };
    let level_str = &rest[..sep_pos];
    let after_sep = &rest[sep_pos + HEADING_SEP_MARKER.len()..];
    let content_raw = after_sep.trim_end_matches(HEADING_END_MARKER).trim();
    let Some(level) = heading_level(level_str) else {
        return Ok(None);
    };
    let clean = strip_tags(content_raw)?;
    let text = decode_entities(&clean);
    let mut rows = Vec::new();
    for wrapped in wrap_line_n(&text, heading_columns(level)) {
        rows.push(NativeTextLine::heading(wrapped, level));
    }
    Ok(Some(rows))
}

fn heading_level(level_str: &str) -> Option<u8> {
    match level_str {
        "h1" => Some(HEADING_LEVEL_1),
        "h2" => Some(HEADING_LEVEL_2),
        "h3" => Some(HEADING_LEVEL_3),
        "h4" => Some(HEADING_LEVEL_4),
        "h5" => Some(HEADING_LEVEL_5),
        "h6" => Some(HEADING_LEVEL_6),
        _ => None,
    }
}

fn heading_columns(level: u8) -> usize {
    match level {
        HEADING_LEVEL_1 => HEADING_COLUMN_SIZE_H1,
        HEADING_LEVEL_2 => HEADING_COLUMN_SIZE_H2,
        HEADING_LEVEL_3 => HEADING_COLUMN_SIZE_H3,
        _ => TEXT_COLUMNS,
    }
}

fn append_wrapped_body_lines(line: &str, result: &mut Vec<NativeTextLine>) {
    let normalized = line.split_whitespace().collect::<Vec<_>>().join(" ");
    for wrapped in wrap_line_n(&normalized, TEXT_COLUMNS) {
        result.push(NativeTextLine::body(wrapped));
    }
}

fn wrap_line_n(line: &str, columns: usize) -> Vec<String> {
    let mut rows = Vec::new();
    let mut current = String::new();
    for word in line.split_whitespace() {
        if current.chars().count() + word.chars().count() + WORD_SPACING > columns
            && !current.is_empty()
        {
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
