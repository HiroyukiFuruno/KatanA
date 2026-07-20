use super::HtmlBrowserSurface;
use katana_document_viewer::browser_session::{HtmlBrowserNavigation, HtmlBrowserSource};

const MAX_NAVIGATION_HISTORY: usize = 100;

impl HtmlBrowserSurface {
    pub(crate) fn navigate(&mut self, source: HtmlBrowserSource) {
        let origin = source.origin.as_str().to_owned();
        let Some(adapter) = self.adapter.as_ref() else {
            *self = Self::start(source);
            return;
        };
        let result = HtmlBrowserNavigation::new(source)
            .map_err(|error| error.to_string())
            .and_then(|navigation| {
                adapter
                    .navigate(navigation)
                    .map_err(|error| error.to_string())
            });
        if let Err(error) = result {
            self.record_error(error);
            return;
        }
        self.pending_navigation_url = None;
        self.error = None;
        self.record_navigation(origin);
        self.await_frame();
    }

    pub(super) fn navigation_history(&self) -> Vec<String> {
        self.navigation_history.iter().cloned().collect()
    }

    pub(super) fn record_navigation(&mut self, origin: String) {
        if self.navigation_history.back() == Some(&origin) {
            return;
        }
        self.navigation_history.push_back(origin);
        while self.navigation_history.len() > MAX_NAVIGATION_HISTORY {
            self.navigation_history.pop_front();
        }
    }
}
