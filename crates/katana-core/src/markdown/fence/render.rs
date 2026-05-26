use super::{FenceBlock, MarkdownFenceOps};
use crate::markdown::{DiagramBlock, DiagramKind, DiagramRenderer, DiagramResult};

const ZENUML_BROWSER_START_ERROR: &str = "Failed to start ZenUML browser renderer:";
const PLAYWRIGHT_PACKAGE_ERROR: &str = "Cannot find package 'playwright'";
const PLAYWRIGHT_LOOKUP_ERROR: &str = "Command failed: which playwright";
const PLAYWRIGHT_EXECUTABLE_ERROR: &str = "Executable doesn't exist";

impl MarkdownFenceOps {
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
        if kind.should_preserve_fenced_source(&block.content) {
            return None;
        }
        let diagram = DiagramBlock {
            kind: kind.clone(),
            source: block.content.clone(),
        };
        Some(match renderer.render(&diagram) {
            DiagramResult::Ok(html) => html,
            DiagramResult::OkPng(bytes) => Self::png_html(&bytes),
            DiagramResult::Err { source, error } => {
                if Self::should_preserve_zenuml_dependency_failure(kind, &source, &error) {
                    return None;
                }
                Self::fallback_html(&source, &error)
            }
            DiagramResult::CommandNotFound {
                tool_name,
                install_hint,
                ..
            } => Self::fallback_html("", &format!("{tool_name} not found. {install_hint}")),
            DiagramResult::NotInstalled { kind, message } => {
                Self::fallback_html("", &format!("{kind} is not available. {message}"))
            }
        })
    }

    fn png_html(bytes: &[u8]) -> String {
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
        format!(
            r#"<div class="katana-diagram mermaid"><img src="data:image/png;base64,{b64}" style="max-width:100%" /></div>"#
        )
    }

    fn should_preserve_zenuml_dependency_failure(
        kind: DiagramKind,
        source: &str,
        error: &str,
    ) -> bool {
        kind.is_zenuml_source(source) && Self::is_zenuml_dependency_error(error)
    }

    fn is_zenuml_dependency_error(error: &str) -> bool {
        [
            ZENUML_BROWSER_START_ERROR,
            PLAYWRIGHT_PACKAGE_ERROR,
            PLAYWRIGHT_LOOKUP_ERROR,
            PLAYWRIGHT_EXECUTABLE_ERROR,
        ]
        .iter()
        .any(|marker| error.contains(marker))
    }
}
