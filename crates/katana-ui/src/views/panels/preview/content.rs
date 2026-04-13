use super::types::*;
use crate::app_state::{AppAction, ScrollSource};
use crate::preview_pane::{DownloadRequest, PreviewPane};
use crate::shell_ui::PREVIEW_CONTENT_PADDING;
use eframe::egui;

impl<'a> PreviewContent<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        preview: &'a mut PreviewPane,
        document: Option<&'a katana_core::document::Document>,
        scroll: &'a mut crate::app_state::ScrollState,
        toc_visible: bool,
        show_toc: bool,
        action: &'a mut AppAction,
        scroll_sync: bool,
        search_query: Option<String>,
    ) -> Self {
        Self {
            preview,
            document,
            scroll,
            toc_visible,
            show_toc,
            action,
            scroll_sync,
            search_query,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        let preview = self.preview;
        let document = self.document;
        let scroll = self.scroll;
        let toc_visible = self.toc_visible;
        let show_toc = self.show_toc;
        let action = self.action;
        let scroll_sync = self.scroll_sync;
        let search_query = self.search_query.clone();
        let mut download_req = None;
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        let outer_rect = ui.available_rect_before_wrap();
        let hover_sense = if ui.is_enabled() {
            egui::Sense::hover()
        } else {
            egui::Sense::empty()
        };
        ui.allocate_rect(outer_rect, hover_sense);

        let mut scroll_area = egui::ScrollArea::vertical()
            .id_salt("preview_scroll")
            .auto_shrink(std::array::from_fn(|_| false));

        let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
        let forced_offset = super::types::PreviewLogicOps::compute_forced_offset(
            scroll_sync,
            scroll,
            preview,
            row_height,
        );

        if let Some(target_scroll_offset) = forced_offset {
            scroll.preview_echo.record(target_scroll_offset);
            scroll_area = scroll_area.vertical_scroll_offset(target_scroll_offset);
        }

        let mut content_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(outer_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        content_ui.set_clip_rect(outer_rect);

        let output = scroll_area.show(&mut content_ui, |ui| {
            egui::Frame::NONE
                .inner_margin(egui::Margin::symmetric(
                    PREVIEW_CONTENT_PADDING,
                    PREVIEW_CONTENT_PADDING,
                ))
                .show(ui, |ui| {
                    let content_width = ui.available_width();
                    let child_rect = egui::Rect::from_min_size(
                        ui.next_widget_position(),
                        egui::vec2(content_width, 0.0),
                    );
                    ui.scope_builder(
                        egui::UiBuilder::new()
                            .max_rect(child_rect)
                            .layout(egui::Layout::top_down(egui::Align::Min)),
                        |ui| {
                            /* WHY: 0.25rem padding */
                            const PREVIEW_PANE_TOP_BOTTOM_PADDING: f32 = 4.0;
                            ui.add_space(PREVIEW_PANE_TOP_BOTTOM_PADDING);
                            let is_interactive = ui.is_enabled();
                            let mut hovered_lines = Vec::new();
                            let hover_out = if is_interactive {
                                Some(&mut hovered_lines)
                            } else {
                                None
                            };
                            let (req, actions) = preview.show_content(
                                ui,
                                scroll.active_editor_line,
                                hover_out,
                                search_query.clone(),
                            );
                            if is_interactive
                                && scroll_sync
                                && scroll.source != ScrollSource::Preview
                            {
                                scroll.hovered_preview_lines = hovered_lines.clone();
                            }

                            if is_interactive
                                && ui.rect_contains_pointer(ui.min_rect())
                                && ui.input(|i| i.pointer.primary_clicked())
                                && let Some(hovered) = hovered_lines.first()
                            {
                                scroll.scroll_to_line = Some(hovered.start);
                            }
                            download_req = req;
                            if let Some((global_index, new_state)) = actions.into_iter().next() {
                                *action = AppAction::ToggleTaskList {
                                    global_index,
                                    new_state,
                                };
                            }
                            ui.add_space(PREVIEW_PANE_TOP_BOTTOM_PADDING);
                        },
                    );
                });
        });

        if scroll_sync {
            super::types::PreviewLogicOps::update_scroll_sync(
                scroll,
                preview,
                ui.text_style_height(&egui::TextStyle::Monospace),
                output.content_size.y,
                output.inner_rect.height(),
                output.state.offset.y,
            );
        }

        super::types::PreviewHeader::new(document.is_some(), toc_visible, show_toc, action)
            .show(ui);

        download_req
    }
}
