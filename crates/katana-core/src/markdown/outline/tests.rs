use super::*;
use comrak::nodes::NodeValue;
use comrak::{Arena, Options, parse_document};

#[test]
fn test_extract_outline_with_line_breaks() {
    let md = "Heading\nwith softbreak\n=======\n\nHeading  \nwith linebreak\n-------";
    let outline = MarkdownOutlineOps::extract_outline(md).0;
    assert_eq!(outline.len(), 2);
    assert_eq!(outline[0].text, "Heading with softbreak");
    assert_eq!(outline[1].text, "Heading with linebreak");
}

#[test]
fn test_extract_outline_with_complex_formatting() {
    let md = r#"
# Heading with `code`
Some text
## Heading with [link](http://example.com)
More text
### **Bold** and *italic*
"#;
    let outline = MarkdownOutlineOps::extract_outline(md).0;
    assert_eq!(outline.len(), 3);
    assert_eq!(outline[0].text, "Heading with code");
    assert_eq!(outline[1].text, "Heading with link");
    assert_eq!(outline[2].text, "Bold and italic");
}

#[test]
fn extract_text_collects_nested_code_text() {
    let arena = Arena::new();
    let root = parse_document(&arena, "## **Bold `code`**\n", &Options::default());
    let heading = root
        .descendants()
        .find(|node| matches!(node.data.borrow().value, NodeValue::Heading(_)))
        .expect("heading exists");

    assert_eq!(extract_text(heading), "Bold code");
}

#[test]
fn replace_markdown_links_keeps_unclosed_link_text() {
    assert_eq!(
        MarkdownOutlineOps::replace_markdown_links("Heading [broken"),
        "Heading [broken"
    );
    assert_eq!(
        MarkdownOutlineOps::replace_markdown_links("Heading [label](broken"),
        "Heading [label](broken"
    );
}

#[test]
fn test_extract_outline() {
    let md = r#"
# Heading 1
Some text.
## Heading 2
More text.
### Heading 3
"#;
    let outline = MarkdownOutlineOps::extract_outline(md).0;
    assert_eq!(outline.len(), 3);
    assert_eq!(
        outline[0],
        OutlineItem {
            level: 1,
            text: "Heading 1".to_string(),
            index: 0,
            line_start: 1,
            line_end: 1,
        }
    );
    assert_eq!(
        outline[1],
        OutlineItem {
            level: 2,
            text: "Heading 2".to_string(),
            index: 1,
            line_start: 3,
            line_end: 3,
        }
    );
    assert_eq!(
        outline[2],
        OutlineItem {
            level: 3,
            text: "Heading 3".to_string(),
            index: 2,
            line_start: 5,
            line_end: 5,
        }
    );
}
