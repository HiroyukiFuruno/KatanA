mod label;

pub(super) fn replace_with_text_fallbacks(svg_text: &str) -> String {
    let mut result = String::with_capacity(svg_text.len());
    let mut last_end = 0usize;
    for mat in foreign_object_pattern().find_iter(svg_text) {
        result.push_str(&svg_text[last_end..mat.start()]);
        if !has_svg_text_fallback_after(svg_text, mat.start(), mat.end())
            && let Some(text) = label::to_svg_text(mat.as_str())
        {
            result.push_str(&text);
        }
        last_end = mat.end();
    }
    result.push_str(&svg_text[last_end..]);
    result
}

fn foreign_object_pattern() -> &'static regex::Regex {
    static PATTERN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    PATTERN.get_or_init(|| {
        regex::Regex::new(concat!(
            r"(?is)<foreignObject\b[^>]*/>|",
            r"<foreignObject\b[^>]*>.*?</foreignObject>"
        ))
        .expect("valid foreignObject regex")
    })
}

fn has_svg_text_fallback_after(
    svg_text: &str,
    foreign_object_start: usize,
    foreign_object_end: usize,
) -> bool {
    /* WHY: Plain substring search for `<switch` is safe because SVG 1.1 / 2 has no other
    element name starting with `switch` (only `<switch>` exists), so any hit always opens
    the `<switch>` container we care about. */
    let before = &svg_text[..foreign_object_start];
    let Some(switch_start) = before.rfind("<switch") else {
        return false;
    };
    if before
        .rfind("</switch>")
        .is_some_and(|end| end > switch_start)
    {
        return false;
    }

    let after = &svg_text[foreign_object_end..];
    let Some(switch_end) = after.find("</switch>") else {
        return false;
    };
    let switch_tail = &after[..switch_end];
    let Some(text_start) = switch_tail.find("<text") else {
        return false;
    };
    switch_tail
        .find("<foreignObject")
        .is_none_or(|next_foreign_object| text_start < next_foreign_object)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_no_fallback_after_closed_switch() {
        let source = concat!(
            r#"<svg><switch></switch>"#,
            r#"<foreignObject x="0" y="0"><div>Label</div></foreignObject></svg>"#
        );
        let (start, end) = foreign_object_range(source);

        assert!(!has_svg_text_fallback_after(source, start, end));
    }

    #[test]
    fn detects_no_fallback_without_switch_end() {
        let source = concat!(
            r#"<svg><switch>"#,
            r#"<foreignObject x="0" y="0"><div>Label</div></foreignObject></svg>"#
        );
        let (start, end) = foreign_object_range(source);

        assert!(!has_svg_text_fallback_after(source, start, end));
    }

    #[test]
    fn detects_no_fallback_without_svg_text() {
        let source = concat!(
            r#"<svg><switch>"#,
            r#"<foreignObject x="0" y="0"><div>Label</div></foreignObject>"#,
            r#"<rect width="10" height="10"/></switch></svg>"#
        );
        let (start, end) = foreign_object_range(source);

        assert!(!has_svg_text_fallback_after(source, start, end));
    }

    fn foreign_object_range(source: &str) -> (usize, usize) {
        let start = source.find("<foreignObject").unwrap();
        let end = source.find("</foreignObject>").unwrap() + "</foreignObject>".len();
        (start, end)
    }
}
