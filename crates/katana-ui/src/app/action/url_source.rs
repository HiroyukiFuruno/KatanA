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
        self.state.url_tab.response_rx = Some(receiver);
        self.state.url_tab.pending_document_path = target_document;
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
        let Some(receiver) = &self.state.url_tab.response_rx else {
            return;
        };

        match receiver.try_recv() {
            Ok(Ok(source)) => {
                let target = self.state.url_tab.pending_document_path.take();
                self.apply_fetched_html_source(source, target);
                self.state.layout.status_message = None;
                self.state.url_tab.response_rx = None;
            }
            Ok(Err(error)) => {
                self.state.url_tab.pending_document_path = None;
                self.fail_html_source(error);
                self.state.url_tab.response_rx = None;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.state.url_tab.pending_document_path = None;
                self.fail_html_source(HtmlSourceError::Network(
                    "URL request channel disconnected".to_string(),
                ));
                self.state.url_tab.response_rx = None;
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

        app.handle_action_refresh_document(&ctx, true);
        wait_for_url_load(&mut app, &ctx)?;
        assert_eq!(assert_browser_document(&app, &url)?, path);
        server.join().map_err(|_| "HTML server panicked")??;
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
        app.state.url_tab.response_rx = Some(receiver);
        app.poll_url_source(&ctx);
        assert!(matches!(
            app.state.url_tab.last_error,
            Some(crate::state::HtmlSourceError::Network(_))
        ));
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

    fn serve_html(listener: TcpListener) -> std::io::Result<()> {
        for _ in 0..RESPONSE_COUNT {
            let (mut stream, _) = listener.accept()?;
            write_html_response(&mut stream)?;
        }
        Ok(())
    }

    fn write_html_response(stream: &mut TcpStream) -> std::io::Result<()> {
        let mut request = [0; 1024];
        let _ = stream.read(&mut request)?;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{HTML}",
            HTML.len()
        )
    }

    fn wait_for_url_load(app: &mut KatanaApp, ctx: &egui::Context) -> TestResult {
        let deadline = Instant::now() + URL_LOAD_TIMEOUT;
        while app.state.url_tab.is_loading {
            app.poll_url_source(ctx);
            if Instant::now() >= deadline {
                return Err("timed out waiting for the HTML URL response".into());
            }
            thread::sleep(Duration::from_millis(5));
        }
        app.poll_url_source(ctx);
        Ok(())
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
