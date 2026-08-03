mod document;

use super::file_open::FileOpenOps;
use crate::app::url_source::{UrlResponseCollector, ValidatedHttpUrl, ValidatedLocalHtmlUrl};
use crate::shell::KatanaApp;
use crate::state::{HtmlSource, HtmlSourceError, PendingUrlRequest};

const URL_DOCUMENT_PREFIX: &str = "Katana://URL";
const URL_SOURCE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

impl KatanaApp {
    pub(super) fn handle_open_url(&mut self, ctx: &egui::Context, input: String) {
        if url::Url::parse(input.trim()).is_ok_and(|url| url.scheme() == "file") {
            self.open_local_html_url(&input);
            return;
        }
        let url = match ValidatedHttpUrl::parse(&input) {
            Ok(url) => url,
            Err(error) => {
                self.fail_html_source(HtmlSourceError::InvalidUrl(error));
                return;
            }
        };

        self.fetch_html_url(ctx, url, None);
    }

    fn open_local_html_url(&mut self, input: &str) {
        let source = match ValidatedLocalHtmlUrl::parse(input) {
            Ok(source) => source,
            Err(error) => {
                self.fail_html_source(HtmlSourceError::InvalidUrl(error));
                return;
            }
        };
        let path = match source.path().canonicalize() {
            Ok(path) => path,
            Err(error) => {
                self.fail_html_source(HtmlSourceError::LocalFile {
                    url: source.as_str().to_string(),
                    reason: error.to_string(),
                });
                return;
            }
        };
        if !FileOpenOps::is_openable_file(self, &path) {
            self.fail_html_source(HtmlSourceError::LocalFile {
                url: source.as_str().to_string(),
                reason: "path is not an openable file".to_string(),
            });
            return;
        }

        let canonical_url = match source.canonical_url_for(&path) {
            Ok(url) => url,
            Err(error) => {
                self.fail_html_source(HtmlSourceError::InvalidUrl(error));
                return;
            }
        };
        if katana_core::workspace::TreeEntry::path_is_document(&path) {
            self.state.url_tab.cancel_pending_url_requests();
            self.state.url_tab.input = canonical_url;
            FileOpenOps::open_in_current_workspace(self, path);
            self.state.layout.status_message = None;
            return;
        }
        let raw_html = match std::fs::read_to_string(&path) {
            Ok(raw_html) => raw_html,
            Err(error) => {
                self.fail_html_source(HtmlSourceError::LocalFile {
                    url: canonical_url,
                    reason: error.to_string(),
                });
                return;
            }
        };
        self.state.url_tab.cancel_pending_url_requests();
        self.state.url_tab.open_source(
            HtmlSource {
                raw_html,
                source_url: canonical_url.clone(),
                origin: canonical_url,
            },
            path.clone(),
        );
        FileOpenOps::open_in_current_workspace(self, path);
        self.state.layout.status_message = None;
    }

    pub(super) fn fetch_html_url(
        &mut self,
        ctx: &egui::Context,
        url: ValidatedHttpUrl,
        target_document: Option<std::path::PathBuf>,
    ) {
        let (sender, receiver) = std::sync::mpsc::channel();
        self.state
            .url_tab
            .pending_url_requests
            .push_back(PendingUrlRequest {
                response_rx: receiver,
                target_document,
                source_url: url.as_str().to_owned(),
                deadline: std::time::Instant::now() + URL_SOURCE_TIMEOUT,
            });
        self.state.url_tab.input = url.as_str().to_string();
        self.state.url_tab.is_loading = true;
        self.state.url_tab.last_error = None;

        let collector = std::cell::RefCell::new(UrlResponseCollector::new(url.clone()));
        let repaint = ctx.clone();
        ehttp::streaming::fetch(
            ehttp::Request::get(url.as_str()).with_timeout(Some(URL_SOURCE_TIMEOUT)),
            move |part| {
                let Some(result) = collector.borrow_mut().accept(part) else {
                    return std::ops::ControlFlow::Continue(());
                };
                let _ = sender.send(result);
                repaint.request_repaint_after(std::time::Duration::ZERO);
                std::ops::ControlFlow::Break(())
            },
        );
    }

