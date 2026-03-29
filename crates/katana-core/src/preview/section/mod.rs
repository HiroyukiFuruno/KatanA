mod fence;
mod html;
mod local_image;
mod math;

use crate::markdown::diagram::DiagramKind;
use fence::try_parse_diagram_fence;
pub use html::wrap_standalone_inline_html;
use local_image::extract_standalone_images;
use math::process_relaxed_math;

/// The type of section that makes up a document.
#[derive(Debug, Clone)]
pub enum PreviewSection {
    /// Normal Markdown text.
    Markdown(String),
    /// A diagram fence block.
    Diagram {
        kind: DiagramKind,
        source: String,
        lines: usize,
    },
    /// A standalone local image.
    LocalImage {
        path: String,
        alt: String,
        lines: usize,
    },
}

/// Splits the source text into a list of `PreviewSection`s.
///
/// Detects diagram fences (` ```mermaid` / ` ```plantuml` / ` ```drawio` ),
/// and groups the rest as Markdown sections.
pub fn split_into_sections(source: &str) -> Vec<PreviewSection> {
    // WHY: Pre-processing: Relaxed inline math delimiters
    let source_cow = process_relaxed_math(source);
    let source_processed = source_cow.as_ref();

    let mut initial_sections = Vec::new();
    let mut markdown_acc = String::new();
    let mut remaining = source_processed;

    loop {
        // WHY: Find the next fence: either at the very start of remaining, or after a newline.
        let fence_offset = if remaining.starts_with("```") {
            Some(0)
        } else {
            remaining.find("\n```").map(|pos| pos + 1)
        };
        let Some(offset) = fence_offset else {
            break;
        };

        markdown_acc.push_str(&remaining[..offset]);
        remaining = &remaining[offset..];
        match try_parse_diagram_fence(remaining) {
            Some((kind, fence_source, after)) => {
                // WHY: Do not wrap here, we will wrap in the final merge pass
                if !markdown_acc.is_empty() {
                    initial_sections
                        .push(PreviewSection::Markdown(std::mem::take(&mut markdown_acc)));
                }
                let lines = fence_source.chars().filter(|c| *c == '\n').count();
                initial_sections.push(PreviewSection::Diagram {
                    kind,
                    source: fence_source,
                    lines,
                });
                remaining = after;
            }
            None => {
                // WHY: If not a diagram, treat as plain Markdown.
                markdown_acc.push_str("```");
                remaining = &remaining["```".len()..];
            }
        }
    }

    markdown_acc.push_str(remaining);
    if !markdown_acc.is_empty() {
        initial_sections.push(PreviewSection::Markdown(std::mem::take(&mut markdown_acc)));
    }

    // WHY: We need to pull standalone images to apply different HTML/Markdown transformations on them
    let temp = extract_standalone_images(initial_sections);

    // WHY: Merge adjacent text sections together to prevent fragmented parsing, and apply HTML wrapping for standalone inline elements so they render correctly
    let mut merged = Vec::new();
    let mut md_acc = String::new();
    for sec in temp {
        match sec {
            PreviewSection::Markdown(t) => {
                md_acc.push_str(&t);
                md_acc.push('\n');
            }
            other => {
                if !md_acc.is_empty() {
                    let processed = wrap_standalone_inline_html(&md_acc);
                    merged.push(PreviewSection::Markdown(processed));
                    md_acc.clear();
                }
                merged.push(other);
            }
        }
    }
    if !md_acc.is_empty() {
        let processed = wrap_standalone_inline_html(&md_acc);
        merged.push(PreviewSection::Markdown(processed));
    }

    merged
}
