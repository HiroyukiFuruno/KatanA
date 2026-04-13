mod types;
pub use types::*;

use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, Options, parse_document};

fn extract_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut text = String::new();
    for child in node.children() {
        extract_text_from_node(child, &mut text);
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_text_from_node<'a>(node: &'a AstNode<'a>, out: &mut String) {
    match &node.data.borrow().value {
        NodeValue::Text(text) => out.push_str(text),
        NodeValue::Code(code) => out.push_str(&code.literal),
        NodeValue::SoftBreak | NodeValue::LineBreak => out.push(' '),
        _ => {}
    }
    for child in node.children() {
        extract_text_from_node(child, out);
    }
}

pub struct MarkdownOutlineOps;

impl MarkdownOutlineOps {
    pub fn extract_outline(source: &str) -> (Vec<OutlineItem>, Vec<DocumentAnchor>) {
        let arena = Arena::new();
        let mut options = Options::default();
        options.extension.strikethrough = true;
        options.extension.table = true;
        options.extension.tasklist = true;
        options.extension.autolink = true;
        options.extension.front_matter_delimiter = Some("---".to_string());

        let root = parse_document(&arena, source, &options);
        let mut outline = Vec::new();
        let mut anchors = Vec::new();
        let mut index = 0;

        for node in root.descendants() {
            let line_start = node.data.borrow().sourcepos.start.line.saturating_sub(1);
            let line_end = node.data.borrow().sourcepos.end.line.saturating_sub(1);
            if let NodeValue::Heading(ref heading) = node.data.borrow().value {
                let text = extract_text(node);
                outline.push(OutlineItem {
                    level: heading.level,
                    text,
                    index,
                    line_start,
                    line_end,
                });
                anchors.push(DocumentAnchor {
                    kind: AnchorKind::Heading,
                    line_start,
                    line_end,
                });
                index += 1;
            } else if matches!(
                node.data.borrow().value,
                NodeValue::CodeBlock(_)
                    | NodeValue::Paragraph
                    | NodeValue::Table(_)
                    | NodeValue::BlockQuote
            ) {
                anchors.push(DocumentAnchor {
                    kind: AnchorKind::Block,
                    line_start,
                    line_end,
                });
            }
        }

        (outline, anchors)
    }
}

#[cfg(test)]
mod tests;
