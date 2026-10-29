use super::{navigation_command, navigation_response};
use crate::app::ActionOps;
use crate::app_state::{AppAction, AppState};
use crate::shell::KatanaApp;
use eframe::egui;
use katana_core::{ai::AiProviderRegistry, document::Document, plugin::PluginRegistry};
use std::sync::Arc;

const PAGE_COUNT: usize = 3;
const LOADING_FOCUS_FRAMES: usize = 2;

fn app_with_documents(path: &std::path::Path) -> KatanaApp {
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        Default::default(),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state
        .document
        .open_documents
        .push(Document::new_empty("/tmp/keyboard-focus.md"));
    state
        .document
        .open_documents
        .push(Document::new_empty(path));
    state.document.active_doc_idx = Some(0);
    KatanaApp::new(state)
}

fn selected_pdf(
    ctx: &egui::Context,
    app: &mut KatanaApp,
    path: &std::path::Path,
) -> std::path::PathBuf {
    app.process_action(ctx, AppAction::SelectDocument(path.to_path_buf()));
    path.to_path_buf()
}

pub(super) fn page_input(events: Vec<egui::Event>) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(100.0, 100.0),
        )),
        focused: true,
        events,
        ..Default::default()
    }
}

pub(super) fn selected_pdf_app() -> (
    tempfile::TempDir,
    egui::Context,
    KatanaApp,
    std::path::PathBuf,
) {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("deck.pdf");
    std::fs::write(&path, b"%PDF-1.7\n").expect("write fixture");
    let ctx = egui::Context::default();
    let mut app = app_with_documents(&path);
    let uri = selected_pdf(&ctx, &mut app, &path);
    (directory, ctx, app, uri)
}

#[test]
fn delayed_page_activation_survives_previous_widget_focus_expiring() {
    let (_directory, ctx, mut app, owner_path) = selected_pdf_app();
    assert!(activation_focus(&ctx, &owner_path));
    selected_pdf(&ctx, &mut app, &owner_path);
    for _ in 0..LOADING_FOCUS_FRAMES {
        let output = ctx.run_ui(page_input(Vec::new()), |ui| {
            ui.spinner();
        });
        output.drop_without_applying_deltas();
    }
    assert_eq!(ctx.memory(|memory| memory.focused()), None);
    assert!(activation_focus(&ctx, &owner_path));
    assert_eq!(
        next_page_command(&ctx),
        Some(katana_document_viewer::DocumentViewerCommand::Next)
    );
}

fn activation_focus(ctx: &egui::Context, owner_path: &std::path::Path) -> bool {
    let mut focused = false;
    let mut output = ctx.run_ui(page_input(Vec::new()), |ui| {
        let response = navigation_response(ui, 0);
        focused = crate::app::preview::keyboard_focus::PageNavigationFocusOps::focus_if_activated(
            ui.ctx(),
            &response,
            owner_path,
        ) == Some(true)
            && response.has_focus();
    });
    output.textures_delta.clear();
    drop(output);
    focused
}

fn next_page_command(ctx: &egui::Context) -> Option<katana_document_viewer::DocumentViewerCommand> {
    let mut command = None;
    let input = page_input(vec![egui::Event::Key {
        key: egui::Key::ArrowRight,
        physical_key: Some(egui::Key::ArrowRight),
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    }]);
    let mut output = ctx.run_ui(input, |ui| {
        let response = navigation_response(ui, 0);
        command = navigation_command(
            ui,
            &response,
            &super::keyboard_tests::frame(
                katana_document_viewer::ViewerDocumentFormat::Pdf,
                0,
                PAGE_COUNT,
            ),
        );
    });
    output.textures_delta.clear();
    drop(output);
    command
}

fn render_text_input(ctx: &egui::Context, text: &mut String, id: egui::Id) {
    let mut output = ctx.run_ui(page_input(Vec::new()), |ui| {
        ui.add(egui::TextEdit::singleline(text).id(id))
            .request_focus();
    });
    output.textures_delta.clear();
    drop(output);
}

#[test]
fn explicit_document_selection_focuses_real_page_surface_then_routes_one_arrow() {
    let (_directory, ctx, _app, owner_path) = selected_pdf_app();
    assert!(activation_focus(&ctx, &owner_path));
    assert_eq!(
        next_page_command(&ctx),
        Some(katana_document_viewer::DocumentViewerCommand::Next)
    );
}

#[test]
fn async_selection_does_not_take_focus_after_user_focuses_text_input() {
    let (_directory, ctx, _app, owner_path) = selected_pdf_app();
    let editor_id = egui::Id::new("keyboard-focus-editor");
    let mut text = String::new();
    render_text_input(&ctx, &mut text, editor_id);

    let mut activation_result = None;
    let mut output = ctx.run_ui(page_input(Vec::new()), |ui| {
        ui.add(egui::TextEdit::singleline(&mut text).id(editor_id));
        let response = navigation_response(ui, 0);
        activation_result =
            crate::app::preview::keyboard_focus::PageNavigationFocusOps::focus_if_activated(
                ui.ctx(),
                &response,
                &owner_path,
            );
    });
    output.textures_delta.clear();
    drop(output);
    assert_eq!(activation_result, Some(false));
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(editor_id));
}
