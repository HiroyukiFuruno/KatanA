pub mod fence;
pub mod html;
pub mod local_image;
pub mod math;

use crate::markdown::DiagramBlock;
use crate::preview::types::{PreviewSection, PreviewSectionOps};
use crate::preview::{DiagramSectionOps, HtmlPreviewOps};

impl PreviewSectionOps {
    pub fn split_sections(content: &str) -> Vec<PreviewSection> {
        let content_processed = content;

        /* WHY: We must extract and remove footnote definitions globally before parsing block boundaries */
        let mut opts = pulldown_cmark::Options::empty();
        opts.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
        let parser = pulldown_cmark::Parser::new_ext(content_processed, opts).into_offset_iter();

        let mut footnote_spans = Vec::new();
        for (event, span) in parser {
            if let pulldown_cmark::Event::Start(pulldown_cmark::Tag::FootnoteDefinition(_)) = event
            {
                footnote_spans.push(span);
            }
        }
        footnote_spans.sort_by_key(|s| s.start);

        let mut clean_content = String::with_capacity(content_processed.len());
        let mut footnotes = String::new();
        let mut last_end = 0;

        for span in footnote_spans {
            if span.start >= last_end {
                clean_content.push_str(&content_processed[last_end..span.start]);
                let footnote_text = &content_processed[span.start..span.end];
                footnotes.push_str(footnote_text);
                footnotes.push('\n');

                /* WHY: Preserve exact line length of the original span so global_line_offset matches */
                /* WHY: Put back the original newlines so the line mapping remains accurate */
                let newlines_count = footnote_text.chars().filter(|c| *c == '\n').count();
                clean_content.extend(std::iter::repeat_n('\n', newlines_count));

                last_end = span.end;
            }
        }
        clean_content.push_str(&content_processed[last_end..]);

        /* WHY: Run the normal marker splitting on the cleaned string to ensure diagram fences parse correctly */
        let mut sections = Vec::new();
        let mut remaining = clean_content.as_str();
        let mut acc = String::new();

        while !remaining.is_empty() {
            let markers = ["```", "~~~", "<mxGraphModel", "@startuml"];
            let mut earliest = None;
            for m in markers {
                let pos_opt = if remaining.starts_with(m) {
                    Some(0)
                } else {
                    /* WHY: Fences must start at the beginning of a line. */
                    remaining.find(&format!("\n{m}")).map(|p| p + 1)
                };

                if let Some(pos) = pos_opt
                    && earliest.is_none_or(|(p, _)| pos < p)
                {
                    earliest = Some((pos, m));
                }
            }

            if let Some((pos, marker)) = earliest {
                let content_from_marker = &remaining[pos..];
                let parsed_fence = DiagramSectionOps::try_parse_diagram_fence(content_from_marker);

                if parsed_fence.is_none() {
                    /* WHY: For non-diagram fences (```markdown, ```rust, etc.) skip the entire
                     * fence body so nested diagram fences inside are not incorrectly extracted.
                     * Delegates to DiagramSectionOps which implements CommonMark fence rules. */
                    let consume_len =
                        DiagramSectionOps::non_diagram_fence_consume_len(pos, marker, remaining);
                    acc.push_str(&remaining[..consume_len]);
                    remaining = &remaining[consume_len..];
                    continue;
                }

                let (kind, source, after) = parsed_fence.unwrap();

                acc.push_str(&remaining[..pos]);
                if !acc.is_empty() {
                    sections.push(PreviewSection::Markdown(
                        acc.clone(),
                        Self::count_lines(&acc),
                    ));
                    acc.clear();
                }

                let consumed_len = content_from_marker.len() - after.len();
                let consumed_text = &content_from_marker[..consumed_len];
                sections.push(PreviewSection::Diagram {
                    kind,
                    source,
                    lines: Self::count_lines(consumed_text),
                });
                remaining = after;
            } else {
                acc.push_str(remaining);
                break;
            }
        }

        if !acc.is_empty() {
            sections.push(PreviewSection::Markdown(
                acc.clone(),
                Self::count_lines(&acc),
            ));
        }

        /* WHY: Append global footnotes to all Markdown sections without changing their line counts */
        if !footnotes.is_empty() {
            let mut has_markdown = false;
            for section in sections.iter_mut() {
                if let PreviewSection::Markdown(md, _) = section {
                    md.push_str("\n\n");
                    md.push_str(&footnotes);
                    has_markdown = true;
                }
            }
            if !has_markdown {
                sections.push(PreviewSection::Markdown(format!("\n\n{}", footnotes), 0));
            }
        }

        sections
    }

    pub fn split_into_sections(content: &str) -> Vec<PreviewSection> {
        let initial = Self::split_sections(content);
        crate::preview::ImageSectionOps::extract_standalone_images(initial)
    }

    pub fn render_sections(secs: Vec<PreviewSection>, base_dir: &std::path::Path) -> String {
        let mut html = String::new();
        for sec in secs {
            match sec {
                PreviewSection::Markdown(md, _) => {
                    html.push_str(&HtmlPreviewOps::parse_html(&md, base_dir));
                }
                PreviewSection::Diagram { kind, source, .. } => {
                    let block = DiagramBlock { kind, source };
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

impl PreviewSectionOps {
    fn count_lines(s: &str) -> usize {
        s.chars().filter(|c| *c == '\n').count() + usize::from(!s.is_empty() && !s.ends_with('\n'))
    }
}
