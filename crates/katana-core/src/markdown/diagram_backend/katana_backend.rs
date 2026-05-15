//! KatanA diagram backend adapter implementations.

use std::sync::OnceLock;

use self::error_mapping::{diagram_result_to_backend, kdr_error_to_backend};
use super::adapter::DiagramBackendAdapter;
use super::kdr_theme_adapter::KdrThemeAdapter;
use super::result::{DiagramBackendOutput, DiagramBackendRenderResult};
use super::types::{
    DiagramBackendId, DiagramBackendInput, DiagramBackendLanguage, DiagramBackendVersion,
};
use crate::markdown::{DiagramBlock, DiagramKind, plantuml_renderer};
use katana_diagram_renderer::{
    DrawioRenderer, MermaidRenderer, RenderConfig, RenderContext, RenderInput, RenderPolicy,
    Renderer, RuntimePathResolver,
};

mod error_mapping;

const KATANA_BACKEND_VERSION: &str = env!("CARGO_PKG_VERSION");
const KDR_CRATE_VERSION: &str = env!("KATANA_DIAGRAM_RENDERER_VERSION");
const KDR_MERMAID_PROFILE: &str = "katana-mermaid";
const KDR_DRAWIO_PROFILE: &str = "katana-drawio";

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
        ID.get_or_init(|| DiagramBackendId::new(DiagramBackendLanguage::Mermaid, "kdr-mermaid"))
    }

    fn version(&self) -> &DiagramBackendVersion {
        static VER: OnceLock<DiagramBackendVersion> = OnceLock::new();
        VER.get_or_init(mermaid_backend_version)
    }

    fn render(&self, input: &DiagramBackendInput) -> DiagramBackendRenderResult {
        let runtime_path =
            RuntimePathResolver::resolve(katana_diagram_renderer::DiagramKind::Mermaid, None)
                .map_err(kdr_error_to_backend)?;
        let renderer = MermaidRenderer::with_runtime_path(runtime_path);
        let output = renderer
            .render(&kdr_input(
                katana_diagram_renderer::DiagramKind::Mermaid,
                input,
            ))
            .map_err(kdr_error_to_backend)?;
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
        ID.get_or_init(|| DiagramBackendId::new(DiagramBackendLanguage::DrawIo, "kdr-drawio"))
    }

    fn version(&self) -> &DiagramBackendVersion {
        static VER: OnceLock<DiagramBackendVersion> = OnceLock::new();
        VER.get_or_init(drawio_backend_version)
    }

    fn render(&self, input: &DiagramBackendInput) -> DiagramBackendRenderResult {
        let runtime_path =
            RuntimePathResolver::resolve(katana_diagram_renderer::DiagramKind::Drawio, None)
                .map_err(kdr_error_to_backend)?;
        let renderer = DrawioRenderer::with_runtime_path(runtime_path);
        let output = renderer
            .render(&kdr_input(
                katana_diagram_renderer::DiagramKind::Drawio,
                input,
            ))
            .map_err(kdr_error_to_backend)?;
        Ok(DiagramBackendOutput::HtmlFragment(output.svg))
    }
}

fn kdr_input(
    kind: katana_diagram_renderer::DiagramKind,
    input: &DiagramBackendInput,
) -> RenderInput {
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
            theme: Some(KdrThemeAdapter::convert(&input.theme)),
        },
    }
}

fn mermaid_backend_version() -> DiagramBackendVersion {
    DiagramBackendVersion::from_kdr(
        KDR_CRATE_VERSION,
        "Mermaid.js",
        katana_diagram_renderer::markdown::mermaid_renderer::MERMAID_JS_VERSION,
        katana_diagram_renderer::markdown::mermaid_renderer::MERMAID_JS_CHECKSUM,
        KDR_MERMAID_PROFILE,
    )
}

fn drawio_backend_version() -> DiagramBackendVersion {
    DiagramBackendVersion::from_kdr(
        KDR_CRATE_VERSION,
        "Draw.io",
        katana_diagram_renderer::markdown::drawio_renderer::DRAWIO_JS_VERSION,
        katana_diagram_renderer::markdown::drawio_renderer::DRAWIO_JS_CHECKSUM,
        KDR_DRAWIO_PROFILE,
    )
}

#[cfg(test)]
mod tests;
