//! KatanA-internal `DiagramBackendAdapter` implementations.
//!
//! These thin wrappers delegate to the existing `mermaid_renderer`,
//! `plantuml_renderer`, and `drawio_renderer` modules.  At kcf intake
//! (v0.26.0) the call sites in `katana-ui` can simply swap in kcf's
//! `impl DiagramBackendAdapter` without touching any other code.

use std::sync::OnceLock;

use super::adapter::DiagramBackendAdapter;
use super::result::{DiagramBackendError, DiagramBackendOutput, DiagramBackendRenderResult};
use super::types::{
    DiagramBackendId, DiagramBackendInput, DiagramBackendLanguage, DiagramBackendVersion,
};
use crate::markdown::{DiagramBlock, DiagramKind, DiagramResult};
use crate::markdown::{drawio_renderer, mermaid_renderer, plantuml_renderer};

const KATANA_BACKEND_VERSION: &str = env!("CARGO_PKG_VERSION");

fn diagram_result_to_backend(result: DiagramResult) -> DiagramBackendRenderResult {
    match result {
        DiagramResult::Ok(html) => Ok(DiagramBackendOutput::HtmlFragment(html)),
        DiagramResult::OkPng(bytes) => Ok(DiagramBackendOutput::Png(bytes)),
        DiagramResult::Err { error, .. } => {
            Err(DiagramBackendError::RenderFailed { message: error })
        }
        DiagramResult::CommandNotFound {
            tool_name,
            install_hint,
            ..
        } => Err(DiagramBackendError::CommandNotFound {
            tool_name,
            install_hint,
        }),
        DiagramResult::NotInstalled {
            kind,
            download_url,
            install_path,
        } => Err(DiagramBackendError::NotInstalled {
            kind,
            download_url,
            install_path,
        }),
    }
}

/// Factory for KatanA-internal diagram backend adapters.
pub struct DiagramBackendFactory;

impl DiagramBackendFactory {
    pub fn create(language: DiagramBackendLanguage) -> Box<dyn DiagramBackendAdapter> {
        match language {
            DiagramBackendLanguage::Mermaid => Box::new(KatanaMermaidBackend),
            DiagramBackendLanguage::PlantUml => Box::new(KatanaPlantUmlBackend),
            DiagramBackendLanguage::DrawIo => Box::new(KatanaDrawIoBackend),
        }
    }
}

/// KatanA-internal Mermaid backend.
struct KatanaMermaidBackend;

impl DiagramBackendAdapter for KatanaMermaidBackend {
    fn id(&self) -> &DiagramBackendId {
        static ID: OnceLock<DiagramBackendId> = OnceLock::new();
        ID.get_or_init(|| DiagramBackendId::new(DiagramBackendLanguage::Mermaid, "katana-mermaid"))
    }

    fn version(&self) -> &DiagramBackendVersion {
        static VER: OnceLock<DiagramBackendVersion> = OnceLock::new();
        VER.get_or_init(|| DiagramBackendVersion::new(KATANA_BACKEND_VERSION))
    }

    fn render(&self, input: &DiagramBackendInput) -> DiagramBackendRenderResult {
        let block = DiagramBlock {
            kind: DiagramKind::Mermaid,
            source: input.source.clone(),
        };
        diagram_result_to_backend(mermaid_renderer::MermaidRenderOps::render_mermaid(&block))
    }
}

/// KatanA-internal PlantUML backend.
struct KatanaPlantUmlBackend;

impl DiagramBackendAdapter for KatanaPlantUmlBackend {
    fn id(&self) -> &DiagramBackendId {
        static ID: OnceLock<DiagramBackendId> = OnceLock::new();
        ID.get_or_init(|| {
            DiagramBackendId::new(DiagramBackendLanguage::PlantUml, "katana-plantuml")
        })
    }

    fn version(&self) -> &DiagramBackendVersion {
        static VER: OnceLock<DiagramBackendVersion> = OnceLock::new();
        VER.get_or_init(|| DiagramBackendVersion::new(KATANA_BACKEND_VERSION))
    }

