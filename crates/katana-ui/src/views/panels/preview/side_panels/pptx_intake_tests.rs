use super::{AppAction, PreviewSidePanels, count_fullscreen, egui, make_app};
use crate::app::action::ActionOps;

fn unready_pptx_app() -> crate::shell::KatanaApp {
    let mut app = make_app();
    app.state
        .document
        .open_documents
        .push(katana_core::document::Document::new_empty("unready.pptx"));
    app.state.document.active_doc_idx = Some(0);
    app
}

#[test]
fn unavailable_pptx_action_does_not_enter_blank_fullscreen() {
    let mut app = unready_pptx_app();
    let context = egui::Context::default();
    let mut output = context.run_ui(egui::RawInput::default(), |ui| {
        app.process_action(ui.ctx(), AppAction::ToggleSlideshow);
    });
    output.textures_delta.clear();
    let commands: Vec<_> = output
        .viewport_output
        .values()
        .flat_map(|viewport| viewport.commands.iter().cloned())
        .collect();
    assert!(!app.state.layout.show_slideshow);
    assert_eq!(count_fullscreen(&commands, true), 0);
}

#[test]
fn unavailable_pptx_sidebar_does_not_preserve_blank_slideshow() {
    let mut app = unready_pptx_app();
    app.state.layout.show_slideshow = true;
    let context = egui::Context::default();
    let mut output = context.run_ui(egui::RawInput::default(), |ui| {
        PreviewSidePanels::new(&mut app).show(ui);
    });
    output.textures_delta.clear();
    let commands: Vec<_> = output
        .viewport_output
        .values()
        .flat_map(|viewport| viewport.commands.iter().cloned())
        .collect();
    assert!(!app.state.layout.show_slideshow);
    assert_eq!(count_fullscreen(&commands, false), 1);
}

#[test]
fn unavailable_pptx_system_modal_does_not_preserve_blank_slideshow() {
    let mut app = unready_pptx_app();
    app.tab_previews.push(crate::shell::TabPreviewCache {
        path: "unready.pptx".into(),
        pane: crate::preview_pane::PreviewPane::default(),
        hash: 0,
    });
    app.state.layout.show_slideshow = true;
    let context = egui::Context::default();
    let mut output = context.run_ui(egui::RawInput::default(), |ui| {
        app.show_system_modals(ui.ctx());
    });
    output.textures_delta.clear();
    let commands: Vec<_> = output
        .viewport_output
        .values()
        .flat_map(|viewport| viewport.commands.iter().cloned())
        .collect();
    assert!(!app.state.layout.show_slideshow);
    assert_eq!(count_fullscreen(&commands, false), 1);
}
