mod foreign_object;
mod light_dark;

pub(super) fn preprocess_for_rasterizer(svg_text: &str) -> String {
    let with_xml_entities = normalize_html_entities_for_xml(svg_text);
    let without_plantuml_metadata = strip_plantuml_processing_instructions(&with_xml_entities);
    let with_svg_text_fallbacks =
        foreign_object::replace_with_text_fallbacks(&without_plantuml_metadata);
    light_dark::resolve_functions(&with_svg_text_fallbacks)
}

fn normalize_html_entities_for_xml(svg_text: &str) -> String {
    svg_text.replace("&nbsp;", "&#160;")
}

fn strip_plantuml_processing_instructions(svg_text: &str) -> String {
    plantuml_processing_instruction_pattern()
        .replace_all(svg_text, "")
        .to_string()
}

fn plantuml_processing_instruction_pattern() -> &'static regex::Regex {
    static PATTERN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    PATTERN.get_or_init(|| {
        regex::Regex::new(r"(?is)<\?plantuml(?:-src)?\b.*?\?>")
            .expect("valid PlantUML processing instruction regex")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preprocess_reuses_invalid_light_dark_syntax() {
        let source = "<svg>fill: light-dark(red)</svg>";
        let output = preprocess_for_rasterizer(source);
        assert_eq!(output, source);
    }

    #[test]
    fn preprocess_converts_foreign_object_label_to_svg_text() {
        let source = concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="60">"#,
            r#"<foreignObject x="10" y="20" width="80" height="24">"#,
            r#"<div xmlns="http://www.w3.org/1999/xhtml" "#,
            r#"style="font-size: 12px; color: #E0E0E0;">Node &amp; Label</div>"#,
            r#"</foreignObject></svg>"#
        );

        let output = preprocess_for_rasterizer(source);

        assert!(!output.contains("<foreignObject"));
        assert!(output.contains("<text"));
        assert!(output.contains(r##"fill="#E0E0E0""##));
        assert!(output.contains(">Node &amp; Label<"));
    }

    #[test]
    fn preprocess_uses_existing_svg_text_in_switch_fallback() {
        let source = concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="60">"#,
            r#"<switch><foreignObject x="10" y="20" width="80" height="24">"#,
            r#"<div xmlns="http://www.w3.org/1999/xhtml">HTML Label</div>"#,
            r#"</foreignObject><text x="10" y="20">SVG Label</text></switch></svg>"#
        );

        let output = preprocess_for_rasterizer(source);

        assert!(!output.contains("<foreignObject"));
        assert!(!output.contains("HTML Label"));
        assert!(output.contains("SVG Label"));
        assert_eq!(output.matches("<text").count(), 1);
    }

    #[test]
    fn preprocess_removes_plantuml_processing_instructions() {
        let source = concat!(
            r#"<?plantuml 1.2026.2?>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="20px" height="20px">"#,
            r##"<g><?plantuml-src abc?><rect width="20" height="20" fill="#2D2D2D"/>"##,
            r##"<text x="2" y="12" fill="#E0E0E0">PUML</text></g></svg>"##
        );

        let output = preprocess_for_rasterizer(source);

        assert!(!output.contains("<?plantuml"));
    }
}
