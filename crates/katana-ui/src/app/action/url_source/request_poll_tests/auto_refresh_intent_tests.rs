use super::app;
use super::pending_intent_tests::{ControlledResponse, LOAD_TIMEOUT};
use crate::shell::KatanaApp;
use katana_core::document::Document;
use std::path::PathBuf;
use std::thread;
use std::time::Instant;

const PDF_BODY: &[u8] = b"%PDF-1.7\n%auto-refresh\n";
const HTTP_OK: u16 = 200;
const HTTP_INTERNAL_SERVER_ERROR: u16 = 500;

struct Scenario {
    app: KatanaApp,
    ctx: egui::Context,
    b_url: String,
    c_url: String,
    b_path: PathBuf,
    c_path: PathBuf,
}

impl Scenario {
    fn new(b_url: String, c_url: String) -> Self {
        let mut app = app();
        let c_path = seed_pdf(&mut app, &c_url);
        let b_path = super::super::document::remote_document_path(&b_url, "html");
        Self {
            app,
            ctx: egui::Context::default(),
            b_url,
            c_url,
            b_path,
            c_path,
        }
    }

    fn enqueue(
        &mut self,
        b: &ControlledResponse,
        c: &ControlledResponse,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.app.handle_open_url(&self.ctx, self.b_url.clone());
        b.wait_accepted()?;
        self.app.handle_action_refresh_document(&self.ctx, false);
        c.wait_accepted()?;
        Ok(())
    }
}

#[test]
fn user_success_before_auto_failure_preserves_user_intent() -> Result<(), Box<dyn std::error::Error>>
{
    let scenario = run_scenario(true)?;
    assert_results(&scenario);
    Ok(())
}

#[test]
fn auto_failure_before_user_success_preserves_user_intent() -> Result<(), Box<dyn std::error::Error>>
{
    let scenario = run_scenario(false)?;
    assert_results(&scenario);
    Ok(())
}

fn run_scenario(user_first: bool) -> Result<Scenario, Box<dyn std::error::Error>> {
    let b =
        ControlledResponse::start_with(HTTP_OK, "b.html", b"<html><body>B</body></html>".to_vec())?;
    let c = ControlledResponse::start_with(HTTP_INTERNAL_SERVER_ERROR, "c.pdf", PDF_BODY.to_vec())?;
    let mut scenario = Scenario::new(b.url.clone(), c.url.clone());
    scenario.enqueue(&b, &c)?;
    if user_first {
        finish_user_then_auto(&mut scenario, b, c)?;
    } else {
        finish_auto_then_user(&mut scenario, b, c)?;
    }
    Ok(scenario)
}

fn seed_pdf(app: &mut KatanaApp, url: &str) -> PathBuf {
    let path = super::super::document::remote_document_path(url, "pdf");
    app.state
        .document
        .open_documents
        .push(Document::new(path.clone(), "C".to_owned()));
    app.state.document.active_doc_idx = Some(0);
    app.state
        .url_tab
        .open_document_source(url.to_owned(), path.clone());
    path
}

fn finish_user_then_auto(
    scenario: &mut Scenario,
    b: ControlledResponse,
    c: ControlledResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(b.finish()?, "/b.html");
    wait_for_pending(&mut scenario.app, &scenario.ctx, 1)?;
    assert_user_state(scenario);
    assert_eq!(c.finish()?, "/c.pdf");
    wait_for_pending(&mut scenario.app, &scenario.ctx, 0)?;
    Ok(())
}

fn finish_auto_then_user(
    scenario: &mut Scenario,
    b: ControlledResponse,
    c: ControlledResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(c.finish()?, "/c.pdf");
    wait_for_pending(&mut scenario.app, &scenario.ctx, 1)?;
    assert_eq!(b.finish()?, "/b.html");
    wait_for_pending(&mut scenario.app, &scenario.ctx, 0)?;
    Ok(())
}

fn wait_for_pending(
    app: &mut KatanaApp,
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

fn assert_user_state(scenario: &Scenario) {
    assert_eq!(
        scenario.app.state.active_path(),
        Some(scenario.b_path.clone())
    );
    assert_eq!(scenario.app.state.url_tab.input, scenario.b_url);
    assert_eq!(
        super::source_body(&scenario.app, &scenario.b_path).as_deref(),
        Some("<html><body>B</body></html>")
    );
    assert!(scenario.app.state.url_tab.last_error.is_none());
}

fn assert_results(scenario: &Scenario) {
    assert_user_state(scenario);
    assert!(
        scenario
            .app
            .state
            .document
            .open_documents
            .iter()
            .any(|doc| doc.path == scenario.c_path)
    );
    let failure = scenario
        .app
        .tab_previews
        .iter()
        .find(|preview| preview.path == scenario.c_path)
        .and_then(|preview| preview.pane.document_failure.as_ref())
        .expect("auto refresh failure must target C");
    assert_eq!(failure.document, scenario.c_url);
    assert_eq!(failure.operation, "fetch");
    assert_eq!(
        scenario
            .app
            .state
            .url_tab
            .document_source_url_for_document(&scenario.c_path),
        Some(scenario.c_url.as_str())
    );
}
