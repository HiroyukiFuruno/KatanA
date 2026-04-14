use super::*;

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
