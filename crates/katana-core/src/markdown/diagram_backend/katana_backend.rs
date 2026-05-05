//! KatanA diagram backend adapter implementations.

use std::sync::OnceLock;

use super::adapter::DiagramBackendAdapter;
use super::result::{DiagramBackendError, DiagramBackendOutput, DiagramBackendRenderResult};
use super::types::{
    DiagramBackendId, DiagramBackendInput, DiagramBackendLanguage, DiagramBackendVersion,
};
use crate::markdown::{DiagramBlock, DiagramKind, DiagramResult, plantuml_renderer};
use katana_canvas_forge::{
    DrawioRenderer, MermaidRenderer, RenderConfig, RenderContext, RenderError, RenderInput,
    RenderPolicy, Renderer, RuntimePathResolver,
};

const KATANA_BACKEND_VERSION: &str = env!("CARGO_PKG_VERSION");
const KCF_MERMAID_BACKEND_VERSION: &str =
    "crate=katana-canvas-forge:0.1.0;runtime=Mermaid.js:3.3.1;profile=kcf-mermaid";
const KCF_DRAWIO_BACKEND_VERSION: &str =
    "crate=katana-canvas-forge:0.1.0;runtime=Draw.io:29.7.10;profile=kcf-drawio";

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

fn kcf_error_to_backend(error: RenderError) -> DiagramBackendError {
    match error {
        RenderError::NotInstalled {
            kind,
            download_url,
            install_path,
        } => DiagramBackendError::NotInstalled {
            kind,
            download_url,
            install_path,
        },
        RenderError::InvalidInput(message)
        | RenderError::Runtime(message)
        | RenderError::RuntimeResolution(message) => DiagramBackendError::RenderFailed { message },
        RenderError::UnsupportedKind => DiagramBackendError::RenderFailed {
            message: "unsupported diagram kind".to_string(),
        },
    }
}

pub(super) fn create_backend(language: DiagramBackendLanguage) -> Box<dyn DiagramBackendAdapter> {
    match language {
        DiagramBackendLanguage::Mermaid => Box::new(KatanaMermaidBackend),
        DiagramBackendLanguage::PlantUml => Box::new(KatanaPlantUmlBackend),
        DiagramBackendLanguage::DrawIo => Box::new(KatanaDrawIoBackend),
    }
}

/// KatanA-internal Mermaid backend.
struct KatanaMermaidBackend;

impl DiagramBackendAdapter for KatanaMermaidBackend {
    fn id(&self) -> &DiagramBackendId {
        static ID: OnceLock<DiagramBackendId> = OnceLock::new();
        ID.get_or_init(|| DiagramBackendId::new(DiagramBackendLanguage::Mermaid, "kcf-mermaid"))
    }

    fn version(&self) -> &DiagramBackendVersion {
        static VER: OnceLock<DiagramBackendVersion> = OnceLock::new();
        VER.get_or_init(|| DiagramBackendVersion::new(KCF_MERMAID_BACKEND_VERSION))
    }

    fn render(&self, input: &DiagramBackendInput) -> DiagramBackendRenderResult {
        let runtime_path =
            RuntimePathResolver::resolve(katana_canvas_forge::DiagramKind::Mermaid, None)
                .map_err(kcf_error_to_backend)?;
        let renderer = MermaidRenderer::with_runtime_path(runtime_path);
        let output = renderer
            .render(&kcf_input(katana_canvas_forge::DiagramKind::Mermaid, input))
            .map_err(kcf_error_to_backend)?;
        Ok(DiagramBackendOutput::HtmlFragment(output.svg))
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
        ID.get_or_init(|| DiagramBackendId::new(DiagramBackendLanguage::DrawIo, "kcf-drawio"))
    }

    fn version(&self) -> &DiagramBackendVersion {
        static VER: OnceLock<DiagramBackendVersion> = OnceLock::new();
        VER.get_or_init(|| DiagramBackendVersion::new(KCF_DRAWIO_BACKEND_VERSION))
    }

    fn render(&self, input: &DiagramBackendInput) -> DiagramBackendRenderResult {
        let runtime_path =
            RuntimePathResolver::resolve(katana_canvas_forge::DiagramKind::Drawio, None)
                .map_err(kcf_error_to_backend)?;
        let renderer = DrawioRenderer::with_runtime_path(runtime_path);
        let output = renderer
            .render(&kcf_input(katana_canvas_forge::DiagramKind::Drawio, input))
            .map_err(kcf_error_to_backend)?;
        Ok(DiagramBackendOutput::HtmlFragment(output.svg))
    }
}

fn kcf_input(kind: katana_canvas_forge::DiagramKind, input: &DiagramBackendInput) -> RenderInput {
    RenderInput {
        kind,
        source: input.source.clone(),
        config: RenderConfig {
            vendor_config: serde_json::json!({
                "outputFormat": format!("{:?}", input.options.output_format),
                "timeoutMillis": input.options.timeout_millis,
                "scalePercent": input.options.scale_percent,
            }),
        },
        policy: RenderPolicy {
            background: Some(input.theme.background.clone()),
            cache_profile: Some(input.theme.name.clone()),
            ..RenderPolicy::default()
        },
        context: RenderContext {
            theme_fingerprint: Some(input.theme.fingerprint()),
            document_id: Some(input.document.cache_id()),
        },
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
        assert_eq!(id.implementation, "kcf-mermaid");
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
        assert_eq!(id.implementation, "kcf-drawio");
    }

    #[test]
    fn drawio_backend_version_is_non_empty() {
        assert!(!KatanaDrawIoBackend.version().value.is_empty());
    }
}
