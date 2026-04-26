use std::sync::Arc;

use crate::app::{ActionOps, DocumentOps};
use crate::app_action::AppAction;
use crate::shell::KatanaApp;
use katana_markdown_linter::rules::markdown::DiagnosticFix;

fn make_app() -> KatanaApp {
    let state = crate::app_state::AppState::new(
        katana_core::ai::AiProviderRegistry::new(),
        katana_core::plugin::PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    KatanaApp::new(state)
}

#[test]
fn lint_fix_records_undo_point_for_active_file() {
    let mut app = make_app();
    let ctx = eframe::egui::Context::default();
    let dir = tempfile::tempdir().expect("tempdir must be created");
    let path = dir.path().join("doc.md");
    std::fs::write(&path, "alpha\n").expect("fixture must be written");

    app.handle_select_document(path.clone(), true);
    app.process_action(
        &ctx,
        AppAction::ApplyLintFixes(vec![DiagnosticFix {
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 6,
            replacement: "beta".to_string(),
        }]),
    );

    let doc = app
        .state
        .active_document()
        .expect("document must stay active");
    assert_eq!(doc.buffer, "beta\n");
    let restored = undo_text(&ctx, None, &path, "beta\n");
    assert_eq!(restored, "alpha\n");
}

fn undo_text(
    ctx: &eframe::egui::Context,
    workspace_root: Option<&std::path::Path>,
    path: &std::path::Path,
    current: &str,
) -> String {
    let id = crate::editor_undo::EditorUndoIdentity::text_edit_id(workspace_root, path);
    let state = eframe::egui::TextEdit::load_state(ctx, id).expect("undo state must exist");
    let mut undoer = state.undoer();
    let cursor = eframe::egui::text::CCursorRange::one(eframe::egui::text::CCursor::new(
        current.chars().count(),
    ));
    let restored = undoer
        .undo(&(cursor, current.to_string()))
        .expect("undo point must exist");
    restored.1.clone()
}
