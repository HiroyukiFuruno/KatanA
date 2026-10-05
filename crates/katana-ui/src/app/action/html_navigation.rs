use crate::app::PreviewOps;
use crate::app::url_source::ValidatedHttpUrl;
use crate::app_state::StatusType;
use crate::shell::KatanaApp;
use crate::state::{HtmlSource, HtmlSourceError};

impl KatanaApp {
    pub(crate) fn poll_html_browser_navigation(&mut self, ctx: &egui::Context) {
        let Some(active_path) = self.state.active_path() else {
            return;
        };
        while let Some(navigation) = self
            .tab_previews
            .iter_mut()
            .find(|preview| preview.path == active_path)
            .and_then(|preview| preview.pane.take_html_browser_navigation())
        {
            self.handle_html_navigation(ctx, active_path.clone(), navigation);
        }
    }

    fn handle_html_navigation(
        &mut self,
        ctx: &egui::Context,
        active_path: std::path::PathBuf,
        navigation_url: String,
    ) {
        let url = match url::Url::parse(&navigation_url) {
            Ok(url) => url,
            Err(error) => {
                self.state.layout.status_message = Some((
                    format!("Invalid HTML navigation URL: {error}"),
                    StatusType::Error,
                ));
                return;
            }
        };
        match url.scheme() {
            "file" => self.navigate_html_file(active_path, url),
            "http" | "https" => match ValidatedHttpUrl::parse(url.as_str()) {
                Ok(url) => self.fetch_html_url(ctx, url, Some(active_path)),
                Err(error) => self.fail_html_source(HtmlSourceError::InvalidUrl(error)),
            },
            scheme => {
                self.state.layout.status_message = Some((
                    format!("Unsupported HTML navigation scheme: {scheme}"),
                    StatusType::Error,
                ))
            }
        }
    }

    fn navigate_html_file(&mut self, active_path: std::path::PathBuf, url: url::Url) {
        let path = match url.to_file_path() {
            Ok(path) => path,
            Err(()) => {
                self.state.layout.status_message = Some((
                    "HTML navigation contained an invalid file URL".to_string(),
                    StatusType::Error,
                ));
                return;
            }
        };
        match std::fs::read_to_string(&path) {
            Ok(raw_html) => self.replace_html_document(
                HtmlSource {
                    raw_html,
                    source_url: url.to_string(),
                    origin: url.to_string(),
                },
                path,
                Some(active_path),
            ),
            Err(error) => {
                self.state.layout.status_message = Some((
                    format!("Failed to load HTML navigation: {error}"),
                    StatusType::Error,
                ))
            }
        }
    }

    pub(super) fn replace_html_document(
        &mut self,
        source: HtmlSource,
        document_path: std::path::PathBuf,
        previous_path: Option<std::path::PathBuf>,
    ) {
        self.replace_html_document_with_source(source, document_path, previous_path);
    }

