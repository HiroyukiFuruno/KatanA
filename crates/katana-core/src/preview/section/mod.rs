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
            if let Some(fence_start) = remaining.find("```") {
                if fence_start > 0 {
                    sections.push(PreviewSection::Markdown(remaining[..fence_start].to_string()));
                }

                let content_after_fence = &remaining[fence_start..];
                if let Some((kind, source, _)) = DiagramSectionOps::try_parse_diagram_fence(content_after_fence) {
                    let consumed = source.len() + 10; 
                    sections.push(PreviewSection::Diagram { kind, source, lines: 0 });
                    remaining = &content_after_fence[consumed.min(content_after_fence.len())..];
                } else {
                    sections.push(PreviewSection::Markdown("```".to_string()));
                    remaining = &content_after_fence[3..];
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
