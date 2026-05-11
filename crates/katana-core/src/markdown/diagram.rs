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

    pub fn should_preserve_fenced_source(&self, source: &str) -> bool {
        matches!(self, Self::Mermaid) && should_preserve_mermaid_fence(source)
    }
}

fn should_preserve_mermaid_fence(source: &str) -> bool {
    let trimmed = source.trim_start_matches('\u{feff}').trim_start();
    if trimmed.trim().is_empty() {
        return true;
    }
    mermaid_source_starts_with_keyword(trimmed, "zenuml")
}

fn mermaid_source_starts_with_keyword(source: &str, keyword: &str) -> bool {
    let lower_source = source.to_ascii_lowercase();
    let Some(rest) = lower_source.strip_prefix(keyword) else {
        return false;
    };
    rest.chars().next().is_none_or(char::is_whitespace)
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
        self.render_with_theme(super::diagram_backend::DiagramThemeSnapshot::current())
    }

    pub fn render_with_theme(
        &self,
        theme: super::diagram_backend::DiagramThemeSnapshot,
    ) -> DiagramResult {
        let language = match self.kind {
            DiagramKind::Mermaid => super::diagram_backend::DiagramBackendLanguage::Mermaid,
            DiagramKind::PlantUml => super::diagram_backend::DiagramBackendLanguage::PlantUml,
            DiagramKind::DrawIo => super::diagram_backend::DiagramBackendLanguage::DrawIo,
        };
        let backend = super::diagram_backend::DiagramBackendFactory::create(language.clone());
        let input = super::diagram_backend::DiagramBackendInput {
            language,
            source: self.source.clone(),
            options: super::diagram_backend::DiagramRenderOptions::default(),
            theme,
            document: super::diagram_backend::DiagramDocumentContext::Detached {
                display_name: String::new(),
            },
        };
        match backend.render(&input) {
            Ok(output) => output.into_diagram_result(),
            Err(error) => error.into_diagram_result(&self.source),
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
