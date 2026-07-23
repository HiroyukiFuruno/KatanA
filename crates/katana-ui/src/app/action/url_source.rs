use crate::app::url_source::ValidatedHttpUrl;
use crate::app_state::StatusType;
use crate::shell::KatanaApp;
use crate::state::{HtmlSource, HtmlSourceError};

const URL_DOCUMENT_PREFIX: &str = "Katana://URL";

impl KatanaApp {
    pub(super) fn handle_open_url(&mut self, ctx: &egui::Context, input: String) {
        let url = match ValidatedHttpUrl::parse(&input) {
            Ok(url) => url,
            Err(error) => {
                self.fail_html_source(HtmlSourceError::InvalidUrl(error));
                return;
            }
        };

        self.fetch_html_url(ctx, url, None);
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
            .push_back((receiver, target_document));
        self.state.url_tab.input = url.as_str().to_string();
        self.state.url_tab.is_loading = true;
        self.state.url_tab.last_error = None;

        let request_url = url.clone();
        let repaint = ctx.clone();
        ehttp::fetch(ehttp::Request::get(url.as_str()), move |response| {
            let _ = sender.send(request_url.process_response(response));
            repaint.request_repaint_after(std::time::Duration::ZERO);
        });
    }

    pub(super) fn apply_fetched_html_source(
        &mut self,
        source: HtmlSource,
        target_document: Option<std::path::PathBuf>,
    ) {
        let document_path =
            target_document.unwrap_or_else(|| remote_document_path(&source.source_url));
        self.state
            .url_tab
            .open_source(source.clone(), document_path.clone());
        self.replace_html_document(source, document_path, None);
    }

    pub(crate) fn poll_url_source(&mut self, _ctx: &egui::Context) {
        while let Some((response_rx, target_document)) =
            self.state.url_tab.pending_url_requests.pop_front()
        {
            match response_rx.try_recv() {
                Ok(Ok(source)) => {
                    self.apply_fetched_html_source(source, target_document);
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
                    self.state
                        .url_tab
                        .pending_url_requests
                        .push_front((response_rx, target_document));
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

    pub(super) fn fail_html_source(&mut self, error: HtmlSourceError) {
        self.state.layout.status_message = Some((error.to_string(), StatusType::Error));
        self.state.url_tab.fail(error);
    }
}

fn remote_document_path(source_url: &str) -> std::path::PathBuf {
    let hash = katana_core::document::DocumentOps::compute_hash(source_url);
    std::path::PathBuf::from(format!("{URL_DOCUMENT_PREFIX}/{hash:x}.html"))
}

#[cfg(test)]
mod tests {
    use super::remote_document_path;
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

    #[test]
    fn user_entered_http_document_keeps_origin_through_refresh_and_browser_session() -> TestResult {
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
        let first = remote_document_path("https://example.com/docs");
        let second = remote_document_path("https://example.com/docs");

        assert_eq!(first, second);
        assert!(first.to_string_lossy().starts_with("Katana://URL/"));
        assert_eq!(
            first.extension().and_then(|extension| extension.to_str()),
            Some("html")
        );
    }

    #[test]
    fn invalid_url_and_disconnected_request_are_visible_as_status_errors() {
        let ctx = egui::Context::default();
        let mut app = app();

        app.handle_open_url(&ctx, "file:///tmp/index.html".to_string());
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
            .push_back((receiver, None));
        app.poll_url_source(&ctx);
        assert!(matches!(
            app.state.url_tab.last_error,
            Some(crate::state::HtmlSourceError::Network(_))
        ));
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

        let second_path = remote_document_path(&second_url);
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
        let (url, request_log, server) = html_server_with_resource_requests()?;
        let ctx = egui::Context::default();
        let mut app = app();

        app.handle_open_url(&ctx, format!("{url}/page"));
        wait_for_all_url_loads(&mut app, &ctx)?;

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
        let deadline = Instant::now() + URL_LOAD_TIMEOUT;
        loop {
            for preview in app.tab_previews.iter_mut() {
                preview.pane.poll_html_browser(ctx);
            }
            if let Some(viewport) = app.html_browser_frame_viewport_for_test() {
                return Ok(viewport);
            }
            if Instant::now() >= deadline {
                return Err("timed out waiting for the HTML browser frame".into());
            }
            thread::sleep(Duration::from_millis(5));
        }
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