    pub(crate) fn poll_url_source(&mut self, ctx: &egui::Context) {
        while let Some(request) = self.state.url_tab.pending_url_requests.pop_front() {
            match request.response_rx.try_recv() {
                Ok(Ok(source)) => {
                    self.apply_fetched_url_source(source, request.target_document);
                    self.state.layout.status_message = None;
                    self.state.url_tab.is_loading =
                        !self.state.url_tab.pending_url_requests.is_empty();
                }
                Ok(Err(error)) => {
                    self.fail_html_source(error);
                    self.state.url_tab.is_loading =
                        !self.state.url_tab.pending_url_requests.is_empty();
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    if std::time::Instant::now() >= request.deadline {
                        self.fail_html_source(HtmlSourceError::Timeout {
                            url: request.source_url,
                            seconds: URL_SOURCE_TIMEOUT.as_secs(),
                        });
                        self.state.url_tab.is_loading =
                            !self.state.url_tab.pending_url_requests.is_empty();
                        continue;
                    }
                    let repaint_after = request
                        .deadline
                        .saturating_duration_since(std::time::Instant::now())
                        .min(std::time::Duration::from_millis(100));
                    self.state.url_tab.pending_url_requests.push_front(request);
                    ctx.request_repaint_after(repaint_after);
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.fail_html_source(HtmlSourceError::Network(
                        "URL request channel disconnected".to_string(),
                    ));
                    self.state.url_tab.is_loading =
                        !self.state.url_tab.pending_url_requests.is_empty();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::URL_SOURCE_TIMEOUT;
    use super::document::{failed_document_identity, remote_document_path};
    use crate::app::url_source::ValidatedHttpUrl;
    use crate::shell::KatanaApp;
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        sync::Arc,
        thread,
        time::{Duration, Instant},
    };

    const HTML: &str =
        "<html><body><details><summary>More</summary><p>Body</p></details></body></html>";
    const RESPONSE_COUNT: usize = 2;
    const URL_LOAD_TIMEOUT: Duration = Duration::from_secs(2);
    const BROWSER_UPDATE_TIMEOUT: Duration = Duration::from_secs(10);

    #[test]
    fn user_entered_http_document_keeps_origin_through_refresh_and_browser_session() -> TestResult {
        let _runtime_guard = crate::preview_pane::html_browser_runtime_test_guard();
        let (url, server) = html_server()?;
        let ctx = egui::Context::default();
        let mut app = app();

        app.handle_open_url(&ctx, url.clone());
        wait_for_url_load(&mut app, &ctx)?;
        let path = assert_browser_document(&app, &url)?;
        let viewport = wait_for_browser_frame(&mut app, &ctx)?;
        assert!(viewport.0 > 0.0 && viewport.1 > 0.0);

        app.handle_action_refresh_document(&ctx, true);
        wait_for_url_load(&mut app, &ctx)?;
        assert_eq!(assert_browser_document(&app, &url)?, path);
        server.join().map_err(|_| "HTML server panicked")??;
        Ok(())
    }

    #[test]
    fn response_uses_final_redirect_url_as_origin() -> TestResult {
        let request = ValidatedHttpUrl::parse("https://example.test/start").expect("request url");
        let response = ehttp::Response {
            url: "https://example.test/final.html".to_string(),
            ok: true,
            status: 200,
            status_text: "status".to_string(),
            headers: ehttp::Headers::new(&[("content-type", "text/html; charset=utf-8")]),
            bytes: b"<html></html>".to_vec(),
        };

        let source = request
            .process_response(Ok(response))
            .expect("process response");
        let crate::state::FetchedUrlSource::Html(source) = source else {
            return Err("expected HTML source".into());
        };

        assert_eq!(source.origin, "https://example.test/final.html");
        assert_eq!(source.source_url, "https://example.test/final.html");
        Ok(())
    }

    #[test]
    fn loopback_html_redirect_sets_origin_to_final_url() -> TestResult {
        let (url, request_log, server) = html_server_with_redirect()?;
        let final_url = format!("{url}/final");
        let ctx = egui::Context::default();
        let mut app = app();

        app.handle_open_url(&ctx, format!("{url}/start"));
        wait_for_all_url_loads(&mut app, &ctx)?;

        let active = app.state.active_path().ok_or("no active path")?;
        let source = app
            .state
            .url_tab
            .source_for_document(&active)
            .ok_or("source missing")?;
        assert_eq!(source.origin, final_url);
        assert_eq!(source.source_url, final_url);

        wait_for_logged_requests(
            std::time::Duration::from_secs(2),
            &request_log,
            &["/start", "/final"],
        )?;

        server.join().map_err(|_| "redirect server panicked")??;
        Ok(())
    }

    #[test]
    fn remote_document_path_is_stable_and_isolated_from_local_files() {
        let first = remote_document_path("https://example.com/docs", "html");
        let second = remote_document_path("https://example.com/docs", "html");

        assert_eq!(first, second);
        assert!(first.to_string_lossy().starts_with("Katana://URL/"));
        assert_eq!(
            first.extension().and_then(|extension| extension.to_str()),
            Some("html")
        );
    }

    #[test]
    fn failed_document_identity_preserves_local_paths_and_isolates_remote_urls() -> TestResult {
        let directory = tempfile::tempdir()?;
        let local = directory.path().join("missing report.PDF");
        let local_url = url::Url::from_file_path(&local).map_err(|_| "local URL")?;
        let (local_format, local_path) =
            failed_document_identity(local_url.as_str()).ok_or("local identity")?;
        assert_eq!(
            local_format,
            katana_core::document_source::BinaryDocumentFormat::Pdf
        );
        assert_eq!(local_path, local);

        let (remote_format, remote_path) =
            failed_document_identity("https://example.test/deck.pptx?download=1")
                .ok_or("remote identity")?;
        assert_eq!(
            remote_format,
            katana_core::document_source::BinaryDocumentFormat::Pptx
        );
        assert!(remote_path.to_string_lossy().starts_with("Katana://URL/"));
        assert_eq!(
            remote_path.extension().and_then(|value| value.to_str()),
            Some("pptx")
        );
        assert!(failed_document_identity("https://example.test/page.html").is_none());
        Ok(())
    }

    #[test]
    fn user_entered_file_url_opens_the_local_html_browser_session() -> TestResult {
        let _runtime_guard = crate::preview_pane::html_browser_runtime_test_guard();
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("local document.html");
        std::fs::write(
            &path,
            "<html><body style=\"background:#123456\"><p>Local document</p></body></html>",
        )?;
        let mut url = url::Url::from_file_path(&path).map_err(|_| "file URL")?;
        url.set_query(Some("slide=2"));
        url.set_fragment(Some("deck"));
        let canonical_path = path.canonicalize()?;
        let mut canonical_url =
            url::Url::from_file_path(&canonical_path).map_err(|_| "canonical file URL")?;
        canonical_url.set_query(url.query());
        canonical_url.set_fragment(url.fragment());
        let ctx = egui::Context::default();
        let mut app = app();
        app.state.workspace.data = Some(katana_core::workspace::Workspace::new(
            directory.path(),
            Vec::new(),
        ));
        let (sender, receiver) = std::sync::mpsc::channel();
        sender.send(Ok(crate::state::FetchedUrlSource::Html(
            crate::state::HtmlSource {
                raw_html: "<p>stale remote response</p>".to_string(),
                source_url: "https://example.test/stale.html".to_string(),
                origin: "https://example.test/stale.html".to_string(),
            },
        )))?;
        app.state
            .url_tab
            .pending_url_requests
            .push_back(crate::state::PendingUrlRequest {
                response_rx: receiver,
                target_document: None,
                source_url: "https://example.test/stale.html".to_string(),
                deadline: std::time::Instant::now() + URL_SOURCE_TIMEOUT,
            });
        app.state.url_tab.is_loading = true;

        app.handle_open_url(&ctx, "ftp://example.test/index.html".to_string());
        app.handle_open_url(&ctx, url.to_string());
        app.poll_url_source(&ctx);
        let viewport = wait_for_browser_frame(&mut app, &ctx)?;

        assert_eq!(
            app.state.active_path().as_deref(),
            Some(canonical_path.as_path())
        );
        assert_eq!(app.state.url_tab.input, canonical_url.as_str());
        assert_eq!(app.state.url_tab.last_error, None);
        assert_eq!(
            app.html_browser_origin_for_test().as_deref(),
            Some(canonical_url.as_str())
        );
        assert_eq!(
            app.state
                .url_tab
                .source_for_document(&canonical_path)
                .map(|source| source.origin.as_str()),
            Some(canonical_url.as_str())
        );
        assert!(viewport.0 > 0.0 && viewport.1 > 0.0);
        assert!(app.state.url_tab.pending_url_requests.is_empty());
        assert_eq!(app.state.url_tab.tabs.len(), 1);
        assert!(!app.state.url_tab.is_loading);
        assert_eq!(app.state.layout.status_message, None);
        Ok(())
    }

    #[test]
    fn invalid_url_and_disconnected_request_are_visible_as_status_errors() {
        let ctx = egui::Context::default();
        let mut app = app();

        app.handle_open_url(&ctx, "ftp://example.com/index.html".to_string());
        assert!(matches!(
            app.state.url_tab.last_error,
            Some(crate::state::HtmlSourceError::InvalidUrl(_))
        ));
        assert!(matches!(
            app.state.layout.status_message,
            Some((_, crate::app_state::StatusType::Error))
        ));

        let (sender, receiver) = std::sync::mpsc::channel();
        drop(sender);
        app.state
            .url_tab
            .pending_url_requests
            .push_back(crate::state::PendingUrlRequest {
                response_rx: receiver,
                target_document: None,
                source_url: "https://example.test/disconnected.pdf".to_string(),
                deadline: std::time::Instant::now() + URL_SOURCE_TIMEOUT,
            });
        app.poll_url_source(&ctx);
        assert!(matches!(
            app.state.url_tab.last_error,
            Some(crate::state::HtmlSourceError::Network(_))
        ));
    }

    #[test]
    fn expired_document_request_reports_source_url_and_recovers_loading_state() {
        let ctx = egui::Context::default();
        let mut app = app();
        let (_sender, receiver) = std::sync::mpsc::channel();
        let source_url = "https://example.test/slow.pdf";
        app.state
            .url_tab
            .pending_url_requests
            .push_back(crate::state::PendingUrlRequest {
                response_rx: receiver,
                target_document: None,
                source_url: source_url.to_string(),
                deadline: std::time::Instant::now(),
            });
        app.state.url_tab.input = source_url.to_string();
        app.state.url_tab.is_loading = true;

        app.poll_url_source(&ctx);

        assert_eq!(
            app.state.url_tab.last_error,
            Some(crate::state::HtmlSourceError::Timeout {
                url: source_url.to_string(),
                seconds: URL_SOURCE_TIMEOUT.as_secs(),
            })
        );
        assert!(app.state.url_tab.pending_url_requests.is_empty());
        assert!(!app.state.url_tab.is_loading);
        let details = app.document_failure_for_test().expect("document failure");
        assert!(details.contains("Layer: source intake"));
        assert!(details.contains("Operation: fetch"));
        assert!(details.contains("Format: pdf"));
        assert!(details.contains(source_url));
        assert!(details.contains("timed out after 30 seconds"));
    }

    #[test]
    fn failed_document_url_opens_structured_source_intake_diagnostics() -> TestResult {
        let directory = tempfile::tempdir()?;
        let missing = directory.path().join("missing.pdf");
        let url = url::Url::from_file_path(&missing).map_err(|_| "file URL")?;
        let ctx = egui::Context::default();
        let mut app = app();

        app.handle_open_url(&ctx, url.to_string());

        let details = app.document_failure_for_test().ok_or("document failure")?;
        assert!(details.contains("Layer: source intake"));
        assert!(details.contains("Operation: fetch"));
        assert!(details.contains("Format: pdf"));
        assert!(details.contains("missing.pdf"));
        assert!(details.contains("Cause: Local file error"));
        assert_eq!(
            app.state.active_view_mode(),
            crate::state::document::ViewMode::PreviewOnly
        );
        Ok(())
    }

    #[test]
    fn rapid_html_url_fetches_are_queued_and_processed_in_order() -> TestResult {
        let (url, server) = html_server_with_staggered_response()?;
        let ctx = egui::Context::default();
        let mut app = app();
        let first_url = format!("{url}/first");
        let second_url = format!("{url}/second");

        app.handle_open_url(&ctx, first_url.clone());
        app.handle_open_url(&ctx, second_url.clone());
        wait_for_all_url_loads(&mut app, &ctx)?;

        let second_path = remote_document_path(&second_url, "html");
        let active = app.state.active_path();
        assert_eq!(active.as_ref(), Some(&second_path));
        assert_eq!(app.state.url_tab.tabs.len(), 2);
        let history: Vec<_> = app
            .state
            .url_tab
            .history
            .iter()
            .map(|entry| entry.as_str())
            .collect();
        assert_eq!(history, vec![second_url.as_str(), first_url.as_str()]);

        server.join().map_err(|_| "HTML server panicked")??;
        Ok(())
    }

    #[test]
    fn loopback_html_resources_are_requested_for_html_page() -> TestResult {
        let _runtime_guard = crate::preview_pane::html_browser_runtime_test_guard();
        let (url, request_log, server) = html_server_with_resource_requests()?;
        let ctx = egui::Context::default();
        let mut app = app();

        app.handle_open_url(&ctx, format!("{url}/page"));
        wait_for_all_url_loads(&mut app, &ctx)?;
        start_pending_browser_sessions(&mut app)?;

        let active = app.state.active_path().unwrap();
        let source = app
            .state
            .url_tab
            .source_for_document(&active)
            .ok_or("source missing")?;
        assert_eq!(source.origin, format!("{url}/page"));

        wait_for_logged_requests(
            std::time::Duration::from_secs(2),
            &request_log,
            &[
                "/page",
                "/assets/style.css",
                "/assets/app.js",
                "/assets/logo.png",
            ],
        )?;

        server.join().map_err(|_| "HTML server panicked")??;
        Ok(())
    }

    type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

    fn app() -> KatanaApp {
        let state = crate::app_state::AppState::new(
            katana_core::ai::AiProviderRegistry::new(),
            katana_core::plugin::PluginRegistry::new(),
            katana_platform::SettingsService::default(),
            Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        KatanaApp::new(state)
    }

    fn html_server() -> TestResult<(String, thread::JoinHandle<std::io::Result<()>>)> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let url = format!("http://{}/document.html", listener.local_addr()?);
        let server = thread::spawn(move || serve_html(listener));
        Ok((url, server))
    }

    fn html_server_with_staggered_response()
    -> TestResult<(String, thread::JoinHandle<std::io::Result<()>>)> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let url = format!("http://{}", listener.local_addr()?);
        let server = thread::spawn(move || serve_staggered_html(listener));
        Ok((url, server))
    }

    fn html_server_with_resource_requests() -> TestResult<(
        String,
        std::sync::Arc<std::sync::Mutex<Vec<String>>>,
        thread::JoinHandle<std::io::Result<()>>,
    )> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let url = format!("http://{}", listener.local_addr()?);
        let request_log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let server_log = request_log.clone();
        let server = thread::spawn(move || serve_html_with_resources(listener, server_log));
        Ok((url, request_log, server))
    }

    fn html_server_with_redirect() -> TestResult<(
        String,
        std::sync::Arc<std::sync::Mutex<Vec<String>>>,
        thread::JoinHandle<std::io::Result<()>>,
    )> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let url = format!("http://{}", listener.local_addr()?);
        let request_log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let server_log = request_log.clone();
        let server = thread::spawn(move || serve_html_with_redirect(listener, server_log));
        Ok((url, request_log, server))
    }

