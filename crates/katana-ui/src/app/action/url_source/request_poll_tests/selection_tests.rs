use super::{app, html, open_html_targets, request, source_body};
use crate::app::action::ActionOps;
use crate::app_state::AppAction;
use std::path::PathBuf;

const OLD_URL: &str = "https://example.test/old.html";
const NEW_URL: &str = "https://example.test/new.html";

#[test]
fn older_targeted_completion_updates_content_without_stealing_new_url_selection() {
    verify_selection_after_late_completion(true);
}

#[test]
fn older_url_bar_completion_does_not_steal_newer_target_selection() {
    verify_selection_after_late_completion(false);
}

fn verify_selection_after_late_completion(older_targeted: bool) {
    let mut app = app();
    let old = super::super::document::remote_document_path(OLD_URL, "html");
    let new = super::super::document::remote_document_path(NEW_URL, "html");
    open_html_targets(&mut app, &old, &new);
    let (first, pending) = request(OLD_URL, older_targeted.then_some(old.clone()));
    let (second, ready) = request(NEW_URL, (!older_targeted).then_some(new.clone()));
    app.state
        .url_tab
        .pending_url_requests
        .extend([pending, ready]);
    second.send(Ok(html(NEW_URL, "new"))).unwrap();
    app.poll_url_source(&egui::Context::default());
    assert_eq!(app.state.active_path(), Some(new.clone()));
    assert_eq!(app.state.url_tab.input, NEW_URL);
    first.send(Ok(html(OLD_URL, "old"))).unwrap();
    app.poll_url_source(&egui::Context::default());
    assert_eq!(
        source_body(&app, &old).as_deref(),
        Some("<html><body>old</body></html>")
    );
    assert_eq!(app.state.active_path(), Some(new));
    assert_eq!(app.state.url_tab.input, NEW_URL);
    assert!(app.state.url_tab.pending_url_requests.is_empty());
}

#[test]
fn targeted_completion_does_not_undo_manual_tab_selection() {
    let mut app = app();
    let old = PathBuf::from("old.html");
    let selected = PathBuf::from("selected.md");
    open_html_targets(&mut app, &old, &selected);
    app.state.document.active_doc_idx = Some(0);
    let (sender, mut pending) = request(OLD_URL, Some(old.clone()));
    pending.selection_owner = Some(app.state.url_tab.selection_owner.clone());
    app.state.url_tab.pending_url_requests.push_back(pending);
    app.poll_url_source(&egui::Context::default());
    app.process_action(
        &egui::Context::default(),
        AppAction::SelectDocument(selected.clone()),
    );
    app.state.url_tab.input = "user input".to_owned();
    sender.send(Ok(html(OLD_URL, "updated"))).unwrap();
    app.poll_url_source(&egui::Context::default());
    assert_eq!(
        source_body(&app, &old).as_deref(),
        Some("<html><body>updated</body></html>")
    );
    assert_eq!(app.state.active_path(), Some(selected));
    assert_eq!(app.state.url_tab.input, "user input");
}

#[test]
fn newer_targeted_request_can_activate_after_older_url_bar_completion() {
    verify_newer_foreground_request(false);
}

#[test]
fn newer_url_bar_request_can_activate_after_older_targeted_completion() {
    verify_newer_foreground_request(true);
}

fn verify_newer_foreground_request(older_targeted: bool) {
    let mut app = app();
    let old = super::super::document::remote_document_path(OLD_URL, "html");
    let new = super::super::document::remote_document_path(NEW_URL, "html");
    open_html_targets(&mut app, &old, &new);
    app.state.document.active_doc_idx = Some(1);
    let (first, pending) = request(OLD_URL, older_targeted.then_some(old));
    let (second, ready) = request(NEW_URL, (!older_targeted).then_some(new.clone()));
    app.state
        .url_tab
        .pending_url_requests
        .extend([pending, ready]);
    first.send(Ok(html(OLD_URL, "old"))).unwrap();
    app.poll_url_source(&egui::Context::default());
    second.send(Ok(html(NEW_URL, "new"))).unwrap();
    app.poll_url_source(&egui::Context::default());
    assert_eq!(app.state.active_path(), Some(new));
    assert_eq!(app.state.url_tab.input, NEW_URL);
}

#[test]
fn manual_selection_round_trip_preserves_user_input_after_target_refresh() {
    let mut app = app();
    let old = super::super::document::remote_document_path(OLD_URL, "html");
    let other = PathBuf::from("selected.md");
    open_html_targets(&mut app, &old, &other);
    app.state.document.active_doc_idx = Some(0);
    let (sender, mut pending) = request(OLD_URL, Some(old.clone()));
    pending.selection_owner = Some(app.state.url_tab.selection_owner.clone());
    app.state.url_tab.pending_url_requests.push_back(pending);
    let ctx = egui::Context::default();
    app.poll_url_source(&ctx);
    app.process_action(&ctx, AppAction::SelectDocument(other));
    app.process_action(&ctx, AppAction::SelectDocument(old.clone()));
    app.state.url_tab.input = "manual input".to_owned();
    sender.send(Ok(html(OLD_URL, "updated"))).unwrap();
    app.poll_url_source(&ctx);
    assert_eq!(app.state.active_path(), Some(old));
    assert_eq!(app.state.url_tab.input, "manual input");
}
