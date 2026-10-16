use super::{app, open_html_targets, source_body};
use crate::app::action::ActionOps;
use crate::app_state::AppAction;
use crate::state::HtmlSource;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

const LOAD_TIMEOUT: Duration = Duration::from_secs(2);
const B_URL: &str = "https://example.test/b.html";
const REQUEST_BUFFER_BYTES: usize = 1024;

#[test]
fn targetless_http_completion_preserves_manual_document_selection()
-> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let url = format!("http://{}/fetched.html", listener.local_addr()?);
    let server = thread::spawn(move || serve_html(listener));
    let mut app = app();
    let ctx = egui::Context::default();
    let a_path =
        super::super::document::remote_document_path("https://example.test/a.html", "html");
    let b_path = super::super::document::remote_document_path(B_URL, "html");
    open_html_targets(&mut app, &a_path, &b_path);
    seed_b_source(&mut app, &b_path);
    app.handle_open_url(&ctx, url.clone());
    app.process_action(&ctx, AppAction::SelectDocument(b_path.clone()));
    app.state.url_tab.input = B_URL.to_owned();
    wait_for_load(&mut app, &ctx)?;
    let requests = server.join().map_err(|_| "loopback server panicked")??;

    assert_results(&app, &b_path, &url, &requests);
    Ok(())
}

fn seed_b_source(app: &mut crate::shell::KatanaApp, b_path: &std::path::Path) {
    app.state.url_tab.open_source(
        HtmlSource {
            raw_html: "<html><body>B</body></html>".to_owned(),
            source_url: B_URL.to_owned(),
            origin: B_URL.to_owned(),
        },
        b_path.to_path_buf(),
    );
}

fn assert_results(
    app: &crate::shell::KatanaApp,
    b_path: &std::path::Path,
    url: &str,
    requests: &[String],
) {
    assert_eq!(app.state.active_path(), Some(b_path.to_path_buf()));
    assert_eq!(app.state.url_tab.input, B_URL);
    assert_eq!(
        source_body(app, b_path),
        Some("<html><body>B</body></html>".to_owned())
    );
    let fetched_path = super::super::document::remote_document_path(url, "html");
    assert_eq!(
        source_body(app, &fetched_path),
        Some("<html><body>fetched</body></html>".to_owned())
    );
    assert_eq!(requests, ["/fetched.html".to_owned()]);
}

fn wait_for_load(
    app: &mut crate::shell::KatanaApp,
    ctx: &egui::Context,
) -> Result<(), &'static str> {
    let deadline = Instant::now() + LOAD_TIMEOUT;
    while app.state.url_tab.is_loading {
        app.poll_url_source(ctx);
        if Instant::now() >= deadline {
            return Err("URL load exceeded original test deadline");
        }
        thread::yield_now();
    }
    Ok(())
}

fn serve_html(listener: TcpListener) -> std::io::Result<Vec<String>> {
    let mut stream = accept_connection(&listener)?;
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(LOAD_TIMEOUT))?;
    let mut request = [0; REQUEST_BUFFER_BYTES];
    let bytes = stream.read(&mut request)?;
    let path = String::from_utf8_lossy(&request[..bytes])
        .split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .to_owned();
    let body = b"<html><body>fetched</body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.write_all(body)?;
    Ok(vec![path])
}

fn accept_connection(listener: &TcpListener) -> std::io::Result<std::net::TcpStream> {
    let deadline = Instant::now() + LOAD_TIMEOUT;
    loop {
        match listener.accept() {
            Ok((stream, _)) => return Ok(stream),
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
            {
                thread::yield_now();
            }
            Err(error) => return Err(error),
        }
    }
}
