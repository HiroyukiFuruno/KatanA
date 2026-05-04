use katana_core::markdown::color_preset::DiagramColorPreset;
use katana_core::markdown::{
    DiagramBackendAdapter, DiagramBackendInput, DiagramBackendLanguage, DiagramBlock,
    DiagramDocumentContext, DiagramKind, DiagramRenderOptions, DiagramResult, DiagramThemeSnapshot,
    KatanaDrawIoBackend, KatanaMermaidBackend, KatanaPlantUmlBackend,
};

pub(super) fn dispatch_renderer(block: &DiagramBlock) -> DiagramResult {
    let backend: Box<dyn DiagramBackendAdapter> = match block.kind {
        DiagramKind::Mermaid => Box::new(KatanaMermaidBackend),
        DiagramKind::PlantUml => Box::new(KatanaPlantUmlBackend),
        DiagramKind::DrawIo => Box::new(KatanaDrawIoBackend),
    };

    let preset = DiagramColorPreset::current();
    let is_dark = DiagramColorPreset::is_dark_mode();
    let input = DiagramBackendInput {
        language: match block.kind {
            DiagramKind::Mermaid => DiagramBackendLanguage::Mermaid,
            DiagramKind::PlantUml => DiagramBackendLanguage::PlantUml,
            DiagramKind::DrawIo => DiagramBackendLanguage::DrawIo,
        },
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
