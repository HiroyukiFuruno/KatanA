use katana_core::markdown::color_preset::DiagramColorPreset;
use katana_core::markdown::{
    DiagramBackendAdapter, DiagramBackendFactory, DiagramBackendInput, DiagramBackendLanguage,
    DiagramBlock, DiagramDocumentContext, DiagramKind, DiagramRenderOptions, DiagramResult,
    DiagramThemeSnapshot,
};

pub(super) fn dispatch_renderer(block: &DiagramBlock) -> DiagramResult {
    let backend_language = match block.kind {
        DiagramKind::Mermaid => DiagramBackendLanguage::Mermaid,
        DiagramKind::PlantUml => DiagramBackendLanguage::PlantUml,
        DiagramKind::DrawIo => DiagramBackendLanguage::DrawIo,
    };
    let backend: Box<dyn DiagramBackendAdapter> =
        DiagramBackendFactory::create(backend_language.clone());

    let preset = DiagramColorPreset::current();
    let is_dark = DiagramColorPreset::is_dark_mode();
    let input = DiagramBackendInput {
        language: backend_language,
        source: block.source.clone(),
        options: DiagramRenderOptions::default(),
        theme: DiagramThemeSnapshot::from_preset(
            if is_dark { "dark" } else { "light" },
            is_dark,
            preset,
        ),
        document: DiagramDocumentContext::Detached {
            display_name: String::new(),
        },
    };

    match backend.render(&input) {
        Ok(output) => output.into_diagram_result(),
        Err(error) => error.into_diagram_result(&block.source),
    }
}
