use super::types::*;
use crate::app_state::AppAction;
use crate::preview_pane::{DownloadRequest, PreviewPane};
use eframe::egui;
const BACK_TO_TOP_THRESHOLD: f32 = 400.0;
use super::types::PreviewLogicOps;

impl<'a> PreviewContent<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        preview: &'a mut PreviewPane,
        document: Option<&'a katana_core::document::Document>,
        scroll: &'a mut crate::app_state::ScrollState,
        action: &'a mut AppAction,
        scroll_sync: bool,
        search_query: Option<String>,
        doc_search_active_index: Option<usize>,
    ) -> Self {
        Self {
            preview,
            document,
            scroll,
            action,
            scroll_sync,
            search_query,
            doc_search_active_index,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        let PreviewContent {
            preview,
            document,
            scroll,
            action,
            scroll_sync,
            search_query,
            doc_search_active_index,
        } = self;

        let mut download_req = None;
        /* WHY: Lock preview content to the panel's current available width so
         * intrinsic-size widgets cannot expand the parent resizable panel state. */
        let panel_width = ui.available_width();
        ui.set_min_width(panel_width);
        ui.set_max_width(panel_width);

        /* WHY: Check for forced scroll target from Sync System or Navigation. */
        let mut forced_offset = PreviewLogicOps::compute_forced_offset(
            scroll_sync,
            scroll,
            preview,
            crate::shell::TREE_ROW_HEIGHT,
            ui.available_height(),
        );

        if preview.scroll_request == Some(0) {
            forced_offset = Some(0.0);
            preview.scroll_request = None;
        } else if let Some(idx) = preview.scroll_request
            && let Some(offset) = PreviewLogicOps::heading_scroll_offset(
                idx,
                &preview.anchor_map,
                preview.content_top_y,
            )
        {
            forced_offset = Some(offset);
            preview.scroll_request = None;
        }

        let mut scroll_area = egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .id_salt((
                "preview_scroll_area",
                document.map(|doc| doc.path.as_path()),
            ));

        if let Some(offset) = forced_offset {
            scroll_area = scroll_area.vertical_scroll_offset(offset);
        }

        /* WHY: Keep the scroll area itself full-width so its scrollbar touches the
         * preview edge; width capping still happens through this fixed child rect. */
        let inner_content_width = ui.available_width();
        let child_rect = egui::Rect::from_min_size(
            ui.next_widget_position(),
            egui::vec2(inner_content_width, ui.available_height()),
        );

        let output = ui
            .scope_builder(
                egui::UiBuilder::new()
                    .max_rect(child_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
                |ui| {
                    scroll_area.show(ui, |ui| {
                        /* WHY: Use a Frame with explicit horizontal margin for consistent padding (12px). */
                        egui::Frame::none()
                            .inner_margin(egui::Margin::symmetric(
                                crate::shell_ui::PREVIEW_CONTENT_PADDING,
                                0,
                            ))
                            .show(ui, |ui| {
                                PreviewLogicOps::render_preview_top_padding(ui);
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
                                    doc_search_active_index,
                                );

                                /* WHY: Hover synchronization is decoupled from scroll_sync.        */
                                if is_interactive {
                                    scroll.hovered_preview_lines = hovered_lines;
                                }

                                /* WHY: Handle clicks for Editor-to-Preview navigation. */
                                if is_interactive
                                    && ui.rect_contains_pointer(ui.min_rect())
                                    && ui.input(|i| i.pointer.primary_clicked())
                                    && let Some(hovered) = scroll.hovered_preview_lines.first()
                                {
                                    scroll.scroll_to_line = Some(hovered.start);
                                }

                                download_req = req;
                                /* WHY: Handle embedded actions like Task List toggling. */
                                if let Some((global_index, new_state)) = actions.into_iter().next() {
                                    *action = AppAction::ToggleTaskList {
                                        global_index,
                                        new_state,
                                    };
                                }

                                PreviewLogicOps::render_preview_bottom_padding(ui, scroll);
                            });
                    })
                },
            );

        if scroll_sync {
            PreviewLogicOps::update_scroll_sync(
                scroll,
                preview,
                crate::shell::TREE_ROW_HEIGHT,
                output.inner.content_size.y,
                output.inner.inner_rect.height(),
                output.inner.state.offset.y,
            );
        }

        /* WHY: We no longer need PreviewHeader. The TOC is toggled from the AppFrame side panel.
        Export and Story view are now floating buttons at the bottom right. */

        /* WHY: Render floating action buttons at the bottom right. */
        let scroll_offset = output.inner.state.offset.y;
        let show_back_to_top = scroll_offset > BACK_TO_TOP_THRESHOLD;
        PreviewLogicOps::render_floating_buttons(
            ui,
            document.is_some(),
            show_back_to_top,
            action,
            preview,
        );

        /* WHY: FB25 — For LinterDocs, render a "View on GitHub" button at the
         * top-right inside the preview pane (not in the toolbar). */
        if let Some(doc) = document {
            PreviewLogicOps::render_linter_docs_github_button(ui, &doc.path);
        }

        download_req
    }
}
