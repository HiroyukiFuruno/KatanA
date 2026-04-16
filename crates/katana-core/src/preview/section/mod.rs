pub mod fence;
pub mod html;
pub mod local_image;
pub mod math;

use crate::markdown::DiagramBlock;
use crate::preview::types::{PreviewSection, PreviewSectionOps};
use crate::preview::{DiagramSectionOps, HtmlPreviewOps, MathPreviewOps};

impl PreviewSectionOps {
    pub fn split_sections(markdown: &str) -> Vec<PreviewSection> {
        let mut sections = Vec::new();
        let mut remaining = markdown;

        while !remaining.is_empty() {
            /* WHY: Find the earliest occurrence of any supported diagram marker. */
            let markers = ["```", "<mxGraphModel", "@startuml"];
            let mut earliest = None;
            for m in markers {
                if let Some(pos) = remaining.find(m) {
                    if earliest.map_or(true, |(p, _)| pos < p) {
                        earliest = Some((pos, m));
                    }
                }
            }

            if let Some((pos, marker)) = earliest {
                if pos > 0 {
                    sections.push(PreviewSection::Markdown(remaining[..pos].to_string()));
                }

                let content_from_marker = &remaining[pos..];
                if let Some((kind, source, after)) = DiagramSectionOps::try_parse_diagram_fence(content_from_marker) {
                    sections.push(PreviewSection::Diagram { kind, source, lines: 0 });
                    remaining = after;
                } else {
                    /* WHY: If parsing fails, consume the marker as plain text to avoid infinite loop. */
                    sections.push(PreviewSection::Markdown(marker.to_string()));
                    remaining = &content_from_marker[marker.len()..];
                }
            } else {
                sections.push(PreviewSection::Markdown(remaining.to_string()));
                break;
            }
        }

        sections
    }

    pub fn split_into_sections(content: &str) -> Vec<PreviewSection> {
        Self::split_sections(content)
    }

    pub fn render_sections(secs: Vec<PreviewSection>, base_dir: &std::path::Path) -> String {
        let mut html = String::new();
        for sec in secs {
            match sec {
                PreviewSection::Markdown(md) => {
                    html.push_str(&HtmlPreviewOps::parse_html(&md, base_dir));
                }
                PreviewSection::Diagram { kind, source, .. } => {
                    let source_cow = MathPreviewOps::process_relaxed_math(&source);
                    let block = DiagramBlock {
                        kind,
                        source: source_cow.into_owned(),
                    };
                    html.push_str(&block.render().to_html());
                }
                PreviewSection::LocalImage { path, alt, .. } => {
                    html.push_str(&format!(r#"<img src="{path}" alt="{alt}">"#));
                }
            }
        }
        html
    }
}
