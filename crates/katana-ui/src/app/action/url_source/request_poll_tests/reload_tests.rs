use super::app;
use crate::shell::KatanaApp;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

const LOAD_TIMEOUT: Duration = Duration::from_secs(2);
const HTTP_FORBIDDEN: u16 = 403;
const HTTP_OK: u16 = 200;
const REQUEST_BUFFER_BYTES: usize = 1024;
const PDF: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../scripts/screenshot/fixtures/v0-22-38-multi-format/representative.pdf"
));
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn failed_initial_pdf_reloads_requested_url_and_accepts_pdf_recovery() -> TestResult {
    let _guard = crate::preview_pane::html_browser_runtime_test_guard();
    let listener = TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let url = format!("http://{}/report.pdf", listener.local_addr()?);
    let server = thread::spawn(move || serve_retry(listener));
    let mut app = app();
    let ctx = egui::Context::default();
    app.handle_open_url(&ctx, url.clone());
    wait_for_load(&mut app, &ctx)?;
    assert_initial_failure(&app, &url);
    app.handle_action_refresh_document(&ctx, true);
    assert!(
        app.state.url_tab.is_loading,
        "manual reload must fetch failed URL"
    );
    wait_for_load(&mut app, &ctx)?;
    let requests = server.join().map_err(|_| "retry server panicked")??;
    assert_retry_success(&app, &url);
    assert_eq!(requests, vec!["/report.pdf"; 2]);
    Ok(())
}

fn assert_retry_success(app: &KatanaApp, url: &str) {
    let active = app
        .state
        .document
        .active_doc_idx
        .expect("recovered document");
    assert!(
        app.state.url_tab.last_error.is_none(),
        "retry error: {:?}",
        app.state.url_tab.last_error
    );
    assert_recovered_document(app, active, url);
}

fn assert_initial_failure(app: &KatanaApp, url: &str) {
    assert_eq!(
        app.tab_previews[0]
            .pane
            .document_failure
            .as_ref()
            .unwrap()
            .document,
        url
    );
}

fn assert_recovered_document(app: &KatanaApp, active: usize, url: &str) {
    let path = &app.state.document.open_documents[active].path;
    assert_eq!(
        app.state.url_tab.document_source_url_for_document(path),
        Some(url)
    );
    assert!(
        app.tab_previews
            .iter()
            .all(|preview| preview.pane.document_failure.is_none())
    );
}

fn wait_for_load(app: &mut KatanaApp, ctx: &egui::Context) -> TestResult {
    let deadline = Instant::now() + LOAD_TIMEOUT;
    while app.state.url_tab.is_loading {
        app.poll_url_source(ctx);
        if Instant::now() >= deadline {
            return Err("URL load exceeded original test deadline".into());
        }
        thread::yield_now();
    }
    Ok(())
}

fn serve_retry(listener: TcpListener) -> std::io::Result<Vec<String>> {
    let mut requests = Vec::new();
    for status in [HTTP_FORBIDDEN, HTTP_OK] {
        let deadline = Instant::now() + LOAD_TIMEOUT;
        let mut stream = accept_until(&listener, deadline)?;
        stream.set_nonblocking(false)?;
        stream.set_read_timeout(Some(LOAD_TIMEOUT))?;
        let mut request = [0; REQUEST_BUFFER_BYTES];
        let bytes = stream.read(&mut request)?;
        requests.push(
            String::from_utf8_lossy(&request[..bytes])
                .split_whitespace()
                .nth(1)
                .unwrap_or_default()
                .to_owned(),
        );
        let body = if status == HTTP_FORBIDDEN {
            &[][..]
        } else {
            PDF
        };
        let mut response = format!("HTTP/1.1 {status} OK\r\nContent-Type: application/pdf\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).into_bytes();
        response.extend_from_slice(body);
        stream.write_all(&response)?;
    }
    Ok(requests)
}

fn accept_until(listener: &TcpListener, deadline: Instant) -> std::io::Result<std::net::TcpStream> {
    loop {
        match listener.accept() {
            Ok((stream, _)) => return Ok(stream),
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
            {
                thread::yield_now()
            }
            Err(error) => return Err(error),
        }
    }
}
