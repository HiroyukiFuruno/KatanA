mod types;
pub use types::*;

use crate::markdown::{DiagramBlock, DiagramKind, DiagramRenderer, DiagramResult};

pub const FENCE_OPEN_LEN: usize = 3;
pub const FENCE_CLOSE_LEN: usize = 4;

pub struct MarkdownFenceOps;

impl MarkdownFenceOps {
    pub fn extract_fence_block(s: &str) -> Option<(FenceBlock, &str)> {
        let body = s.strip_prefix("```")?;
        let info_end = body.find('\n')?;
        let info = body[..info_end].trim().to_string();
        let after_info = &body[info_end + 1..];
        let close = after_info.find("\n```")?;
        let content = after_info[..close].to_string();
        let raw = format!("```{info}\n{content}\n```");
        let rest = after_info[close + FENCE_CLOSE_LEN..]
            .strip_prefix('\n')
            .unwrap_or(&after_info[close + FENCE_CLOSE_LEN..]);
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
        let Some((block, after)) = Self::extract_fence_block(remaining) else {
            output.push_str("```");
            *remaining = &remaining[FENCE_OPEN_LEN..];
            return;
        };
        if let Some(html) = Self::render_diagram_block(&block, renderer) {
            output.push('\n');
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
            let fence_offset = if remaining.starts_with("```") {
                Some(0)
            } else {
                remaining.find("\n```").map(|pos| pos + 1)
            };
            let Some(offset) = fence_offset else {
                break;
            };
            output.push_str(&remaining[..offset]);
            remaining = &remaining[offset..];
            Self::process_fence(&mut output, &mut remaining, renderer);
        }
        output.push_str(remaining);
        output
    }
}

#[cfg(test)]
mod tests;
