use super::types::*;

impl PreviewPane {
    pub fn update_html_document_sections(
        &mut self,
        source: &str,
        html_file_path: &std::path::Path,
    ) {
        let current_origin = self.html_browser_origin();
        self.md_file_path = html_file_path.to_path_buf();
        self.outline_items.clear();
        self.anchor_map.clear();
        self.document_anchors.clear();
        self.replace_html_browser(source, html_file_path, current_origin.as_deref());
    }

    pub fn full_render_html_document(
        &mut self,
        source: &str,
        html_file_path: &std::path::Path,
        force: bool,
    ) {
        let current_origin = self.html_browser_origin();
        if force {
            self.viewer_states.clear();
            self.fullscreen_viewer_state.reset();
            self.fullscreen_image = None;
        }

        self.md_file_path = html_file_path.to_path_buf();
        self.outline_items.clear();
        self.anchor_map.clear();
        self.document_anchors.clear();
        self.reset_html_document_render_state();
        self.session_generation += 1;
        self.replace_html_browser(source, html_file_path, current_origin.as_deref());
    }

    fn reset_html_document_render_state(&mut self) {
        self.image_preload_queue.clear();
        self.image_cache.clear();
        self.render_rx = None;
        self.is_loading = false;
        self.cancel_token
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.cancel_token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    }

    fn replace_html_browser(
        &mut self,
        source: &str,
        html_file_path: &std::path::Path,
        current_origin: Option<&str>,
    ) {
        self.sections.clear();
        self.section_lifecycle.clear();
        match local_html_source(source, html_file_path, current_origin) {
            Ok(source) => self.navigate_or_start_html_browser(source),
            Err(error) => {
                self.html_browser =
                    Some(super::image_html_surface::HtmlBrowserSurface::failed(error));
            }
        }
    }

    pub(crate) fn full_render_html_source(
        &mut self,
        source: katana_document_viewer::browser_session::HtmlBrowserSource,
        force: bool,
    ) {
        if force {
            self.viewer_states.clear();
            self.fullscreen_viewer_state.reset();
            self.fullscreen_image = None;
        }
        self.outline_items.clear();
        self.anchor_map.clear();
        self.document_anchors.clear();
        self.reset_html_document_render_state();
        self.session_generation += 1;
        self.navigate_or_start_html_browser(source);
    }

    fn navigate_or_start_html_browser(
        &mut self,
        source: katana_document_viewer::browser_session::HtmlBrowserSource,
    ) {
        if let Some(browser) = &mut self.html_browser {
            browser.navigate(source);
        } else {
            self.html_browser = Some(super::image_html_surface::HtmlBrowserSurface::start(source));
        }
    }
}

fn local_html_source(
    raw_html: &str,
    document_path: &std::path::Path,
    current_origin: Option<&str>,
) -> Result<katana_document_viewer::browser_session::HtmlBrowserSource, String> {
    let path = document_path
        .canonicalize()
        .map_err(|error| format!("failed to canonicalize HTML document path: {error}"))?;
    let default_origin = url::Url::from_file_path(&path)
        .map_err(|_| "failed to construct a file URL for the HTML document".to_string())?;
    let origin = current_origin
        .and_then(|origin| matching_local_origin(origin, &path))
        .unwrap_or(default_origin);
    katana_document_viewer::browser_session::HtmlBrowserSource::new(raw_html, origin.as_str())
        .map_err(|error| error.to_string())
}

fn matching_local_origin(origin: &str, document_path: &std::path::Path) -> Option<url::Url> {
    let url = url::Url::parse(origin).ok()?;
    if url.scheme() != "file" {
        return None;
    }
    let mut path_url = url.clone();
    path_url.set_query(None);
    path_url.set_fragment(None);
    let origin_path = path_url.to_file_path().ok()?.canonicalize().ok()?;
    (origin_path == document_path).then_some(url)
}

#[cfg(test)]
mod tests {
    use super::{PreviewPane, local_html_source};

    #[test]
    fn local_html_source_keeps_the_canonical_file_origin() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("browser document.html");
        std::fs::write(&path, "<p>browser</p>").expect("fixture document");

        let source = local_html_source("<p>browser</p>", &path, None).expect("browser source");

        assert_eq!(source.raw_html, "<p>browser</p>");
        assert!(source.origin.as_str().starts_with("file://"));
        assert!(source.origin.as_str().contains("browser%20document.html"));
    }

    #[test]
    fn local_html_source_preserves_only_a_matching_file_fragment_origin() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("index.html");
        let other = directory.path().join("other.html");
        std::fs::write(&path, "<p>browser</p>").expect("fixture document");
        std::fs::write(&other, "<p>other</p>").expect("other fixture");
        let current = file_origin(&path, "#target");
        let unrelated = file_origin(&other, "#other");

        let preserved =
            local_html_source("<p>browser</p>", &path, Some(&current)).expect("preserved source");
        let replaced =
            local_html_source("<p>browser</p>", &path, Some(&unrelated)).expect("replaced source");

        assert!(preserved.origin.as_str().ends_with("index.html#target"));
        assert!(replaced.origin.as_str().ends_with("index.html"));
    }

    fn file_origin(path: &std::path::Path, fragment: &str) -> String {
        let url = url::Url::from_file_path(path).expect("file URL");
        format!("{}{fragment}", url.as_str())
    }

    #[test]
    fn html_document_uses_the_kdv_krr_browser_surface_without_static_sections() {
        let directory = tempfile::tempdir().expect("temporary HTML workspace");
        let path = directory.path().join("index.html");
        let html = "<html><body><details><summary>More</summary><button>Run</button></details></body></html>";
        std::fs::write(&path, html).expect("HTML fixture");
        let mut pane = PreviewPane::default();

        pane.update_html_document_sections(html, &path);

        assert!(pane.has_html_browser());
        assert!(pane.sections.is_empty());
    }
}
