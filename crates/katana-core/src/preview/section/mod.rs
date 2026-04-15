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
        let mut acc = String::new();
        let mut rem = content;

        while let Some(offset) = Self::find_next_fence_offset(rem) {
            acc.push_str(&rem[..offset]);
            rem = &rem[offset..];

            if let Some((kind, source, after)) = DiagramSectionOps::try_parse_diagram_fence(rem) {
                Self::push_markdown_section(&mut sections, &mut acc);

                /* WHY: source's '\n' count gives (content_lines - 1). We add
                FENCE_LINE_COUNT = opening fence (1) + closing fence (1) + 1 for
                the newline-count-to-line-count conversion. */
                const FENCE_LINE_COUNT: usize = 3;
                let lines = source.chars().filter(|c| *c == '\n').count() + FENCE_LINE_COUNT;
                sections.push(PreviewSection::Diagram {
                    kind,
                    source,
                    lines,
                });
                rem = after;

                /* WHY: Force isolation by inserting a mandatory newline for the next segment.
                This ensures block-level elements (headers, etc.) are correctly parsed. */
                acc.push('\n');
            } else {
                acc.push_str("```");
                rem = &rem["```".len()..];
            }
        }

        acc.push_str(rem);
        if !acc.is_empty() {
            sections.push(PreviewSection::Markdown(acc));
        }

        ImageSectionOps::extract_standalone_images(sections)
    }

    fn find_next_fence_offset(rem: &str) -> Option<usize> {
        if rem.starts_with("```") {
            Some(0)
        } else {
            rem.find("\n```").map(|p| p + 1)
        }
    }

    fn push_markdown_section(sections: &mut Vec<PreviewSection>, acc: &mut String) {
        if acc.is_empty() {
            return;
        }
        /* WHY: Ensure markdown ends with newline to avoid joining blocks with diagrams. */
        if !acc.ends_with('\n') {
            acc.push('\n');
        }
        sections.push(PreviewSection::Markdown(std::mem::take(acc)));
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
