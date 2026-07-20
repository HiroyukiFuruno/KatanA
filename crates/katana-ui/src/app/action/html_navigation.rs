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
        let navigation = self
            .tab_previews
            .iter_mut()
            .find(|preview| preview.path == active_path)
            .and_then(|preview| preview.pane.take_html_browser_navigation());
        let Some(navigation) = navigation else {
            return;
        };
        self.handle_html_navigation(ctx, active_path, navigation);
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
        let source = std::fs::read_to_string(&path)
            .map_err(|error| error.to_string())
            .and_then(|raw_html| {
                katana_document_viewer::browser_session::HtmlBrowserSource::new(
                    raw_html.clone(),
                    url.as_str(),
                )
                .map(|source| (raw_html, source))
                .map_err(|error| error.to_string())
            });
        match source {
            Ok((raw_html, browser_source)) => self.replace_html_document_with_source(
                HtmlSource {
                    raw_html,
                    source_url: url.to_string(),
                    origin: url.to_string(),
                },
                path,
                Some(active_path),
                browser_source,
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
        let browser_source = match katana_document_viewer::browser_session::HtmlBrowserSource::new(
            source.raw_html.clone(),
            source.origin.clone(),
        ) {
            Ok(source) => source,
            Err(error) => {
                self.state.layout.status_message = Some((error.to_string(), StatusType::Error));
                return;
            }
        };
        self.replace_html_document_with_source(
            source,
            document_path,
            previous_path,
            browser_source,
        );
    }

    fn replace_html_document_with_source(
        &mut self,
        source: HtmlSource,
        document_path: std::path::PathBuf,
        previous_path: Option<std::path::PathBuf>,
        browser_source: katana_document_viewer::browser_session::HtmlBrowserSource,
    ) {
        let document_index = previous_path
            .as_ref()
            .and_then(|path| {
                self.state
                    .document
                    .open_documents
                    .iter()
                    .position(|document| document.path == *path)
            })
            .or_else(|| {
                self.state
                    .document
                    .open_documents
                    .iter()
                    .position(|document| document.path == document_path)
            });
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

    fn preview(path: std::path::PathBuf) -> TabPreviewCache {
        TabPreviewCache {
            path,
            pane: PreviewPane::default(),
            hash: 0,
        }
    }

    fn source(url: &str) -> HtmlSource {
        HtmlSource {
            raw_html: "<html><body>browser</body></html>".to_owned(),
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
}