    fn render(&self, input: &DiagramBackendInput) -> DiagramBackendRenderResult {
        let block = DiagramBlock {
            kind: DiagramKind::PlantUml,
            source: input.source.clone(),
        };
        diagram_result_to_backend(plantuml_renderer::PlantUmlRendererOps::render_plantuml(
            &block,
        ))
    }
}

/// KatanA-internal Draw.io backend.
struct KatanaDrawIoBackend;

impl DiagramBackendAdapter for KatanaDrawIoBackend {
    fn id(&self) -> &DiagramBackendId {
        static ID: OnceLock<DiagramBackendId> = OnceLock::new();
        ID.get_or_init(|| DiagramBackendId::new(DiagramBackendLanguage::DrawIo, "katana-drawio"))
    }

    fn version(&self) -> &DiagramBackendVersion {
        static VER: OnceLock<DiagramBackendVersion> = OnceLock::new();
        VER.get_or_init(|| DiagramBackendVersion::new(KATANA_BACKEND_VERSION))
    }

    fn render(&self, input: &DiagramBackendInput) -> DiagramBackendRenderResult {
        let block = DiagramBlock {
            kind: DiagramKind::DrawIo,
            source: input.source.clone(),
        };
        diagram_result_to_backend(drawio_renderer::DrawioRendererOps::render_drawio(&block))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_png_result_maps_to_png_output() {
        let bytes = vec![1u8, 2, 3];
        let result = diagram_result_to_backend(DiagramResult::OkPng(bytes.clone()));
        assert_eq!(result, Ok(DiagramBackendOutput::Png(bytes)));
    }

    #[test]
    fn err_result_maps_to_render_failed() {
        let result = diagram_result_to_backend(DiagramResult::Err {
            source: "src".to_string(),
            error: "oops".to_string(),
        });
        assert_eq!(
            result,
            Err(DiagramBackendError::RenderFailed {
                message: "oops".to_string()
            })
        );
    }

    #[test]
    fn command_not_found_result_maps_to_backend_command_not_found() {
        let result = diagram_result_to_backend(DiagramResult::CommandNotFound {
            tool_name: "plantuml".to_string(),
            install_hint: "brew install plantuml".to_string(),
            source: "src".to_string(),
        });
        assert_eq!(
            result,
            Err(DiagramBackendError::CommandNotFound {
                tool_name: "plantuml".to_string(),
                install_hint: "brew install plantuml".to_string(),
            })
        );
    }

    #[test]
    fn mermaid_backend_id_has_correct_language_and_implementation() {
        let backend = KatanaMermaidBackend;
        let id = backend.id();
        assert_eq!(id.language, DiagramBackendLanguage::Mermaid);
        assert_eq!(id.implementation, "katana-mermaid");
    }

    #[test]
    fn mermaid_backend_version_is_non_empty() {
        assert!(!KatanaMermaidBackend.version().value.is_empty());
    }

    #[test]
    fn plantuml_backend_id_has_correct_language_and_implementation() {
        let backend = KatanaPlantUmlBackend;
        let id = backend.id();
        assert_eq!(id.language, DiagramBackendLanguage::PlantUml);
        assert_eq!(id.implementation, "katana-plantuml");
    }

    #[test]
    fn plantuml_backend_version_is_non_empty() {
        assert!(!KatanaPlantUmlBackend.version().value.is_empty());
    }

    #[test]
    fn drawio_backend_id_has_correct_language_and_implementation() {
        let backend = KatanaDrawIoBackend;
        let id = backend.id();
        assert_eq!(id.language, DiagramBackendLanguage::DrawIo);
        assert_eq!(id.implementation, "katana-drawio");
    }

    #[test]
    fn drawio_backend_version_is_non_empty() {
        assert!(!KatanaDrawIoBackend.version().value.is_empty());
    }
}
