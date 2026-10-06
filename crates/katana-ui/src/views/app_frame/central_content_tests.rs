use super::CentralContent;
use crate::app_state::AppState;
use crate::preview_pane::{DocumentSurfaceSource, PreviewPane};
use crate::shell::{KatanaApp, TabPreviewCache};
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};

#[test]
fn slideshow_skips_background_central_content() {
    let temp_dir = tempfile::tempdir().expect("temporary PPTX directory");
    let document_path = temp_dir.path().join("slideshow.pptx");
    std::fs::write(
        &document_path,
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../scripts/screenshot/fixtures/v0-22-38-multi-format/representative.pptx"
        )),
    )
    .expect("write PPTX fixture");
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state
        .document
        .open_documents
        .push(katana_core::document::Document::new(
            document_path.clone(),
            String::new(),
        ));
    state.document.active_doc_idx = Some(0);
    state.layout.show_slideshow = true;
    let mut app = KatanaApp::new(state);
    let source = DocumentSurfaceSource::local(&document_path).expect("PPTX source");
    let mut pane = PreviewPane::default();
    pane.full_render_document_source(source, true);
    app.tab_previews.push(TabPreviewCache {
        path: document_path.clone(),
        pane,
        hash: 0,
    });
    let ctx = eframe::egui::Context::default();

    let mut output = ctx.run_ui(eframe::egui::RawInput::default(), |ui| {
        eframe::egui::CentralPanel::default().show(ui, |ui| {
            CentralContent::new(&mut app).show(ui);
        });
    });
    output.textures_delta.clear();

    assert!(app.state.layout.show_slideshow);
    assert!(app.tab_previews[0].pane.repaint_ctx.is_none());
}

#[test]
fn markdown_slideshow_keeps_background_preview_polling() {
    let document_path = std::path::PathBuf::from("slideshow.md");
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state
        .document
        .open_documents
        .push(katana_core::document::Document::new_empty(
            document_path.clone(),
        ));
    state.document.active_doc_idx = Some(0);
    state.layout.show_slideshow = true;
    let mut app = KatanaApp::new(state);
    app.tab_previews.push(TabPreviewCache {
        path: document_path,
        pane: PreviewPane::default(),
        hash: 0,
    });
    let ctx = eframe::egui::Context::default();

    let mut output = ctx.run_ui(eframe::egui::RawInput::default(), |ui| {
        eframe::egui::CentralPanel::default().show(ui, |ui| {
            CentralContent::new(&mut app).show(ui);
        });
    });
    output.textures_delta.clear();

    assert!(app.tab_previews[0].pane.repaint_ctx.is_some());
}