    fn replace_html_document_with_source(
        &mut self,
        source: HtmlSource,
        document_path: std::path::PathBuf,
        previous_path: Option<std::path::PathBuf>,
    ) {
        let target_document_index = self
            .state
            .document
            .open_documents
            .iter()
            .position(|document| document.path == document_path);
        let document_index = target_document_index.or_else(|| {
            previous_path.as_ref().and_then(|path| {
                self.state
                    .document
                    .open_documents
                    .iter()
                    .position(|document| document.path == *path)
            })
        });
        let target_collision = target_document_index.filter(|_| {
            previous_path
                .as_ref()
                .is_some_and(|path| path != &document_path)
        });
        let browser_html = target_collision
            .map(|index| self.state.document.open_documents[index].buffer.clone())
            .unwrap_or_else(|| source.raw_html.clone());
        let browser_source = match katana_document_viewer::browser_session::HtmlBrowserSource::new(
            browser_html,
            source.origin.clone(),
        ) {
            Ok(source) => source,
            Err(error) => {
                self.state.layout.status_message = Some((error.to_string(), StatusType::Error));
                return;
            }
        };
        if let Some(index) = target_collision {
            self.state.document.active_doc_idx = Some(index);
            self.state.document.scroll_to_active_tab = true;
            self.full_refresh_html_source(&document_path, browser_source);
            return;
        }
        let was_open = document_index.is_some();
        let index = document_index.unwrap_or_else(|| {
            self.state
                .document
                .open_documents
                .push(katana_core::document::Document::new(
                    document_path.clone(),
                    source.raw_html.clone(),
                ));
            self.state.document.open_documents.len() - 1
        });
        let previous_path = self.state.document.open_documents[index].path.clone();
        let is_remote =
            source.origin.starts_with("http://") || source.origin.starts_with("https://");
        let pinned = self.state.document.open_documents[index].is_pinned;
        self.state.document.open_documents[index] =
            katana_core::document::Document::new(document_path.clone(), source.raw_html);
        self.state.document.open_documents[index].is_pinned = pinned;
        self.state.document.open_documents[index].is_reference = is_remote;
        self.state.document.active_doc_idx = Some(index);
        self.state.document.scroll_to_active_tab = true;
        if previous_path != document_path {
            self.state
                .document
                .replace_path_references(&previous_path, &document_path);
            preserve_html_preview_session(&mut self.tab_previews, &previous_path, &document_path);
        }
        if !was_open {
            self.state.initialize_tab_split_state(document_path.clone());
        }
        self.full_refresh_html_source(&document_path, browser_source);
    }
}

fn preserve_html_preview_session(
    previews: &mut Vec<crate::shell::TabPreviewCache>,
    previous_path: &std::path::Path,
    document_path: &std::path::Path,
) {
    previews.retain(|preview| preview.path != document_path);
    if let Some(preview) = previews
        .iter_mut()
        .find(|preview| preview.path == previous_path)
    {
        preview.path = document_path.to_path_buf();
    }
}

