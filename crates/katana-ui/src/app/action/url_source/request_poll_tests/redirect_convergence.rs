use super::{app, open_html_targets};
use crate::app::ActionOps;
use crate::app_state::AppAction;
use crate::preview_pane::DocumentSurfaceSource;
use crate::state::HtmlSource;
use std::path::PathBuf;
use std::thread;
use std::time::Instant;

mod server;
use server::{LOAD_TIMEOUT, RedirectServer, Release};

#[derive(Clone, Copy)]
enum CompletionOrder {
    AThenB,
    BThenA,
}

#[test]
fn distinct_sources_converging_on_one_pdf_keep_newest_bytes_and_selection()
-> Result<(), Box<dyn std::error::Error>> {
    for order in [CompletionOrder::AThenB, CompletionOrder::BThenA] {
        run_convergence(order)?;
    }
    Ok(())
}

fn run_convergence(order: CompletionOrder) -> Result<(), Box<dyn std::error::Error>> {
    let server = RedirectServer::start()?;
    let final_url = server.final_url.clone();
    let (mut app, b_path) = seeded_app(&server.b_url);
    let ctx = egui::Context::default();
    app.handle_open_url(&ctx, server.a_url.clone());
    app.process_action(&ctx, AppAction::SelectDocument(b_path));
    app.handle_action_refresh_document(&ctx, true);
    server.wait_ready()?;
    let pending = app.state.url_tab.pending_url_requests.len();
    let releases = match order {
        CompletionOrder::AThenB => [Release::A, Release::B],
        CompletionOrder::BThenA => [Release::B, Release::A],
    };
    server.release(releases[0])?;
    wait_for_one(&mut app, &ctx, pending)?;
    let final_path = super::super::document::remote_document_path(&final_url, "pdf");
    let generation = destination_generation(&app, &final_path);
    let revision = destination_revision(&app, &final_path)?;
    server.release(releases[1])?;
    let mut requests = server.join()?;
    wait_for_pending(&mut app, &ctx)?;
    assert_convergence(&app, &final_path, &final_url, order, generation, &revision)?;
    requests.sort();
    assert_eq!(requests, vec!["/a", "/b", "/final.pdf", "/final.pdf"]);
    Ok(())
}

fn seeded_app(url: &str) -> (crate::shell::KatanaApp, PathBuf) {
    let path = super::super::document::remote_document_path(url, "html");
    let mut app = app();
    open_html_targets(&mut app, &path, &PathBuf::from("other.html"));
    app.state.url_tab.open_source(
        HtmlSource {
            raw_html: "<html><body>B</body></html>".to_owned(),
            source_url: url.to_owned(),
            origin: url.to_owned(),
        },
        path.clone(),
    );
    app.state.document.active_doc_idx = Some(0);
    (app, path)
}

fn assert_convergence(
    app: &crate::shell::KatanaApp,
    path: &std::path::Path,
    url: &str,
    order: CompletionOrder,
    generation: u64,
    revision: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_revisions(app, path, url, order, revision)?;
    assert_destination_state(app, path, url);
    let final_generation = destination_generation(app, path);
    match order {
        CompletionOrder::AThenB => assert!(final_generation > generation),
        CompletionOrder::BThenA => assert_eq!(final_generation, generation),
    }
    Ok(())
}

fn assert_revisions(
    app: &crate::shell::KatanaApp,
    path: &std::path::Path,
    url: &str,
    order: CompletionOrder,
    revision: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let expected_a = expected_revision(url, b'A')?;
    let expected_b = expected_revision(url, b'B')?;
    assert_ne!(expected_a, expected_b);
    assert_eq!(
        revision,
        match order {
            CompletionOrder::AThenB => expected_a,
            CompletionOrder::BThenA => expected_b.clone(),
        }
    );
    assert_eq!(destination_revision(app, path)?, expected_b);
    Ok(())
}

fn assert_destination_state(app: &crate::shell::KatanaApp, path: &std::path::Path, url: &str) {
    let documents = &app.state.document.open_documents;
    assert_eq!(app.state.active_path(), Some(path.to_path_buf()));
    assert!(
        documents
            .iter()
            .any(|document| document.path == std::path::Path::new("other.html"))
    );
    assert_eq!(
        documents
            .iter()
            .filter(|document| document.path == path)
            .count(),
        1
    );
    assert!(app.state.url_tab.pending_url_requests.is_empty());
    assert!(!app.state.url_tab.is_loading);
    assert_eq!(
        app.state.url_tab.document_source_url_for_document(path),
        Some(url)
    );
}

fn wait_for_pending(
    app: &mut crate::shell::KatanaApp,
    ctx: &egui::Context,
) -> Result<(), &'static str> {
    let deadline = Instant::now() + LOAD_TIMEOUT;
    while !app.state.url_tab.pending_url_requests.is_empty() {
        app.poll_url_source(ctx);
        if Instant::now() >= deadline {
            return Err("redirect convergence exceeded original test deadline");
        }
        thread::yield_now();
    }
    Ok(())
}

fn wait_for_one(
    app: &mut crate::shell::KatanaApp,
    ctx: &egui::Context,
    pending: usize,
) -> Result<(), &'static str> {
    let deadline = Instant::now() + LOAD_TIMEOUT;
    while app.state.url_tab.pending_url_requests.len() >= pending {
        app.poll_url_source(ctx);
        if Instant::now() >= deadline {
            return Err("first redirect completion exceeded original test deadline");
        }
        thread::yield_now();
    }
    Ok(())
}

fn destination_generation(app: &crate::shell::KatanaApp, path: &std::path::Path) -> u64 {
    app.tab_previews
        .iter()
        .find(|preview| preview.path == path)
        .expect("converged PDF preview")
        .pane
        .session_generation
}
fn destination_revision(
    app: &crate::shell::KatanaApp,
    path: &std::path::Path,
) -> Result<String, &'static str> {
    app.tab_previews
        .iter()
        .find(|preview| preview.path == path)
        .and_then(|preview| preview.pane.document_surface.as_ref())
        .map(|surface| surface.source().revision.clone())
        .ok_or("converged PDF source missing")
}
fn expected_revision(url: &str, marker: u8) -> Result<String, Box<dyn std::error::Error>> {
    Ok(DocumentSurfaceSource::remote(
        url.to_owned(),
        Some("application/pdf"),
        RedirectServer::body(marker),
    )
    .map_err(|error| error.to_string())?
    .revision)
}
