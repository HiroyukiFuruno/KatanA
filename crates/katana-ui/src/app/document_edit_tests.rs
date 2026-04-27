use std::sync::Arc;

use crate::app::{ActionOps, DocumentOps};
use crate::app_action::AppAction;
use crate::shell::KatanaApp;
use katana_markdown_linter::rules::markdown::DiagnosticFix;
use katana_platform::DiffViewMode;

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
fn lint_fix_opens_review_before_changing_active_file() {
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
    assert_eq!(doc.buffer, "alpha\n");
    assert!(app.state.layout.diff_review.is_some());

    app.process_action(&ctx, AppAction::ConfirmCurrentDiffReviewFile);

    let doc = app
        .state
        .active_document()
        .expect("document must stay active");
    assert_eq!(doc.buffer, "beta\n");
    let restored = undo_text(&ctx, None, &path, "beta\n");
    assert_eq!(restored, "alpha\n");
}

#[test]
fn lint_fix_reject_keeps_active_file_unchanged() {
    let mut app = make_app();
    let ctx = eframe::egui::Context::default();
    let dir = tempfile::tempdir().expect("tempdir must be created");
    let path = dir.path().join("doc.md");
    std::fs::write(&path, "alpha\n").expect("fixture must be written");

    app.handle_select_document(path, true);
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
    app.process_action(&ctx, AppAction::RejectCurrentDiffReviewFile);

    let doc = app
        .state
        .active_document()
        .expect("document must stay active");
    assert_eq!(doc.buffer, "alpha\n");
    assert!(app.state.layout.diff_review.is_none());
}

#[test]
fn lint_fix_for_files_applies_only_accepted_files() {
    let mut app = make_app();
    let ctx = eframe::egui::Context::default();
    let dir = tempfile::tempdir().expect("tempdir must be created");
    let first_path = dir.path().join("first.md");
    let second_path = dir.path().join("second.md");
    std::fs::write(&first_path, "alpha\n").expect("first fixture must be written");
    std::fs::write(&second_path, "gamma\n").expect("second fixture must be written");

    app.handle_select_document(first_path.clone(), true);
    app.process_action(
        &ctx,
        AppAction::ApplyLintFixesForFiles(vec![
            crate::app_action::LintFixBatch {
                path: first_path.clone(),
                fixes: vec![DiagnosticFix {
                    start_line: 1,
                    start_column: 1,
                    end_line: 1,
                    end_column: 6,
                    replacement: "beta".to_string(),
                }],
            },
            crate::app_action::LintFixBatch {
                path: second_path.clone(),
                fixes: vec![DiagnosticFix {
                    start_line: 1,
                    start_column: 1,
                    end_line: 1,
                    end_column: 6,
                    replacement: "delta".to_string(),
                }],
            },
        ]),
    );

    app.process_action(&ctx, AppAction::ConfirmCurrentDiffReviewFile);
    app.process_action(&ctx, AppAction::RejectCurrentDiffReviewFile);

    app.handle_select_document(first_path, true);
    let first_doc = app
        .state
        .active_document()
        .expect("first document must be active");
    assert_eq!(first_doc.buffer, "beta\n");
    assert_eq!(
        std::fs::read_to_string(second_path).expect("second fixture must be readable"),
        "gamma\n"
    );
}

#[test]
fn lint_fix_review_uses_setting_without_persisting_temporary_mode() {
    let mut app = make_app();
    let ctx = eframe::egui::Context::default();
    let dir = tempfile::tempdir().expect("tempdir must be created");
    let path = dir.path().join("doc.md");
    std::fs::write(&path, "alpha\n").expect("fixture must be written");
    app.state
        .config
        .settings
        .settings_mut()
        .behavior
        .diff_view_mode = DiffViewMode::Inline;

    app.handle_select_document(path, true);
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

    let review = app
        .state
        .layout
        .diff_review
        .as_mut()
        .expect("diff review must open");
    assert_eq!(review.mode, DiffViewMode::Inline);
    review.mode = DiffViewMode::Split;

    assert_eq!(
        app.state.config.settings.settings().behavior.diff_view_mode,
        DiffViewMode::Inline
    );
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
