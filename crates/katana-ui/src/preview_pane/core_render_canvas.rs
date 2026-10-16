use eframe::egui;

use super::PreviewPane;
use super::canvas_request::{CanvasParseRequest, CanvasRequestError};
use super::canvas_surface::{CanvasLoadError, CanvasSurfaceLabels};

const CANVAS_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(25);

impl PreviewPane {
    pub(crate) fn update_canvas_path(&mut self, path: &std::path::Path, force: bool) {
        let preserve = !force && self.has_canvas_surface() && self.md_file_path == path;
        if preserve && let Some(request) = &mut self.canvas_request {
            /* WHY: 実workerが旧本文を読んだ後の通知も集約し、結果適用後に最新本文を再読込する。 */
            request.request_refresh();
            return;
        }
        let snapshot = if preserve {
            self.canvas_surface
                .as_ref()
                .and_then(|surface| surface.revision_snapshot())
        } else {
            None
        };
        let previous = if preserve {
            self.canvas_surface.take()
        } else {
            None
        };
        self.prepare_canvas_source(path);
        if let Some(surface) = previous {
            self.canvas_surface = Some(surface);
        }
        let request = self.start_canvas_request(path, snapshot);
        match request {
            Ok(request) => self.canvas_request = Some(request),
            Err(error) => self.fail_canvas_request(error),
        }
    }

    fn start_canvas_request(
        &self,
        path: &std::path::Path,
        snapshot: Option<super::canvas_request::CanvasRevisionSnapshot>,
    ) -> Result<CanvasParseRequest, CanvasRequestError> {
        snapshot.map_or_else(
            || {
                CanvasParseRequest::start_path(
                    self.session_generation,
                    path.to_path_buf(),
                    self.repaint_ctx.clone(),
                )
            },
            |snapshot| {
                CanvasParseRequest::start_snapshot(
                    self.session_generation,
                    path.to_path_buf(),
                    snapshot,
                    self.repaint_ctx.clone(),
                )
            },
        )
    }

    pub(crate) fn has_canvas_surface(&self) -> bool {
        self.canvas_surface.is_some()
    }

    pub(crate) fn clear_canvas_surface(&mut self) {
        self.canvas_request = None;
        self.canvas_surface = None;
    }

    pub(crate) fn update_canvas_source(&mut self, source: &str, path: &std::path::Path) {
        self.prepare_canvas_source(path);
        let request = CanvasParseRequest::start(
            self.session_generation,
            source.to_owned(),
            self.repaint_ctx.clone(),
        );
        match request {
            Ok(request) => self.canvas_request = Some(request),
            Err(error) => self.fail_canvas_request(error),
        }
    }

    fn prepare_canvas_source(&mut self, path: &std::path::Path) {
        let mut surface = self.canvas_surface.take().unwrap_or_default();
        self.abort_renders();
        self.document_intake = None;
        self.document_surface = None;
        self.document_failure = None;
        self.html_browser = None;
        self.sections.clear();
        self.section_lifecycle.clear();
        self.outline_items.clear();
        self.document_anchors.clear();
        self.anchor_map.clear();
        self.heading_anchors.clear();
        self.block_anchors.clear();
        self.local_image_loader.reset();
        self.fullscreen_image = None;
        self.session_generation = self.session_generation.wrapping_add(1);
        self.md_file_path = path.to_path_buf();
        surface.loading();
        self.canvas_surface = Some(surface);
        self.is_loading = true;
    }

    pub(crate) fn poll_canvas_request(&mut self, ctx: &egui::Context) {
        let Some(request) = &self.canvas_request else {
            return;
        };
        match request.poll() {
            Ok(Some(result)) => self.finish_canvas_result(result),
            Ok(None) => ctx.request_repaint_after(CANVAS_POLL_INTERVAL),
            Err(error) => self.fail_canvas_request(error),
        }
    }

    fn finish_canvas_result(&mut self, result: super::canvas_request::CanvasParseResult) {
        if result.generation != self.session_generation {
            return;
        }
        let refresh_requested = self.finish_canvas_request();
        match result.result {
            Ok(document) => {
                if let Some(surface) = &mut self.canvas_surface {
                    surface.set_document_revision(document, result.revision);
                }
            }
            Err(error) => self.fail_canvas_request(error),
        }
        self.restart_canvas_refresh(refresh_requested);
    }

    fn fail_canvas_request(&mut self, error: CanvasRequestError) {
        let refresh_requested = self.finish_canvas_request();
        if let Some(surface) = &mut self.canvas_surface {
            let error = match error {
                CanvasRequestError::InvalidCanvas(error) => CanvasLoadError::InvalidCanvas(error),
                error => CanvasLoadError::Request(error),
            };
            surface.set_error(error);
        }
        self.restart_canvas_refresh(refresh_requested);
    }

    fn finish_canvas_request(&mut self) -> bool {
        self.is_loading = false;
        self.canvas_request
            .take()
            .is_some_and(|request| request.refresh_requested())
    }

    fn restart_canvas_refresh(&mut self, refresh_requested: bool) {
        if refresh_requested {
            let path = self.md_file_path.clone();
            self.update_canvas_path(&path, false);
        }
    }

    pub(crate) fn show_canvas_surface(&mut self, ui: &mut egui::Ui) {
        self.heading_anchors.clear();
        self.block_anchors.clear();
        self.visible_rect = Some(ui.clip_rect());
        self.content_top_y = ui.next_widget_position().y;
        if let Some(surface) = &mut self.canvas_surface {
            let messages = crate::i18n::I18nOps::get();
            surface.show(ui, &CanvasSurfaceLabels::new(&messages.preview.canvas));
        }
    }
}
