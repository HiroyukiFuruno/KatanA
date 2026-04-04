pub use super::types::*;
use base64::Engine;

impl DiagramKind {
    pub fn from_info(info: &str) -> Option<Self> {
        match info.trim().to_ascii_lowercase().as_str() {
            "mermaid" => Some(Self::Mermaid),
            "plantuml" => Some(Self::PlantUml),
            "drawio" => Some(Self::DrawIo),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Mermaid => "Mermaid",
            Self::PlantUml => "PlantUML",
            Self::DrawIo => "Draw.io",
        }
    }
}

impl DiagramBlock {
    pub fn validate(&self) -> Result<(), DiagramValidationError> {
        match self.kind {
            DiagramKind::Mermaid => {
                if self.source.trim().is_empty() {
                    return Err(DiagramValidationError::EmptySource {
                        kind: self.kind.display_name(),
                    });
                }
            }
            DiagramKind::PlantUml => {
                let src = self.source.trim();
                if !src.contains("@startuml") || !src.contains("@enduml") {
                    return Err(DiagramValidationError::MissingDelimiters {
                        kind: "plantuml",
                        message: "PlantUML blocks must contain @startuml and @enduml".to_string(),
                    });
                }
            }
            DiagramKind::DrawIo => {
                let src = self.source.trim();
                if !src.contains("<mxfile") && !src.contains("<mxGraphModel") {
                    return Err(DiagramValidationError::UnsupportedEncoding {
                        kind: "drawio",
                        message: "Draw.io blocks must contain raw uncompressed XML with <mxfile> or <mxGraphModel>".to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    pub fn render(&self) -> DiagramResult {
        match self.kind {
            DiagramKind::Mermaid => super::mermaid_renderer::MermaidRenderOps::render_mermaid(self),
            DiagramKind::PlantUml => {
                super::plantuml_renderer::PlantUmlRendererOps::render_plantuml(self)
            }
            DiagramKind::DrawIo => super::drawio_renderer::DrawioRendererOps::render_drawio(self),
        }
    }
}

impl DiagramResult {
    pub fn to_html(&self) -> String {
        match self {
            Self::Ok(html) => html.clone(),
            Self::OkPng(bytes) => {
                let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
                format!(r#"<img src="data:image/png;base64,{b64}" alt="Diagram">"#)
            }
            Self::Err { source, error } => format!(
                r#"<div class="katana-diagram-error"><pre><code>{}</code></pre><p>{}</p></div>"#,
                html_escape(source),
                html_escape(error)
            ),
            Self::CommandNotFound {
                tool_name,
                install_hint,
                source,
            } => format!(
                r#"<div class="katana-diagram-error"><p>{} not found.</p><p>Please install with: <code>{}</code></p><pre><code>{}</code></pre></div>"#,
                html_escape(tool_name),
                html_escape(install_hint),
                html_escape(source)
            ),
            Self::NotInstalled {
                kind,
                download_url,
                install_path,
            } => format!(
                r#"<div class="katana-diagram-error"><p>{} rendering engine is not installed.</p><p>Download from: <a href="{}">{}</a></p><p>Install to: <code>{}</code></p></div>"#,
                html_escape(kind),
                html_escape(download_url),
                html_escape(download_url),
                html_escape(&install_path.to_string_lossy())
            ),
        }
    }
}

impl DiagramRenderer for NoOpRenderer {
    fn render(&self, block: &DiagramBlock) -> DiagramResult {
        let html = format!(
            "<pre><code class=\"language-{kind}\">{source}</code></pre>",
            kind = match block.kind {
                DiagramKind::Mermaid => "mermaid",
                DiagramKind::PlantUml => "plantuml",
                DiagramKind::DrawIo => "drawio",
            },
            source = html_escape(&block.source),
        );
        DiagramResult::Ok(html)
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests;
