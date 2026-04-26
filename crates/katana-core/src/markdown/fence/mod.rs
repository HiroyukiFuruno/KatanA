mod delimiter;
mod types;
pub use delimiter::*;
pub use types::*;

use crate::markdown::{DiagramBlock, DiagramKind, DiagramRenderer, DiagramResult};

const MIN_FENCE_MARKER_LEN: usize = 3;

pub struct MarkdownFenceOps;

impl MarkdownFenceOps {
    pub fn extract_fence_block(s: &str) -> Option<(FenceBlock, &str)> {
        let delimiter = MarkdownFenceDelimiter::parse_at(s)?;
        let body = &s[delimiter.byte_len()..];
        let info_end = body.find('\n')?;
        let info = body[..info_end].trim().to_string();
        let after_info = &body[info_end + 1..];
        let closing = delimiter.find_closing(after_info)?;

        let content = after_info[..closing.content_end].to_string();
        let raw_len = delimiter.byte_len() + info_end + 1 + closing.close_end;
        let raw = s[..raw_len].to_string();
        let rest_slice = &after_info[closing.close_end..];
        let rest = rest_slice.strip_prefix('\n').unwrap_or(rest_slice);

        Some((FenceBlock { info, content, raw }, rest))
    }

    pub fn html_escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }

    pub fn fallback_html(source: &str, error: &str) -> String {
        format!(
            r#"<div class="katana-diagram-error"><p class="katana-diagram-error-label">⚠ Diagram render failed: {e}</p><pre><code>{s}</code></pre></div>"#,
            e = Self::html_escape(error),
            s = Self::html_escape(source),
        )
    }

    pub fn render_diagram_block<R: DiagramRenderer>(
        block: &FenceBlock,
        renderer: &R,
    ) -> Option<String> {
        let kind = DiagramKind::from_info(&block.info)?;
        let diagram = DiagramBlock {
            kind,
            source: block.content.clone(),
        };
        Some(match renderer.render(&diagram) {
            DiagramResult::Ok(html) => html,
            DiagramResult::OkPng(bytes) => {
                use base64::Engine;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                format!(
                    r#"<div class="katana-diagram mermaid"><img src="data:image/png;base64,{b64}" style="max-width:100%" /></div>"#
                )
            }
            DiagramResult::Err { source, error } => Self::fallback_html(&source, &error),
            DiagramResult::CommandNotFound {
                tool_name,
                install_hint,
                ..
            } => Self::fallback_html("", &format!("{tool_name} not found. {install_hint}")),
            DiagramResult::NotInstalled { kind, .. } => {
                Self::fallback_html("", &format!("{kind} is not installed"))
            }
        })
    }

    pub fn process_fence<R: DiagramRenderer>(
        output: &mut String,
        remaining: &mut &str,
        renderer: &R,
    ) {
        let extracted = Self::extract_fence_block(remaining);
        if extracted.is_none() {
            eprintln!("FAILED TO EXTRACT: {:.20}", remaining);
        }
        let Some((block, after)) = extracted else {
            let marker_len = MarkdownFenceDelimiter::parse_at(remaining)
                .map_or(MIN_FENCE_MARKER_LEN, |it| it.byte_len());
            output.push_str(&remaining[..marker_len]);
            *remaining = &remaining[marker_len..];
            return;
        };
        if let Some(html) = Self::render_diagram_block(&block, renderer) {
            output.push_str("\n\n");
            output.push_str(&html);
            /* WHY: CommonMark [HTML blocks, type 6] specifies that blocks end with a blank line.
            Without explicit \n\n, subsequent Markdown elements like # Headings are swallowed! */
            output.push_str("\n\n");
        } else {
            output.push_str(&block.raw);
        }
        *remaining = after;
    }

    pub fn transform_diagram_blocks<R: DiagramRenderer>(source: &str, renderer: &R) -> String {
        let mut output = String::with_capacity(source.len());
        let mut remaining = source;
        loop {
            /* WHY: Find the closest diagram marker (fenced or unfenced) */
            let find = |s: &str, n: &str| {
                if remaining.starts_with(s) {
                    0
                } else {
                    remaining.find(n).map(|p| p + 1).unwrap_or(usize::MAX)
                }
            };
            let pf = find("```", "\n```");
            let pt = find("~~~", "\n~~~");
            let pd = find("<mxGraphModel", "\n<mxGraphModel");
            let pp = find("@startuml", "\n@startuml");

            let Some((offset, marker_type)) = [
                (pf, MarkdownDiagramMarker::Fence),
                (pt, MarkdownDiagramMarker::Fence),
                (pd, MarkdownDiagramMarker::Drawio),
                (pp, MarkdownDiagramMarker::Plantuml),
            ]
            .into_iter()
            .filter(|&(p, _)| p != usize::MAX)
            .min_by_key(|&(p, _)| p) else {
                break;
            };

            output.push_str(&remaining[..offset]);
            remaining = &remaining[offset..];

            match marker_type {
                MarkdownDiagramMarker::Fence => {
                    Self::process_fence(&mut output, &mut remaining, renderer);
                }
                MarkdownDiagramMarker::Drawio => Self::process_raw_tag_diagram(
                    &mut output,
                    &mut remaining,
                    renderer,
                    "<mxGraphModel",
                    "</mxGraphModel>",
                    "drawio",
                ),
                MarkdownDiagramMarker::Plantuml => Self::process_raw_tag_diagram(
                    &mut output,
                    &mut remaining,
                    renderer,
                    "@startuml",
                    "@enduml",
                    "plantuml",
                ),
            }
        }
        output.push_str(remaining);
        output
    }

    fn process_raw_tag_diagram<R: DiagramRenderer>(
        output: &mut String,
        remaining: &mut &str,
        renderer: &R,
        start_tag: &str,
        end_tag: &str,
        info: &str,
    ) {
        if let Some(end_pos) = remaining.find(end_tag) {
            let content = remaining[..end_pos + end_tag.len()].to_string();
            let html = Self::render_diagram_block(
                &FenceBlock {
                    info: info.to_string(),
                    raw: content.clone(),
                    content,
                },
                renderer,
            )
            .expect("Raw target diagrams known to not return None");
            output.push_str("\n\n");
            output.push_str(&html);
            output.push_str("\n\n");
            let after = &remaining[end_pos + end_tag.len()..];
            *remaining = after.strip_prefix('\n').unwrap_or(after);
        } else {
            output.push_str(start_tag);
            *remaining = &remaining[start_tag.len()..];
        }
    }
}

#[cfg(test)]
mod tests;
