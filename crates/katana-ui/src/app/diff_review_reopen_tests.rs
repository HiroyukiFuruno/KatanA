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
fn lint_fix_review_path_uses_non_markdown_virtual_route() {
    let review_path = crate::app::LintFixReviewPath::path();

    assert_eq!(
        review_path,
        std::path::PathBuf::from("Katana://DiffReview/LintFixReview")
    );
    assert!(crate::app::LintFixReviewPath::is_review_path(&review_path));
    assert_ne!(
        review_path.file_name().and_then(|name| name.to_str()),
        Some("lint-fix.md")
    );
    assert_ne!(
        review_path
            .extension()
            .and_then(|extension| extension.to_str()),
        Some("md")
    );
}

#[test]
fn lint_fix_reopening_review_while_active_keeps_original_restore_target() {
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
        }]),
    );

    let review = app
        .state
        .layout
        .diff_review_snapshot()
        .expect("diff review must stay open");
    assert_eq!(review.target_path, path.to_string_lossy());
    assert_eq!(review.after, "delta\n");

    app.process_action(&ctx, AppAction::ConfirmCurrentDiffReviewFile);

    let doc = app
        .state
        .active_document()
        .expect("document must stay active");
    assert_eq!(doc.path, path);
    assert_eq!(doc.buffer, "delta\n");
}

#[test]
fn lint_fix_can_open_again_after_previous_review_is_applied() {
    let mut app = make_app();
    let ctx = eframe::egui::Context::default();
    let dir = tempfile::tempdir().expect("tempdir must be created");
    let path = dir.path().join("doc.md");
    std::fs::write(&path, "alpha\ngamma\n").expect("fixture must be written");

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
                replacement: "beta".to_string(),
            }],
        }]),
    );
    app.process_action(&ctx, AppAction::ConfirmCurrentDiffReviewFile);

    app.process_action(
        &ctx,
        AppAction::ApplyLintFixesForFiles(vec![crate::app_action::LintFixBatch {
            path,
            fixes: vec![DiagnosticFix {
                start_line: 2,
                start_column: 1,
                end_line: 2,
                end_column: 6,
                replacement: "delta".to_string(),
            }],
        }]),
    );

    let doc = app
        .state
        .active_document()
        .expect("diff review document must be active");
    assert_eq!(doc.path, crate::app::LintFixReviewPath::path());
    let review = app
        .state
        .layout
        .diff_review_snapshot()
        .expect("diff review must open again");
    assert_eq!(review.after, "beta\ndelta\n");
}
