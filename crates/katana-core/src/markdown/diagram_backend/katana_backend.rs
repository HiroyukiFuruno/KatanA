use std::sync::OnceLock;

use super::adapter::DiagramBackendAdapter;
use super::result::DiagramBackendRenderResult;
use super::types::{
    DiagramBackendId, DiagramBackendInput, DiagramBackendLanguage, DiagramBackendVersion,
};
use katana_markdown_model::DiagramKind as KdvDiagramKind;

mod kdv_runtime;

#[cfg(test)]
use self::kdv_runtime::{kdv_diagram_output, kdv_render_request_theme};

const KDV_CRATE_VERSION: &str = env!("KATANA_DOCUMENT_VIEWER_VERSION");
const KRR_CRATE_VERSION: &str = env!("KATANA_RENDER_RUNTIME_VERSION");
const KDV_MERMAID_PROFILE: &str = "katana-mermaid";
const KDV_DRAWIO_PROFILE: &str = "katana-drawio";
const KDV_PLANTUML_PROFILE: &str = "katana-plantuml";

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
        ID.get_or_init(|| DiagramBackendId::new(DiagramBackendLanguage::Mermaid, "kdv-krr-mermaid"))
    }

    fn version(&self) -> &DiagramBackendVersion {
        static VER: OnceLock<DiagramBackendVersion> = OnceLock::new();
        VER.get_or_init(mermaid_backend_version)
    }

    fn render(&self, input: &DiagramBackendInput) -> DiagramBackendRenderResult {
        kdv_runtime::render(KdvDiagramKind::Mermaid, input)
    }
}

/// KatanA-internal PlantUML backend.
struct KatanaPlantUmlBackend;

impl DiagramBackendAdapter for KatanaPlantUmlBackend {
    fn id(&self) -> &DiagramBackendId {
        static ID: OnceLock<DiagramBackendId> = OnceLock::new();
        ID.get_or_init(|| {
            DiagramBackendId::new(DiagramBackendLanguage::PlantUml, "kdv-krr-plantuml")
        })
    }

    fn version(&self) -> &DiagramBackendVersion {
        static VER: OnceLock<DiagramBackendVersion> = OnceLock::new();
        VER.get_or_init(plantuml_backend_version)
    }

    fn render(&self, input: &DiagramBackendInput) -> DiagramBackendRenderResult {
        kdv_runtime::render(KdvDiagramKind::PlantUml, input)
    }
}

/// KatanA-internal Draw.io backend.
struct KatanaDrawIoBackend;

impl DiagramBackendAdapter for KatanaDrawIoBackend {
    fn id(&self) -> &DiagramBackendId {
        static ID: OnceLock<DiagramBackendId> = OnceLock::new();
        ID.get_or_init(|| DiagramBackendId::new(DiagramBackendLanguage::DrawIo, "kdv-krr-drawio"))
    }

    fn version(&self) -> &DiagramBackendVersion {
        static VER: OnceLock<DiagramBackendVersion> = OnceLock::new();
        VER.get_or_init(drawio_backend_version)
    }

    fn render(&self, input: &DiagramBackendInput) -> DiagramBackendRenderResult {
        kdv_runtime::render(KdvDiagramKind::DrawIo, input)
    }
}

fn mermaid_backend_version() -> DiagramBackendVersion {
    DiagramBackendVersion::from_kdv_krr(
        KDV_CRATE_VERSION,
        KRR_CRATE_VERSION,
        "Mermaid.js",
        katana_render_runtime::markdown::mermaid_renderer::MERMAID_JS_VERSION,
        katana_render_runtime::markdown::mermaid_renderer::MERMAID_JS_CHECKSUM,
        KDV_MERMAID_PROFILE,
    )
}

fn drawio_backend_version() -> DiagramBackendVersion {
    DiagramBackendVersion::from_kdv_krr(
        KDV_CRATE_VERSION,
        KRR_CRATE_VERSION,
        "Draw.io",
        katana_render_runtime::markdown::drawio_renderer::DRAWIO_JS_VERSION,
        katana_render_runtime::markdown::drawio_renderer::DRAWIO_JS_CHECKSUM,
        KDV_DRAWIO_PROFILE,
    )
}

fn plantuml_backend_version() -> DiagramBackendVersion {
    DiagramBackendVersion::from_kdv_krr(
        KDV_CRATE_VERSION,
        KRR_CRATE_VERSION,
        "PlantUML",
        katana_render_runtime::markdown::plantuml_renderer::PLANTUML_JAR_VERSION,
        katana_render_runtime::markdown::plantuml_renderer::PLANTUML_JAR_CHECKSUM,
        KDV_PLANTUML_PROFILE,
    )
}

#[cfg(test)]
mod tests;
