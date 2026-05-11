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

    let input = DiagramBackendInput {
        language: backend_language,
        source: block.source.clone(),
        options: DiagramRenderOptions::default(),
        theme: DiagramThemeSnapshot::current(),
        document: DiagramDocumentContext::Detached {
            display_name: String::new(),
        },
    };

    match backend.render(&input) {
        Ok(output) => output.into_diagram_result(),
        Err(error) => error.into_diagram_result(&block.source),
    }
}
