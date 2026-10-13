use eframe::egui::{self, ScrollArea};

use super::types::*;

#[path = "ui_fullscreen.rs"]
mod fullscreen;

impl PreviewPane {
    pub fn is_all_drawn(&self) -> bool {
        self.canvas_surface.as_ref().map_or_else(
            || self.section_lifecycle.iter().all(|s| s.is_drawn),
            |surface| surface.is_drawn(),
        )
    }
    pub fn is_any_pending(&self) -> bool {
        self.canvas_request.is_some() || self.section_lifecycle.iter().any(|s| !s.is_loaded)
    }
}

impl PreviewPane {
    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<(usize, char)> {
        self.repaint_ctx = Some(ui.ctx().clone());
        self.poll_renders(ui.ctx());

        if self.has_canvas_surface() {
            self.show_canvas_surface(ui);
            return Vec::new();
        }

        let mut actions = Vec::new();

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                /* WHY: The scroll area must occupy the full preview width so the scrollbar
                 * stays visually attached to the right-side toolbar. */
                let inner_content_width = ui.available_width();
                let child_rect = egui::Rect::from_min_size(
                    ui.next_widget_position(),
                    egui::vec2(inner_content_width, 0.0),
                );
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(child_rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                    |ui| {
                        actions = self.render_sections(ui, None, None, None, None, false);
                    },
                );
            });
        self.render_fullscreen_modal(ui.ctx());
        actions
    }

    pub fn show_content(
        &mut self,
        ui: &mut egui::Ui,
        active_editor_line: Option<usize>,
        hovered_lines: Option<&mut Vec<std::ops::Range<usize>>>,
        search_query: Option<String>,
        search_active_index: Option<usize>,
    ) -> Vec<(usize, char)> {
        self.repaint_ctx = Some(ui.ctx().clone());
        self.poll_renders(ui.ctx());
        if self.has_canvas_surface() {
            self.show_canvas_surface(ui);
            return Vec::new();
        }
        if self.has_document_surface() {
            self.heading_anchors.clear();
            self.block_anchors.clear();
            self.visible_rect = Some(ui.clip_rect());
            self.content_top_y = ui.next_widget_position().y;
            self.show_document_surface(ui);
            return Vec::new();
        }
        if self.has_html_browser() {
            self.heading_anchors.clear();
            self.block_anchors.clear();
            self.visible_rect = Some(ui.clip_rect());
            self.content_top_y = ui.next_widget_position().y;
            self.show_html_browser(ui);
            return Vec::new();
        }
        self.render_sections(
            ui,
            active_editor_line,
            hovered_lines,
            search_query,
            search_active_index,
            false,
        )
    }

    pub(crate) fn render_sections(
        &mut self,
        ui: &mut egui::Ui,
        active_editor_line: Option<usize>,
        mut hovered_lines: Option<&mut Vec<std::ops::Range<usize>>>,
        search_query: Option<String>,
        search_active_index: Option<usize>,
        is_slideshow: bool,
    ) -> Vec<(usize, char)> {
        self.visible_rect = Some(ui.clip_rect());
        self.content_top_y = ui.next_widget_position().y;
        self.heading_anchors.clear();
        self.block_anchors.clear();
        let mut fullscreen_request: Option<usize> = None;
        let actions = crate::preview_pane::SectionLogicOps::render_sections(
            ui,
            &mut self.commonmark_cache,
            &self.sections,
            &self.md_file_path,
            self.scroll_request,
            Some(&mut self.heading_anchors),
            Some(&mut self.block_anchors),
            Some(&mut self.viewer_states),
            &self.local_image_loader,
            if is_slideshow {
                None
            } else {
                Some(&mut self.section_lifecycle)
            },
            Some(&mut fullscreen_request),
            active_editor_line,
            hovered_lines.as_deref_mut(),
            search_query,
            search_active_index,
            is_slideshow,
        );

        crate::preview_pane::types::DocumentAnchorMapItem::sync_rects(
            &mut self.anchor_map,
            &self.heading_anchors,
            &self.block_anchors,
        );

        if let Some(ref mut h) = hovered_lines {
            for item in &self.anchor_map {
                if let Some(rect) = item.outer_rect
                    && ui.rect_contains_pointer(rect)
                {
                    h.push(item.line_span.clone());
                }
            }
        }

        self.scroll_request = None;

        let ctx = ui.ctx().clone();
        self.handle_fullscreen_request(fullscreen_request, Some(&ctx));

        actions
    }
}
impl Drop for PreviewPane {
    fn drop(&mut self) {
        self.cancel_token
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}