    fn serve_staggered_html(listener: TcpListener) -> std::io::Result<()> {
        let html = |path: &str| match path {
            "/first" => "<html><body><p>first</p></body></html>",
            "/second" => "<html><body><p>second</p></body></html>",
            _ => "<html><body><p>late</p></body></html>",
        };
        for _ in 0..RESPONSE_COUNT {
            let (mut stream, _) = listener.accept()?;
            let mut request = [0; 1024];
            let read = stream.read(&mut request)?;
            let path = parse_request_path(&String::from_utf8_lossy(&request[..read]))
                .unwrap_or_else(|| "/".to_string());
            if path == "/first" {
                std::thread::sleep(Duration::from_millis(80));
            }
            write_html_response(
                &mut stream,
                200,
                "text/html; charset=utf-8",
                html(&path).as_bytes().to_vec(),
            )?;
        }
        Ok(())
    }

    fn serve_html_with_resources(
        listener: TcpListener,
        request_log: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    ) -> std::io::Result<()> {
        listener.set_nonblocking(true)?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        let mut request_count = 0usize;
        while request_count < 20 && std::time::Instant::now() < deadline {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream.set_nonblocking(false)?;
                    request_count += 1;
                    let mut raw_request = vec![0; 2048];
                    let read = stream.read(&mut raw_request)?;
                    let path = parse_request_path(&String::from_utf8_lossy(&raw_request[..read]));
                    if let Some(path) = &path {
                        request_log
                            .lock()
                            .expect("request log mutex poisoned")
                            .push(path.clone());
                    }

                    let path = path.unwrap_or_else(|| "/".to_string());
                    match path.as_str() {
                        "/page" => {
                            let html = [
                                "<html><body>",
                                "<link rel=\"stylesheet\" href=\"/assets/style.css\">",
                                "<script src=\"/assets/app.js\"></script>",
                                "<img src=\"/assets/logo.png\" alt=\"logo\"/>",
                                "<a href=\"/next\">next</a>",
                                "</body></html>",
                            ]
                            .concat();
                            write_html_response(
                                &mut stream,
                                200,
                                "text/html; charset=utf-8",
                                html.into_bytes(),
                            )?;
                        }
                        "/next" => {
                            write_html_response(
                                &mut stream,
                                200,
                                "text/html; charset=utf-8",
                                "<html><body>next</body></html>".to_string().into_bytes(),
                            )?;
                        }
                        "/assets/style.css" => {
                            write_html_response(
                                &mut stream,
                                200,
                                "text/css; charset=utf-8",
                                "body { color: #111; }".to_string().into_bytes(),
                            )?;
                        }
                        "/assets/app.js" => {
                            write_html_response(
                                &mut stream,
                                200,
                                "application/javascript; charset=utf-8",
                                "console.log('ok');".to_string().into_bytes(),
                            )?;
                        }
                        "/assets/logo.png" => {
                            write_binary_response(&mut stream, vec![0x89, 0x50, 0x4e, 0x47])?;
                        }
                        _ => {
                            write_html_response(
                                &mut stream,
                                404,
                                "text/plain; charset=utf-8",
                                "not found".as_bytes().to_vec(),
                            )?;
                        }
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn serve_html_with_redirect(
        listener: TcpListener,
        request_log: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    ) -> std::io::Result<()> {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept()?;
            let mut request = [0; 2048];
            let read = stream.read(&mut request)?;
            let path = parse_request_path(&String::from_utf8_lossy(&request[..read]));
            if let Some(path) = &path {
                request_log
                    .lock()
                    .expect("request log mutex poisoned")
                    .push(path.clone());
            }
            match path.as_deref() {
                Some("/start") => {
                    write!(
                        &mut stream,
                        "HTTP/1.1 302 Found\r\nLocation: /final\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    )?;
                }
                Some("/final") => {
                    write_html_response(
                        &mut stream,
                        200,
                        "text/html; charset=utf-8",
                        "<html><body>final</body></html>".to_string().into_bytes(),
                    )?;
                }
                _ => {
                    write_html_response(
                        &mut stream,
                        404,
                        "text/plain; charset=utf-8",
                        b"not found".to_vec(),
                    )?;
                }
            }
        }
        Ok(())
    }

    fn parse_request_path(request: &str) -> Option<String> {
        request
            .lines()
            .next()
            .and_then(|request_line| request_line.split_whitespace().nth(1).map(str::to_string))
            .map(|path| path.split('?').next().unwrap_or_default().to_string())
    }

    fn serve_html(listener: TcpListener) -> std::io::Result<()> {
        for _ in 0..RESPONSE_COUNT {
            let (mut stream, _) = listener.accept()?;
            let mut request = [0; 1024];
            stream.read(&mut request)?;
            write_html_response(
                &mut stream,
                200,
                "text/html; charset=utf-8",
                HTML.as_bytes().to_vec(),
            )?;
        }
        Ok(())
    }

    fn write_html_response(
        stream: &mut TcpStream,
        status: u16,
        content_type: &str,
        body: Vec<u8>,
    ) -> std::io::Result<()> {
        write!(
            stream,
            "HTTP/1.1 {status} OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )?;
        stream.write_all(&body)
    }

    fn write_binary_response(stream: &mut TcpStream, body: Vec<u8>) -> std::io::Result<()> {
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )?;
        stream.write_all(&body)
    }

    fn wait_for_url_load(app: &mut KatanaApp, ctx: &egui::Context) -> TestResult {
        wait_for_all_url_loads(app, ctx)
    }

    fn wait_for_all_url_loads(app: &mut KatanaApp, ctx: &egui::Context) -> TestResult {
        let deadline = Instant::now() + URL_LOAD_TIMEOUT;
        while app.state.url_tab.is_loading {
            app.poll_url_source(ctx);
            for preview in app.tab_previews.iter_mut() {
                preview.pane.poll_html_browser(ctx);
            }
            if Instant::now() >= deadline {
                return Err("timed out waiting for the HTML URL response".into());
            }
            thread::sleep(Duration::from_millis(5));
        }
        app.poll_url_source(ctx);
        for preview in app.tab_previews.iter_mut() {
            preview.pane.poll_html_browser(ctx);
        }
        if let Some(error) = &app.state.url_tab.last_error {
            return Err(format!("HTML URL load failed: {error}").into());
        }
        Ok(())
    }

    fn wait_for_browser_frame(app: &mut KatanaApp, ctx: &egui::Context) -> TestResult<(f32, f32)> {
        start_pending_browser_sessions(app)?;
        app.wait_for_html_browser_frame_for_test(ctx, BROWSER_UPDATE_TIMEOUT)?;
        assert!(app.html_browser_frame_generation_for_test().is_some());
        app.html_browser_frame_viewport_for_test()
            .ok_or_else(|| "HTML browser update did not contain a frame".into())
    }

    fn start_pending_browser_sessions(app: &mut KatanaApp) -> TestResult {
        let viewport =
            katana_document_viewer::browser_session::HtmlBrowserViewport::new(1024, 768, 1.0)?;
        for preview in app.tab_previews.iter_mut() {
            preview.pane.start_html_browser_for_test(viewport);
        }
        Ok(())
    }

    fn wait_for_logged_requests(
        timeout: Duration,
        request_log: &std::sync::Arc<std::sync::Mutex<Vec<String>>>,
        required: &[&str],
    ) -> TestResult {
        let deadline = Instant::now() + timeout;
        loop {
            {
                let requests = request_log.lock().map_err(|error| error.to_string())?;
                if required
                    .iter()
                    .all(|path| requests.iter().any(|entry| entry == path))
                {
                    return Ok(());
                }
            }
            if Instant::now() >= deadline {
                return Err(format!(
                    "timed out waiting for browser resource requests: {:?}",
                    *request_log.lock().map_err(|error| error.to_string())?
                )
                .into());
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn assert_browser_document(app: &KatanaApp, url: &str) -> TestResult<std::path::PathBuf> {
        let path = app
            .state
            .active_path()
            .ok_or("HTML URL did not become the active document")?;
        let source = app
            .state
            .url_tab
            .source_for_document(&path)
            .ok_or("active HTML URL source is missing")?;
        let pane = &app
            .tab_previews
            .iter()
            .find(|preview| preview.path == path)
            .ok_or("active HTML URL preview is missing")?
            .pane;

        assert_eq!(source.raw_html, HTML);
        assert_eq!(source.origin, url);
        assert!(pane.has_html_browser());
        assert!(pane.sections.is_empty());
        Ok(path)
    }
}
