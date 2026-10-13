mod types;
pub use types::*;

use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, Options, parse_document};
use katana_markdown_model::{KatanaMarkdownModel, KmmNodeKind, MarkdownInput};

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
        match Self::extract_with_kmm(source) {
            Some(kmm_result) if !kmm_result.0.is_empty() => kmm_result,
            Some(kmm_result) => {
                let comrak_result = Self::extract_with_comrak(source);
                if comrak_result.0.is_empty() {
                    kmm_result
                } else {
                    comrak_result
                }
            }
            None => Self::extract_with_comrak(source),
        }
    }

    fn extract_with_kmm(source: &str) -> Option<(Vec<OutlineItem>, Vec<DocumentAnchor>)> {
        let document = KatanaMarkdownModel::parse(MarkdownInput::from_content("", source)).ok()?;
        let mut outline = Vec::new();
        let mut anchors = Vec::new();
        let mut index = 0;

        for node in document.nodes {
            let line_start = node.source.line_column_range.start.line.saturating_sub(1);
            let line_end = node.source.line_column_range.end.line.saturating_sub(1);
            match node.kind {
                KmmNodeKind::Heading(heading) => {
                    outline.push(OutlineItem {
                        level: heading.level,
                        text: Self::normalize_kmm_heading_text(&heading.text),
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
                }
                kind if Self::is_kmm_block_anchor(&kind) => {
                    anchors.push(DocumentAnchor {
                        kind: AnchorKind::Block,
                        line_start,
                        line_end,
                    });
                }
                _ => {}
            }
        }

        Some((outline, anchors))
    }

    fn is_kmm_block_anchor(kind: &KmmNodeKind) -> bool {
        matches!(
            kind,
            KmmNodeKind::Paragraph
                | KmmNodeKind::CodeBlock(_)
                | KmmNodeKind::Table(_)
                | KmmNodeKind::BlockQuote
                | KmmNodeKind::Alert { .. }
                | KmmNodeKind::DescriptionList { .. }
                | KmmNodeKind::HtmlBlock(_)
                | KmmNodeKind::RawBlock { .. }
        )
    }

    fn normalize_kmm_heading_text(text: &str) -> String {
        let linked = Self::replace_markdown_links(text);
        linked
            .chars()
            .filter(|character| !matches!(character, '`' | '*' | '_'))
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn replace_markdown_links(text: &str) -> String {
        let mut output = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(label_start) = rest.find('[') {
            output.push_str(&rest[..label_start]);
            let after_label_start = &rest[label_start + 1..];
            let Some(label_end) = after_label_start.find("](") else {
                output.push_str(&rest[label_start..]);
                return output;
            };
            let after_url_start = &after_label_start[label_end + 2..];
            let Some(url_end) = after_url_start.find(')') else {
                output.push_str(&rest[label_start..]);
                return output;
            };
            output.push_str(&after_label_start[..label_end]);
            rest = &after_url_start[url_end + 1..];
        }
        output.push_str(rest);
        output
    }

    fn extract_with_comrak(source: &str) -> (Vec<OutlineItem>, Vec<DocumentAnchor>) {
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
