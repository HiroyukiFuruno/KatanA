/* WHY: Verification of Markdown inline elements (images, links) that are often embedded or mixed with HTML tags. */

use super::*;
use crate::html::{HtmlNode, LinkTarget};
use std::path::PathBuf;

#[test]
fn parse_md_image() {
    let nodes = parser().parse("![alt text](image.png)");
    assert_eq!(
        nodes,
        vec![HtmlNode::Image {
            src: "image.png".into(),
            alt: "alt text".into(),
        }]
    );
}

#[test]
fn parse_md_link() {
    let nodes = parser().parse("[ {65e5} {672c} {8a9e}](README.ja.md)");
    assert_eq!(
        nodes,
        vec![HtmlNode::Link {
            target: LinkTarget::InternalFile(PathBuf::from("/project/README.ja.md")),
            children: vec![HtmlNode::Text(" {65e5} {672c} {8a9e}".into())],
        }]
    );
}

#[test]
fn parse_md_external_link() {
    let nodes = parser().parse("[GitHub](https://github.com)");
    assert_eq!(
        nodes,
        vec![HtmlNode::Link {
            target: LinkTarget::External("https://github.com".into()),
            children: vec![HtmlNode::Text("GitHub".into())],
        }]
    );
}

#[test]
fn parse_text_with_md_link() {
    let nodes = parser().parse("English | [ {65e5} {672c} {8a9e}](README.ja.md)");
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0], HtmlNode::Text("English | ".into()));
    assert_eq!(
        nodes[1],
        HtmlNode::Link {
            target: LinkTarget::InternalFile(PathBuf::from("/project/README.ja.md")),
            children: vec![HtmlNode::Text(" {65e5} {672c} {8a9e}".into())],
        }
    );
}
