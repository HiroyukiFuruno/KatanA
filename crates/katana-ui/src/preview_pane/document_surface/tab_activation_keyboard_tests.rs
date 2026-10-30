use super::activation_keyboard_tests::{
    activation_focus, app_with_documents, next_page_command, page_input, render_text_input,
};
use crate::app::ActionOps;
use crate::app_state::AppAction;
use eframe::egui;
use katana_document_viewer::{DocumentViewerCommand, ViewerDocumentFormat};
const LOADING_FOCUS_FRAMES: usize = 2;

fn assert_tab_transition_arms_page_focus(
    extension: &str,
    format: ViewerDocumentFormat,
    action: AppAction,
) {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join(format!("deck.{extension}"));
    std::fs::write(&path, b"fixture").expect("write fixture");
    let ctx = egui::Context::default();
    let mut app = app_with_documents(&path);
    let editor_id = egui::Id::new("tab-activation-previous-editor");
    let mut text = String::new();
    render_text_input(&ctx, &mut text, editor_id);
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(editor_id));

    app.process_action(&ctx, action);
    assert_eq!(app.state.active_path(), Some(path.clone()));

    for _ in 0..LOADING_FOCUS_FRAMES {
        let output = ctx.run_ui(page_input(Vec::new()), |ui| {
            ui.spinner();
        });
        output.drop_without_applying_deltas();
    }
    assert_eq!(ctx.memory(|memory| memory.focused()), None);
    assert!(activation_focus(&ctx, &path));
    assert_eq!(
        next_page_command(&ctx, format),
        Some(DocumentViewerCommand::Next)
    );
}

#[test]
fn next_and_previous_tab_actions_focus_pdf_and_pptx_page_surfaces() {
    for (extension, format) in [
        ("pdf", ViewerDocumentFormat::Pdf),
        ("pptx", ViewerDocumentFormat::Pptx),
    ] {
        assert_tab_transition_arms_page_focus(extension, format, AppAction::SelectNextTab);
        assert_tab_transition_arms_page_focus(extension, format, AppAction::SelectPrevTab);
    }
}

#[test]
fn one_tab_next_action_does_not_arm_page_focus() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("deck.pdf");
    std::fs::write(&path, b"fixture").expect("write fixture");
    for action in [AppAction::SelectNextTab, AppAction::SelectPrevTab] {
        let ctx = egui::Context::default();
        let mut app = app_with_documents(&path);
        app.state.document.open_documents.remove(0);
        app.state.document.active_doc_idx = Some(0);
        let previous_path = app.state.active_path();
        assert_eq!(previous_path, Some(path.clone()));

        app.process_action(&ctx, action);

        assert_eq!(app.state.active_path(), previous_path);
        assert!(!activation_focus(&ctx, &path));
    }
}

#[test]
fn switching_to_text_document_keeps_existing_text_focus() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("notes.md");
    std::fs::write(&path, "# notes").expect("write fixture");
    let ctx = egui::Context::default();
    let mut app = app_with_documents(&path);
    let editor_id = egui::Id::new("tab-activation-text-editor");
    let mut text = String::new();
    let mut output = ctx.run_ui(page_input(Vec::new()), |ui| {
        ui.add(egui::TextEdit::singleline(&mut text).id(editor_id))
            .request_focus();
    });
    output.textures_delta.clear();
    drop(output);

    app.process_action(&ctx, AppAction::SelectNextTab);

    assert_eq!(app.state.active_path(), Some(path.clone()));
    let mut activation_result = Some(true);
    let mut output = ctx.run_ui(page_input(Vec::new()), |ui| {
        ui.add(egui::TextEdit::singleline(&mut text).id(editor_id));
        let response = super::navigation_response(ui, 0);
        activation_result =
            crate::app::preview::keyboard_focus::PageNavigationFocusOps::focus_if_activated(
                ui.ctx(),
                &response,
                &path,
            );
    });
    output.textures_delta.clear();
    drop(output);
    assert_eq!(activation_result, None);
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(editor_id));
}