#[cfg(test)]
mod tests {
    use super::preserve_html_preview_session;
    use crate::preview_pane::PreviewPane;
    use crate::shell::KatanaApp;
    use crate::shell::TabPreviewCache;
    use crate::state::HtmlSource;
    use std::sync::Arc;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
        time::{Duration, Instant},
    };

    #[test]
    fn top_level_navigation_moves_the_existing_browser_session_to_the_target_path() {
        let previous = std::path::PathBuf::from("workspace/index.html");
        let target = std::path::PathBuf::from("workspace/linked.html");
        let mut previews = vec![preview(previous.clone()), preview(target.clone())];

        preserve_html_preview_session(&mut previews, &previous, &target);

        assert_eq!(previews.len(), 1);
        assert_eq!(previews[0].path, target);
    }

    #[test]
    fn document_navigation_preserves_one_browser_surface_and_its_tab_history() {
        let mut app = test_app();
        let initial_path = std::path::PathBuf::from("Katana://URL/initial.html");
        let target_path = std::path::PathBuf::from("Katana://URL/target.html");
        app.replace_html_document(
            source("https://example.com/initial"),
            initial_path.clone(),
            None,
        );

        app.replace_html_document(
            source("https://example.com/target"),
            target_path.clone(),
            Some(initial_path),
        );

        assert_eq!(app.tab_previews.len(), 1);
        assert_eq!(app.tab_previews[0].path, target_path);
        assert_eq!(
            app.html_browser_navigation_history_for_test(),
            Some(vec![
                "https://example.com/initial".to_owned(),
                "https://example.com/target".to_owned(),
            ])
        );
    }

    #[test]
    fn navigation_to_an_open_target_reuses_target_and_preserves_source_document() {
        use katana_platform::{PaneOrder, SplitDirection};

        let mut app = test_app();
        let source_path = std::path::PathBuf::from("Katana://URL/source.html");
        let target_path = std::path::PathBuf::from("Katana://URL/target.html");
        let source_document = source_with_body("https://example.com/source", "source");
        let target_document = source_with_body("https://example.com/target", "target");
        let navigated_target = source_with_body("https://example.com/navigated", "navigated");

        app.state
            .url_tab
            .open_source(source_document.clone(), source_path.clone());
        app.replace_html_document(source_document, source_path.clone(), None);
        app.state
            .set_active_view_mode(crate::state::ViewMode::CodeOnly);

        app.state
            .url_tab
            .open_source(target_document.clone(), target_path.clone());
        app.replace_html_document(target_document, target_path.clone(), None);
        app.state.document.open_documents[1].is_pinned = true;
        app.state
            .set_active_view_mode(crate::state::ViewMode::Split);
        app.state
            .set_active_split_direction(SplitDirection::Vertical);
        app.state.set_active_pane_order(PaneOrder::PreviewFirst);
        let target_preview_history = app
            .html_browser_navigation_history_for_test()
            .expect("target preview history");

        app.state
            .url_tab
            .open_source(navigated_target.clone(), target_path.clone());
        app.replace_html_document(
            navigated_target,
            target_path.clone(),
            Some(source_path.clone()),
        );

        assert_eq!(app.state.document.open_documents.len(), 2);
        assert_eq!(
            app.state
                .document
                .open_documents
                .iter()
                .map(|document| document.path.clone())
                .collect::<Vec<_>>(),
            vec![source_path.clone(), target_path.clone()]
        );
        assert_eq!(app.state.active_path(), Some(target_path.clone()));
        assert_eq!(
            app.state.document.open_documents[0].buffer,
            "<html><body>source</body></html>"
        );
        assert_eq!(
            app.state.document.open_documents[1].buffer,
            "<html><body>target</body></html>"
        );
        assert!(app.state.document.open_documents[1].is_pinned);
        assert_eq!(app.state.active_view_mode(), crate::state::ViewMode::Split);
        assert_eq!(app.state.active_split_direction(), SplitDirection::Vertical);
        assert_eq!(app.state.active_pane_order(), PaneOrder::PreviewFirst);
        assert_eq!(app.tab_previews.len(), 2);
        assert!(
            app.tab_previews
                .iter()
                .any(|preview| preview.path == source_path)
        );
        assert!(
            app.tab_previews
                .iter()
                .any(|preview| preview.path == target_path)
        );
        assert_eq!(
            app.state.url_tab.source_for_document(&source_path),
            Some(&HtmlSource {
                raw_html: "<html><body>source</body></html>".to_owned(),
                source_url: "https://example.com/source".to_owned(),
                origin: "https://example.com/source".to_owned(),
            })
        );
        assert_eq!(
            app.html_browser_navigation_history_for_test(),
            Some({
                let mut history = target_preview_history;
                history.push("https://example.com/navigated".to_owned());
                history
            })
        );
    }

    #[test]
    fn file_navigation_to_an_open_dirty_target_preserves_target_state() -> TestResult {
        use katana_platform::{PaneOrder, SplitDirection};

        let temporary_directory = tempfile::tempdir()?;
        let source_path = temporary_directory.path().join("source.html");
        let target_path = temporary_directory.path().join("target.html");
        std::fs::write(&source_path, "<html><body>source-file</body></html>")?;
        std::fs::write(
            &target_path,
            "<html><body style=\"background: rgb(0, 0, 255)\">target-file</body></html>",
        )?;

        let mut app = test_app();
        let source_url = url::Url::from_file_path(&source_path).map_err(|()| "source URL")?;
        let target_url = url::Url::from_file_path(&target_path).map_err(|()| "target URL")?;
        let source_document = source_with_body(source_url.as_str(), "source-document");
        let target_document = source_with_body(target_url.as_str(), "target-document");
        app.replace_html_document(source_document, source_path.clone(), None);
        app.replace_html_document(target_document, target_path.clone(), None);
        app.state.document.open_documents[1].update_buffer(
            "<html><body style=\"background: rgb(255, 0, 0)\">unsaved-target</body></html>",
        );
        app.state.document.open_documents[1].is_pinned = true;
        app.state
            .set_active_view_mode(crate::state::ViewMode::Split);
        app.state
            .set_active_split_direction(SplitDirection::Vertical);
        app.state.set_active_pane_order(PaneOrder::PreviewFirst);
        app.state.document.active_doc_idx = Some(0);
        let target_history = app
            .tab_previews
            .iter()
            .find(|preview| preview.path == target_path)
            .and_then(|preview| preview.pane.html_browser_navigation_history())
            .expect("target preview history");

        let mut target_navigation_url = target_url.clone();
        target_navigation_url.set_fragment(Some("details"));
        app.handle_html_navigation(
            &egui::Context::default(),
            source_path.clone(),
            target_navigation_url.to_string(),
        );

        assert_eq!(app.state.document.open_documents.len(), 2);
        assert_eq!(app.state.active_path(), Some(target_path.clone()));
        assert_eq!(
            app.state.document.open_documents[0].buffer,
            "<html><body>source-document</body></html>"
        );
        assert_eq!(
            app.state.document.open_documents[1].buffer,
            "<html><body style=\"background: rgb(255, 0, 0)\">unsaved-target</body></html>"
        );
        assert!(app.state.document.open_documents[1].is_dirty);
        assert!(app.state.document.open_documents[1].is_pinned);
        assert_eq!(app.state.active_split_direction(), SplitDirection::Vertical);
        assert_eq!(app.state.active_pane_order(), PaneOrder::PreviewFirst);
        assert_eq!(
            app.tab_previews
                .iter()
                .filter(|preview| preview.path == source_path || preview.path == target_path)
                .count(),
            2
        );
        assert_eq!(
            app.tab_previews
                .iter()
                .find(|preview| preview.path == target_path)
                .and_then(|preview| preview.pane.html_browser_navigation_history()),
            Some({
                let mut history = target_history;
                history.push(target_navigation_url.to_string());
                history
            })
        );
        let viewport =
            katana_document_viewer::browser_session::HtmlBrowserViewport::new(320, 240, 1.0)?;
        app.tab_previews
            .iter_mut()
            .find(|preview| preview.path == target_path)
            .ok_or("target preview missing")?
            .pane
            .start_html_browser_for_test(viewport);
        app.wait_for_html_browser_frame_for_test(
            &egui::Context::default(),
            Duration::from_secs(2),
        )?;
        assert!(
            app.html_browser_frame_matching_rgb_pixels_for_test([255, 0, 0])
                .is_some_and(|pixels| pixels > 0),
            "dirty target preview must render the preserved buffer"
        );
        assert_eq!(
            app.html_browser_origin_for_test(),
            Some(target_navigation_url.to_string())
        );
        Ok(())
    }

    #[test]
    fn same_path_reload_replaces_document_and_updates_browser_history() {
        let mut app = test_app();
        let path = std::path::PathBuf::from("Katana://URL/reload.html");
        app.replace_html_document(
            source_with_body("https://example.com/first", "first"),
            path.clone(),
            None,
        );
        app.replace_html_document(
            source_with_body("https://example.com/second", "second"),
            path.clone(),
            Some(path.clone()),
        );

        assert_eq!(app.state.document.open_documents.len(), 1);
        assert_eq!(
            app.state.document.open_documents[0].buffer,
            "<html><body>second</body></html>"
        );
        assert_eq!(app.state.active_path(), Some(path));
        assert_eq!(
            app.html_browser_navigation_history_for_test(),
            Some(vec![
                "https://example.com/first".to_owned(),
                "https://example.com/second".to_owned(),
            ])
        );
    }

    #[test]
    fn invalid_browser_origin_keeps_existing_document_and_preview() {
        let mut app = test_app();
        let path = std::path::PathBuf::from("Katana://URL/invalid-origin.html");
        app.replace_html_document(source("https://example.com/valid"), path.clone(), None);
        let document_before = app.state.document.open_documents[0].clone();
        let history_before = app
            .html_browser_navigation_history_for_test()
            .expect("existing preview history");

        app.replace_html_document(
            HtmlSource {
                raw_html: "<html><body>invalid</body></html>".to_owned(),
                source_url: "not-a-url".to_owned(),
                origin: "not-a-url".to_owned(),
            },
            path.clone(),
            Some(path.clone()),
        );

        assert_eq!(app.state.document.open_documents[0], document_before);
        assert_eq!(app.state.active_path(), Some(path));
        assert_eq!(
            app.html_browser_navigation_history_for_test(),
            Some(history_before)
        );
    }

    #[test]
    fn consecutive_navigation_intents_preserve_active_tab_and_queue_order() -> TestResult {
        let (base_url, server) = navigation_server()?;
        let mut app = test_app();
        let active_path = std::path::PathBuf::from("Katana://URL/initial.html");
        let first = format!("{base_url}/first");
        let second = format!("{base_url}/second");

        app.replace_html_document(
            source("https://example.com/initial"),
            active_path.clone(),
            None,
        );

        let ctx = egui::Context::default();
        app.handle_html_navigation(&ctx, active_path.clone(), first);
        app.handle_html_navigation(&ctx, active_path.clone(), second.clone());
        wait_for_navigation_queue(&mut app)?;

        let source = app
            .state
            .url_tab
            .source_for_document(&active_path)
            .ok_or("active source missing after navigation")?;
        assert_eq!(app.state.active_path(), Some(active_path.clone()));
        assert_eq!(source.source_url, second);
        assert_eq!(
            app.html_browser_navigation_history_for_test(),
            Some(vec![
                "https://example.com/initial".to_string(),
                format!("{base_url}/first"),
                format!("{base_url}/second"),
            ])
        );

        server.join().map_err(|_| "navigation server panicked")??;
        Ok(())
    }

    fn preview(path: std::path::PathBuf) -> TabPreviewCache {
        TabPreviewCache {
            path,
            pane: PreviewPane::default(),
            hash: 0,
        }
    }

    fn source(url: &str) -> HtmlSource {
        source_with_body(url, "browser")
    }

    fn source_with_body(url: &str, body: &str) -> HtmlSource {
        HtmlSource {
            raw_html: format!("<html><body>{body}</body></html>"),
            source_url: url.to_owned(),
            origin: url.to_owned(),
        }
    }

    fn test_app() -> KatanaApp {
        let state = crate::app_state::AppState::new(
            katana_core::ai::AiProviderRegistry::new(),
            katana_core::plugin::PluginRegistry::new(),
            katana_platform::SettingsService::default(),
            Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        KatanaApp::new(state)
    }

    type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

    fn navigation_server() -> TestResult<(String, thread::JoinHandle<std::io::Result<()>>)> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let url = format!("http://{}", listener.local_addr()?);
        let server = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept()?;
                let mut request = [0; 1024];
                let read = stream.read(&mut request)?;
                let request_line = String::from_utf8_lossy(&request[..read]).to_string();
                let path = request_line
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or("/first")
                    .to_string();
                if path == "/first" {
                    std::thread::sleep(Duration::from_millis(80));
                }
                let body = if path.contains("/first") {
                    "<html><body>first</body></html>"
                } else {
                    "<html><body>second</body></html>"
                };
                write!(
                    &mut stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                )?;
            }
            Ok(())
        });
        Ok((url, server))
    }

    fn wait_for_navigation_queue(app: &mut KatanaApp) -> TestResult {
        let ctx = egui::Context::default();
        let deadline = Instant::now() + Duration::from_secs(2);
        while app.state.url_tab.is_loading {
            app.poll_url_source(&ctx);
            app.poll_html_browser_navigation(&ctx);
            if Instant::now() >= deadline {
                return Err("timed out waiting for navigation queue".into());
            }
            thread::sleep(Duration::from_millis(5));
        }
        app.poll_url_source(&ctx);
        app.poll_html_browser_navigation(&ctx);
        Ok(())
    }
}
