use crate::markdown::MarkdownError;

const TEXT_COLUMNS: usize = 82;

pub(super) fn extract_lines(html: &str) -> Result<Vec<String>, MarkdownError> {
    let body = body_content(html)?;
    let with_image_alt = replace_image_alt(&body)?;
    let without_scripts = remove_tag_blocks(&with_image_alt, "script")?;
    let without_style = remove_tag_blocks(&without_scripts, "style")?;
    let with_breaks = block_tags_to_breaks(&without_style)?;
    let without_tags = strip_tags(&with_breaks)?;
    Ok(normalized_lines(&decode_entities(&without_tags)))
}

fn body_content(html: &str) -> Result<String, MarkdownError> {
    let regex = regex::Regex::new(r"(?is)<body[^>]*>(.*?)</body>")
        .map_err(|error| MarkdownError::ExportFailed(error.to_string()))?;
    Ok(regex
        .captures(html)
        .and_then(|captures| captures.get(1))
        .map(|body| body.as_str().to_string())
        .unwrap_or_else(|| html.to_string()))
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
        r"(?is)</?(h[1-6]|p|div|section|article|header|footer|li|tr|table|pre|blockquote|br|hr)\b[^>]*>",
    )
    .map_err(|error| MarkdownError::ExportFailed(error.to_string()))?;
    Ok(regex.replace_all(html, "\n").to_string())
}

fn strip_tags(html: &str) -> Result<String, MarkdownError> {
    let regex = regex::Regex::new(r"(?is)<[^>]+>")
        .map_err(|error| MarkdownError::ExportFailed(error.to_string()))?;
    Ok(regex.replace_all(html, " ").to_string())
}

fn normalized_lines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for raw_line in text.lines() {
        let line = raw_line.split_whitespace().collect::<Vec<_>>().join(" ");
        if line.is_empty() {
            continue;
        }
        lines.extend(wrap_line(&line));
    }
    lines
}

fn wrap_line(line: &str) -> Vec<String> {
    let mut rows = Vec::new();
    let mut current = String::new();
    for word in line.split_whitespace() {
        if current.chars().count() + word.chars().count() + 1 > TEXT_COLUMNS && !current.is_empty()
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

fn decode_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}
