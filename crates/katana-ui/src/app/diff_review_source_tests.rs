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
fn lint_fix_batch_source_is_used_for_review_content() {
    let mut app = make_app();
    let ctx = eframe::egui::Context::default();
    let dir = tempfile::tempdir().expect("tempdir must be created");
    let path = dir.path().join("doc.md");
    std::fs::write(&path, "disk\n").expect("fixture must be written");

    app.process_action(
        &ctx,
        AppAction::ApplyLintFixesForFiles(vec![crate::app_action::LintFixBatch {
            path: path.clone(),
            fixes: vec![DiagnosticFix {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 7,
                replacement: "fixed".to_string(),
            }],
            source: Some("source\n".to_string()),
        }]),
    );

    let review = app
        .state
        .layout
        .diff_review_snapshot()
        .expect("diff review must open from batch source");
    assert_eq!(review.target_path, path.to_string_lossy());
    assert_eq!(review.before, "source\n");
    assert_eq!(review.after, "fixed\n");
}

#[test]
fn lint_fix_bulk_uses_batch_source_for_unloaded_open_document() {
    let mut app = make_app();
    let ctx = eframe::egui::Context::default();
    let dir = tempfile::tempdir().expect("tempdir must be created");
    let path = dir.path().join("doc.md");
    std::fs::write(&path, "disk\n").expect("fixture must be written");
    app.state
        .document
        .open_documents
        .push(katana_core::document::Document::new_empty(&path));
    app.state.document.active_doc_idx = Some(0);

    app.process_action(
        &ctx,
        AppAction::ApplyLintFixesForFiles(vec![crate::app_action::LintFixBatch {
            path: path.clone(),
            fixes: vec![DiagnosticFix {
                start_line: 2,
                start_column: 1,
                end_line: 2,
                end_column: 7,
                replacement: "fixed".to_string(),
            }],
            source: Some("first\nsecond\n".to_string()),
        }]),
    );

    let review = app
        .state
        .layout
        .diff_review_snapshot()
        .expect("diff review must open from batch source");
    assert_eq!(review.target_path, path.to_string_lossy());
    assert_eq!(review.before, "first\nsecond\n");
    assert_eq!(review.after, "first\nfixed\n");
}

#[test]
fn lint_fix_bulk_applies_unloaded_open_document_after_loading_content() {
    let mut app = make_app();
    let ctx = eframe::egui::Context::default();
    let dir = tempfile::tempdir().expect("tempdir must be created");
    let path = dir.path().join("doc.md");
    std::fs::write(&path, "first\nsecond\n").expect("fixture must be written");
    app.state
        .document
        .open_documents
        .push(katana_core::document::Document::new_empty(&path));
    app.state.document.active_doc_idx = Some(0);

    app.process_action(
        &ctx,
        AppAction::ApplyLintFixesForFiles(vec![crate::app_action::LintFixBatch {
            path: path.clone(),
            fixes: vec![DiagnosticFix {
                start_line: 2,
                start_column: 1,
                end_line: 2,
                end_column: 7,
                replacement: "fixed".to_string(),
            }],
            source: Some("first\nsecond\n".to_string()),
        }]),
    );
    app.process_action(&ctx, AppAction::ConfirmCurrentDiffReviewFile);

    let doc = app
        .state
        .active_document()
        .expect("document must stay active");
    assert_eq!(doc.path, path);
    assert_eq!(doc.buffer, "first\nfixed\n");
}

#[test]
fn lint_fix_bulk_prefers_open_document_over_stale_batch_source() {
    let mut app = make_app();
    let ctx = eframe::egui::Context::default();
    let dir = tempfile::tempdir().expect("tempdir must be created");
    let path = dir.path().join("doc.md");
    std::fs::write(&path, "gamma\n").expect("fixture must be written");

    app.handle_select_document(path.clone(), true);
    app.process_action(
        &ctx,
        AppAction::ApplyLintFixesForFiles(vec![crate::app_action::LintFixBatch {
            path: path.clone(),
            fixes: vec![DiagnosticFix {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 6,
                replacement: "delta".to_string(),
            }],
            source: Some("alpha\n".to_string()),
        }]),
    );

    let review = app
        .state
        .layout
        .diff_review_snapshot()
        .expect("diff review must use the open document content");
    assert_eq!(review.before, "gamma\n");
    assert_eq!(review.after, "delta\n");

    app.process_action(&ctx, AppAction::ConfirmCurrentDiffReviewFile);
    let doc = app
        .state
        .active_document()
        .expect("document must stay active");
    assert_eq!(doc.path, path);
    assert_eq!(doc.buffer, "delta\n");
}
