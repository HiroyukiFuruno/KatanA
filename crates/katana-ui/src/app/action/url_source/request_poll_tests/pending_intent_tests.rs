use super::{app, html, open_html_targets, request, source_body};
use crate::state::HtmlSource;
use std::thread;
use std::time::Instant;

const A_URL: &str = "https://example.test/a.html";

mod http_failure;
use http_failure::ControlledFailure;
pub(super) use http_failure::{ControlledResponse, LOAD_TIMEOUT};

struct Scenario {
    app: crate::shell::KatanaApp,
    a_path: std::path::PathBuf,
    b_path: std::path::PathBuf,
    b_url: String,
}

impl Scenario {
    fn new(b_url: String) -> Self {
        let mut app = app();
        let (a_path, b_path) = seed_documents(&mut app, &b_url);
        Self {
            app,
            a_path,
            b_path,
            b_url,
        }
    }
}

#[test]
fn targeted_success_does_not_override_new_targetless_failure_intent()
-> Result<(), Box<dyn std::error::Error>> {
    let Scenario {
        app,
        a_path,
        b_path,
        b_url,
    } = run_scenario(true)?;
    assert_b_selection(&app, &b_path, &b_url);
    assert_b_failure(&app, &b_url);
    assert_eq!(
        source_body(&app, &a_path).as_deref(),
        Some("<html><body>A refreshed</body></html>")
    );
    Ok(())
}

#[test]
fn older_targeted_success_after_new_failure_preserves_newer_intent()
-> Result<(), Box<dyn std::error::Error>> {
    let Scenario {
        app,
        a_path,
        b_path,
        b_url,
    } = run_scenario(false)?;
    assert_b_selection(&app, &b_path, &b_url);
    assert_b_failure(&app, &b_url);
    assert_eq!(
        source_body(&app, &a_path).as_deref(),
        Some("<html><body>A refreshed</body></html>")
    );
    Ok(())
}

fn run_scenario(targeted_first: bool) -> Result<Scenario, Box<dyn std::error::Error>> {
    let failure = ControlledFailure::start()?;
    let mut scenario = Scenario::new(failure.url.clone());
    let a_sender = enqueue_targeted(&mut scenario.app, &scenario.a_path);
    scenario.app.state.url_tab.is_loading = true;
    let ctx = egui::Context::default();
    scenario.app.handle_open_url(&ctx, scenario.b_url.clone());
    failure.wait_accepted()?;
    if targeted_first {
        a_sender.send(Ok(html(A_URL, "A refreshed")))?;
        scenario.app.poll_url_source(&ctx);
        assert_b_selection(&scenario.app, &scenario.b_path, &scenario.b_url);
    }
    let requested_path = failure.finish()?;
    assert_eq!(requested_path, "/b.html");
    let remaining = if targeted_first { 0 } else { 1 };
    wait_for_pending_count(&mut scenario.app, &ctx, remaining)?;
    if !targeted_first {
        a_sender.send(Ok(html(A_URL, "A refreshed")))?;
        scenario.app.poll_url_source(&ctx);
    }
    Ok(scenario)
}

fn enqueue_targeted(
    app: &mut crate::shell::KatanaApp,
    a_path: &std::path::Path,
) -> std::sync::mpsc::Sender<Result<crate::state::FetchedUrlSource, crate::state::HtmlSourceError>>
{
    let (sender, mut pending) = request(A_URL, Some(a_path.to_path_buf()));
    pending.selection_owner = Some(app.state.url_tab.selection_owner.clone());
    app.state.url_tab.pending_url_requests.push_back(pending);
    sender
}

fn seed_documents(
    app: &mut crate::shell::KatanaApp,
    b_url: &str,
) -> (std::path::PathBuf, std::path::PathBuf) {
    let a_path = super::super::document::remote_document_path(A_URL, "html");
    let b_path = super::super::document::remote_document_path(b_url, "html");
    open_html_targets(app, &a_path, &b_path);
    seed_source(app, A_URL, &a_path, "A");
    seed_source(app, b_url, &b_path, "B");
    app.state.document.active_doc_idx = Some(1);
    app.state.url_tab.input = b_url.to_owned();
    (a_path, b_path)
}

fn seed_source(app: &mut crate::shell::KatanaApp, url: &str, path: &std::path::Path, body: &str) {
    app.state.url_tab.open_source(
        HtmlSource {
            raw_html: format!("<html><body>{body}</body></html>"),
            source_url: url.to_owned(),
            origin: url.to_owned(),
        },
        path.to_path_buf(),
    );
}

fn assert_b_failure(app: &crate::shell::KatanaApp, url: &str) {
    assert!(matches!(&app.state.url_tab.last_error,
        Some(crate::state::HtmlSourceError::HttpStatus { status: 500, url: failed_url, .. }) if failed_url == url));
    let (message, status) = app
        .state
        .layout
        .status_message
        .as_ref()
        .expect("newer failed request must retain its global failure message");
    assert!(message.contains(url));
    assert_eq!(*status, crate::app_state::StatusType::Error);
}

fn assert_b_selection(app: &crate::shell::KatanaApp, b_path: &std::path::Path, b_url: &str) {
    assert_eq!(app.state.active_path(), Some(b_path.to_path_buf()));
    assert_eq!(app.state.url_tab.input, b_url);
    assert_eq!(
        source_body(app, b_path).as_deref(),
        Some("<html><body>B</body></html>")
    );
}

fn wait_for_pending_count(
    app: &mut crate::shell::KatanaApp,
    ctx: &egui::Context,
    count: usize,
) -> Result<(), &'static str> {
    let deadline = Instant::now() + LOAD_TIMEOUT;
    while app.state.url_tab.pending_url_requests.len() != count {
        app.poll_url_source(ctx);
        if Instant::now() >= deadline {
            return Err("URL load exceeded original test deadline");
        }
        thread::yield_now();
    }
    Ok(())
}
