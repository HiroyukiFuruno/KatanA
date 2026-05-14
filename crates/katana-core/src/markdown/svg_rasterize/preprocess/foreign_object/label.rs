const DEFAULT_FOREIGN_OBJECT_FONT_SIZE: f32 = 14.0;
const FOREIGN_OBJECT_BASELINE_OFFSET_RATIO: f32 = 0.35;

pub(super) fn to_svg_text(fragment: &str) -> Option<String> {
    let element = xmltree::Element::parse(fragment.as_bytes()).ok()?;
    let label = normalized_text_content(&element);
    if label.is_empty() {
        return None;
    }

    let x = parse_number_attr(&element, "x").unwrap_or(0.0);
    let y = parse_number_attr(&element, "y").unwrap_or(0.0);
    let width = parse_number_attr(&element, "width").unwrap_or(0.0);
    let height = parse_number_attr(&element, "height").unwrap_or(0.0);
    let font_size = find_style_property(&element, "font-size")
        .as_deref()
        .and_then(parse_leading_number)
        .unwrap_or(DEFAULT_FOREIGN_OBJECT_FONT_SIZE);
    let fill = find_style_property(&element, "color")
        .or_else(|| find_style_property(&element, "fill"))
        .or_else(|| find_attr_recursive(&element, "fill").map(ToString::to_string))
        .unwrap_or_else(|| "currentColor".to_string());
    let text_anchor = resolve_text_anchor(&element);
    let text_x = match text_anchor {
        "middle" => x + (width / 2.0),
        "end" => x + width,
        _ => x,
    };
    let text_y = if height > 0.0 {
        y + (height / 2.0) + (font_size * FOREIGN_OBJECT_BASELINE_OFFSET_RATIO)
    } else {
        y + font_size
    };

    Some(format!(
        r#"<text x="{x}" y="{y}" fill="{fill}" font-size="{font_size}" text-anchor="{anchor}">{label}</text>"#,
        x = format_svg_number(text_x),
        y = format_svg_number(text_y),
        fill = escape_xml_attr(&fill),
        font_size = format_svg_number(font_size),
        anchor = text_anchor,
        label = escape_xml_text(&label),
    ))
}

fn normalized_text_content(element: &xmltree::Element) -> String {
    let mut text = String::new();
    collect_text_content(element, &mut text);
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn collect_text_content(element: &xmltree::Element, output: &mut String) {
    for child in &element.children {
        match child {
            xmltree::XMLNode::Text(text) | xmltree::XMLNode::CData(text) => {
                output.push(' ');
                output.push_str(text);
            }
            xmltree::XMLNode::Element(child_element) => {
                if child_element.name.eq_ignore_ascii_case("br") {
                    output.push(' ');
                }
                collect_text_content(child_element, output);
            }
            _ => {}
        }
    }
}

fn parse_number_attr(element: &xmltree::Element, name: &str) -> Option<f32> {
    element
        .attributes
        .get(name)
        .and_then(|value| parse_leading_number(value))
}

fn find_style_property(element: &xmltree::Element, property: &str) -> Option<String> {
    element
        .attributes
        .get("style")
        .and_then(|style| style_property(style, property))
        .or_else(|| {
            element.children.iter().find_map(|child| match child {
                xmltree::XMLNode::Element(child_element) => {
                    find_style_property(child_element, property)
                }
                _ => None,
            })
        })
}

fn style_property(style: &str, property: &str) -> Option<String> {
    style.split(';').find_map(|declaration| {
        let (name, value) = declaration.split_once(':')?;
        name.trim()
            .eq_ignore_ascii_case(property)
            .then(|| value.trim().to_string())
    })
}

fn find_attr_recursive<'a>(element: &'a xmltree::Element, name: &str) -> Option<&'a str> {
    element
        .attributes
        .get(name)
        .map(String::as_str)
        .or_else(|| {
            element.children.iter().find_map(|child| match child {
                xmltree::XMLNode::Element(child_element) => {
                    find_attr_recursive(child_element, name)
                }
                _ => None,
            })
        })
}

fn resolve_text_anchor(element: &xmltree::Element) -> &'static str {
    find_style_property(element, "text-align")
        .as_deref()
        .map(str::trim)
        .map(|align| {
            if align.eq_ignore_ascii_case("right") {
                "end"
            } else if align.eq_ignore_ascii_case("left") {
                "start"
            } else {
                "middle"
            }
        })
        .unwrap_or("middle")
}

fn parse_leading_number(value: &str) -> Option<f32> {
    let number_end = value
        .char_indices()
        .find_map(|(index, character)| {
            (!matches!(character, '0'..='9' | '.' | '-' | '+')).then_some(index)
        })
        .unwrap_or(value.len());
    value[..number_end].trim().parse::<f32>().ok()
}

fn format_svg_number(value: f32) -> String {
    if value.fract().abs() < f32::EPSILON {
        format!("{value:.0}")
    } else {
        format!("{value:.3}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn escape_xml_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_xml_attr(text: &str) -> String {
    escape_xml_text(text).replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_right_aligned_label_to_end_anchor() {
        let output = to_svg_text(concat!(
            r#"<foreignObject x="10" y="20" width="80" height="24">"#,
            r#"<div xmlns="http://www.w3.org/1999/xhtml" "#,
            r#"style="text-align: right; color: #fff;">Right</div></foreignObject>"#
        ))
        .unwrap();

        assert!(output.contains(r#"x="90""#));
        assert!(output.contains(r#"text-anchor="end""#));
    }

    #[test]
    fn converts_left_aligned_label_to_start_anchor() {
        let output = to_svg_text(concat!(
            r#"<foreignObject x="10" y="20" width="80" height="0">"#,
            r#"<div xmlns="http://www.w3.org/1999/xhtml" "#,
            r#"style="text-align: left; font-size: 12px;">Left</div></foreignObject>"#
        ))
        .unwrap();

        assert!(output.contains(r#"x="10""#));
        assert!(output.contains(r#"y="32""#));
        assert!(output.contains(r#"text-anchor="start""#));
    }

    #[test]
    fn converts_center_aligned_label_to_middle_anchor() {
        let output = to_svg_text(concat!(
            r#"<foreignObject x="10" y="20" width="80" height="24">"#,
            r#"<div xmlns="http://www.w3.org/1999/xhtml" "#,
            r#"style="text-align: center;">Center</div></foreignObject>"#
        ))
        .unwrap();

        assert!(output.contains(r#"x="50""#));
        assert!(output.contains(r#"text-anchor="middle""#));
    }

    #[test]
    fn uses_nested_fill_attribute_when_style_color_is_missing() {
        let output = to_svg_text(concat!(
            r#"<foreignObject x="0" y="0" width="20" height="20">"#,
            r#"<div xmlns="http://www.w3.org/1999/xhtml">"#,
            r##"<span fill="#123456">Nested</span><!-- ignored --></div></foreignObject>"##
        ))
        .unwrap();

        assert!(output.contains(r##"fill="#123456""##));
        assert!(output.contains(">Nested<"));
    }

    #[test]
    fn normalizes_line_breaks_in_label_text() {
        let output = to_svg_text(concat!(
            r#"<foreignObject x="0" y="0" width="80" height="20">"#,
            r#"<div xmlns="http://www.w3.org/1999/xhtml">First<br/>Second</div>"#,
            r#"</foreignObject>"#
        ))
        .unwrap();

        assert!(output.contains(">First Second<"));
    }
}
