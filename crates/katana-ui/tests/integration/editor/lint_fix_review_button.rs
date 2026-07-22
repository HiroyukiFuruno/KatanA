use crate::integration::harness_utils::{fresh_temp_dir, setup_harness, wait_for_workspace_load};
use eframe::egui;
use egui_kittest::kittest::Queryable;
use katana_markdown_linter::rules::markdown::{
    DiagnosticFix, DiagnosticRange, DiagnosticSeverity, MarkdownDiagnostic, OfficialRuleMeta,
    RuleParityStatus,
};
use katana_ui::app_state::{AppAction, ViewMode};
use katana_ui::i18n::I18nOps;
use katana_ui::state::diagnostics::ProblemsScope;
use std::path::PathBuf;

const UI_SETTLE_FRAME_LIMIT: usize = 10;

#[test]
fn editor_diagnostic_fix_button_opens_lint_fix_review_tab() {
    let _guard = crate::integration::lock_serial_test_mutex();
    I18nOps::set_language("en");
    let mut harness = setup_harness();
    harness.step();

    let file_path = open_document_with_fixable_diagnostic(&mut harness);
    let line_number_rect = harness
        .query_all_by_label("1")
        .next()
        .expect("line number must be visible")
        .rect();
    let diagnostic_icon = harness
        .query_all_by_role(egui::accesskit::Role::Image)
        .find(|node| {
            let rect = node.rect();
            rect.center().x < line_number_rect.left()
                && (rect.center().y - line_number_rect.center().y).abs() < 8.0
        })
        .expect("diagnostic action icon must be visible");

    diagnostic_icon.hover();
    harness.run_steps(2);
    harness.get_by_label(&I18nOps::get().linter.fix).click();
    harness.run_steps(5);

    let state = harness.state_mut().app_state_mut();
    let active_path = state
        .active_document()
        .expect("lint fix review tab must be active")
        .path
        .to_string_lossy()
        .to_string();
    assert_eq!(active_path, "Katana://DiffReview/LintFixReview");
    let review = state
        .layout
        .diff_review_snapshot()
        .expect("lint fix review state must be created");
    assert_eq!(review.target_path, file_path.to_string_lossy());
    assert_eq!(review.after, "beta\n");
}

#[test]
fn lint_fix_review_tab_shows_cancel_all_button() {
    let _guard = crate::integration::lock_serial_test_mutex();
    I18nOps::set_language("en");
    let mut harness = setup_harness();
    harness.step();

    let restore_path = open_document_with_fixable_diagnostic(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::ApplyLintFixesForFiles(vec![
            lint_fix_batch(PathBuf::from("/tmp/first.md"), "alpha\n", "beta"),
            lint_fix_batch(PathBuf::from("/tmp/second.md"), "gamma\n", "delta"),
        ]));
    harness.run_steps(5);

    harness
        .get_by_label(&I18nOps::get().diff_review.reject_all)
        .click();
    harness.run_steps(5);

    let state = harness.state_mut().app_state_mut();
    assert!(state.layout.diff_review_snapshot().is_none());
    let active_path = state
        .active_document()
        .expect("original document must be restored")
        .path
        .clone();
    assert_eq!(active_path, restore_path);
}

#[test]
fn problems_fix_all_detected_opens_review_for_every_open_tab() {
    let _guard = crate::integration::lock_serial_test_mutex();
    I18nOps::set_language("en");
    let mut harness = setup_harness();
    harness.step();

    let paths = open_three_documents_with_fixable_diagnostics(&mut harness);
    harness
        .state_mut()
        .app_state_mut()
        .diagnostics
        .is_panel_open = true;
    harness.run_steps(3);

    harness
        .get_by_label(&I18nOps::get().status.fix_all_detected_problems)
        .click();
    harness.run_steps(5);

    let state = harness.state_mut().app_state_mut();
    let review = state
        .layout
        .diff_review_snapshot()
        .expect("lint fix review state must be created");
    assert_eq!(review.file_count, paths.len());
}

#[test]
fn problems_fix_all_keeps_unloaded_open_tab_in_review() {
    let _guard = crate::integration::lock_serial_test_mutex();
    I18nOps::set_language("en");
    let mut harness = setup_harness();
    harness.step();

    let paths = open_three_documents_with_fixable_diagnostics(&mut harness);
    let temp_dir = paths[0].parent().expect("fixture must have parent");
    let unloaded_path = temp_dir.join("unloaded.md");
    std::fs::write(&unloaded_path, "alpha\nsecond\n").expect("fixture must be written");
    let unloaded_path = unloaded_path
        .canonicalize()
        .expect("fixture path must be canonicalized");
    harness
        .state_mut()
        .app_state_mut()
        .document
        .open_documents
        .push(katana_core::document::Document::new_empty(&unloaded_path));
    harness
        .state_mut()
        .app_state_mut()
        .diagnostics
        .update_diagnostics_for_content(
            unloaded_path,
            "alpha\nsecond\n",
            vec![second_line_fixable_diagnostic()],
        );
    harness
        .state_mut()
        .app_state_mut()
        .diagnostics
        .is_panel_open = true;
    harness.run_steps(3);

    harness
        .get_by_label(&I18nOps::get().status.fix_all_detected_problems)
        .click();
    harness.run_steps(5);

    let state = harness.state_mut().app_state_mut();
    let review = state
        .layout
        .diff_review_snapshot()
        .expect("lint fix review state must be created");
    assert_eq!(review.file_count, paths.len() + 1);
}

