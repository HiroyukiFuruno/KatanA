use super::HtmlBrowserSurface;
use crate::preview_pane::DocumentWorkerLifecycle;

#[cfg(test)]
#[path = "image_html_surface_close_tests.rs"]
mod tests;

impl Drop for HtmlBrowserSurface {
    fn drop(&mut self) {
        let Some(mut adapter) = self.adapter.take() else {
            return;
        };
        /* WHY: KDVのcloseはworkerをjoinするため、タブ破棄のUIスレッドでは待たない。 */
        if let Err(error) =
            DocumentWorkerLifecycle::spawn("katana-html-preview-close".to_owned(), move || {
                if let Err(error) = adapter.close() {
                    tracing::error!(%error, "HTML browser cleanup failed");
                }
            })
        {
            tracing::error!(%error, "HTML browser cleanup worker could not start");
        }
    }
}
