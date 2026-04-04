pub mod fence;
pub mod html;
pub mod local_image;
pub mod math;

use crate::markdown::DiagramBlock;
use crate::preview::types::{PreviewSection, PreviewSectionOps};
use crate::preview::{DiagramSectionOps, HtmlPreviewOps, ImageSectionOps, MathPreviewOps};

impl PreviewSectionOps {
    pub fn split_sections(content: &str) -> Vec<PreviewSection> {
        let mut sections = Vec::new();
        let mut rem = content;

        while !rem.is_empty() {
            if let Some((kind, source, after)) = DiagramSectionOps::try_parse_diagram_fence(rem) {
                let lines = source.chars().filter(|c| *c == '\n').count() + 2;
                sections.push(PreviewSection::Diagram {
                    kind,
                    source,
                    lines,
                });
                rem = after;
                continue;
            }

            let next_fence = rem.find("\n```").unwrap_or(rem.len());
            if next_fence == 0 {
                // WHY: Consumer the leading newline that would otherwise cause an infinite loop.
                if let Some(PreviewSection::Markdown(md)) = sections.last_mut() {
                    md.push('\n');
                } else {
                    sections.push(PreviewSection::Markdown("\n".to_string()));
                }
                rem = &rem[1..];
            } else {
                let current = &rem[..next_fence];
                if !current.is_empty() {
                    sections.push(PreviewSection::Markdown(current.to_string()));
                }
                rem = &rem[next_fence..];
            }
        }

        ImageSectionOps::extract_standalone_images(sections)
    }

    /// Compatibility alias for `split_sections`.
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