#[test]
fn problems_status_count_follows_scope_only_while_panel_open() {
    let _guard = crate::integration::lock_serial_test_mutex();
    I18nOps::set_language("en");
    let mut harness = setup_harness();
    harness.step();

    open_three_documents_with_fixable_diagnostics(&mut harness);
    harness
        .state_mut()
        .app_state_mut()
        .diagnostics
        .is_panel_open = true;
    harness.run_steps(3);

    wait_for_label(&mut harness, &problem_count_label(3));
    harness.state_mut().app_state_mut().diagnostics.scope = ProblemsScope::ActiveTab;
    wait_for_label(&mut harness, &problem_count_label(1));

    let close_label = I18nOps::get().status.problems_panel_close.clone();
    wait_for_label(&mut harness, &close_label);
    harness.get_by_label(&close_label).click();
    wait_for_label(&mut harness, &problem_count_label(3));
}

fn wait_for_label(
    harness: &mut egui_kittest::Harness<'static, katana_ui::shell::KatanaApp>,
    label: &str,
) {
    for _ in 0..UI_SETTLE_FRAME_LIMIT {
        if harness.query_all_by_label(label).next().is_some() {
            return;
        }
        harness.step();
    }
    panic!("label did not appear after UI settled: {label}");
}

fn open_document_with_fixable_diagnostic(
    harness: &mut egui_kittest::Harness<'static, katana_ui::shell::KatanaApp>,
) -> PathBuf {
    let temp_dir = fresh_temp_dir("katana_test_editor_fix_button");
    let file_path = temp_dir.join("doc.md");
    std::fs::write(&file_path, "alpha\n").expect("fixture must be written");
    let file_path = file_path
        .canonicalize()
        .expect("fixture path must be canonicalized");

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir));
    wait_for_workspace_load(harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(file_path.clone()));
    harness.run_steps(5);
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness
        .state_mut()
        .app_state_mut()
        .diagnostics
        .update_diagnostics_for_content(
            file_path.clone(),
            "alpha\n",
            vec![fixable_diagnostic(file_path.clone())],
        );
    harness.run_steps(5);
    file_path
}

fn open_three_documents_with_fixable_diagnostics(
    harness: &mut egui_kittest::Harness<'static, katana_ui::shell::KatanaApp>,
) -> Vec<PathBuf> {
    let temp_dir = fresh_temp_dir("katana_test_open_tabs_fix_all");
    let mut paths = Vec::new();
    for filename in ["first.md", "second.md", "third.md"] {
        let file_path = temp_dir.join(filename);
        std::fs::write(&file_path, "alpha\n").expect("fixture must be written");
        paths.push(
            file_path
                .canonicalize()
                .expect("fixture path must be canonicalized"),
        );
    }

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir));
    wait_for_workspace_load(harness);
    for path in &paths {
        harness
            .state_mut()
            .trigger_action(AppAction::SelectDocument(path.clone()));
        harness.run_steps(3);
        harness
            .state_mut()
            .app_state_mut()
            .diagnostics
            .update_diagnostics_for_content(
                path.clone(),
                "alpha\n",
                vec![fixable_diagnostic(path.clone())],
            );
    }
    harness.run_steps(5);
    paths
}

fn lint_fix_batch(
    path: PathBuf,
    source: &str,
    replacement: &str,
) -> katana_ui::app_action::LintFixBatch {
    katana_ui::app_action::LintFixBatch {
        path,
        fixes: vec![DiagnosticFix {
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 6,
            replacement: replacement.to_string(),
        }],
        source: Some(source.to_string()),
    }
}

fn problem_count_label(count: usize) -> String {
    I18nOps::tf(
        &I18nOps::get().status.problems_count_format,
        &[("count", &count.to_string())],
    )
}

fn fixable_diagnostic(file_path: PathBuf) -> MarkdownDiagnostic {
    MarkdownDiagnostic {
        file: file_path,
        rule_id: "MD001".to_string(),
        severity: DiagnosticSeverity::Warning,
        message: "message".to_string(),
        range: DiagnosticRange {
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 6,
        },
        fix_info: Some(DiagnosticFix {
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 6,
            replacement: "beta".to_string(),
        }),
        official_meta: Some(OfficialRuleMeta {
            code: "MD001",
            title: "title",
            description: "description",
            docs_url: "",
            aliases: &[],
            parity: RuleParityStatus::Official,
            is_fixable: true,
            properties: &[],
        }),
    }
}

fn second_line_fixable_diagnostic() -> MarkdownDiagnostic {
    let mut diagnostic = fixable_diagnostic(PathBuf::from("/tmp/unloaded.md"));
    diagnostic.range = DiagnosticRange {
        start_line: 2,
        start_column: 1,
        end_line: 2,
        end_column: 7,
    };
    diagnostic.fix_info = Some(DiagnosticFix {
        start_line: 2,
        start_column: 1,
        end_line: 2,
        end_column: 7,
        replacement: "fixed".to_string(),
    });
    diagnostic
}
