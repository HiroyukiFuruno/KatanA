use super::types::*;
use crate::app_state::{AppAction, ScrollSource};
use crate::preview_pane::{DownloadRequest, PreviewPane};
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
        doc_search_active_index: Option<usize>,
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
            doc_search_active_index,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        let PreviewContent {
            preview,
            document,
            scroll,
            toc_visible,
            show_toc,
            action,
            scroll_sync,
            search_query,
            doc_search_active_index,
        } = self;

        let mut download_req = None;

        /* WHY: Check for forced scroll target from Sync System or Navigation. */
        let forced_offset = PreviewLogicOps::compute_forced_offset(
            scroll_sync,
            scroll,
            preview,
            crate::shell::TREE_ROW_HEIGHT,
            ui.available_height(),
        );

        let mut scroll_area = egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .id_source("preview_scroll_area");

        if let Some(offset) = forced_offset {
            scroll_area = scroll_area.vertical_scroll_offset(offset);
        }

        let output = scroll_area.show(ui, |ui| {
            /* WHY: Use a Frame with explicit horizontal margin for consistent padding (12px). */
            egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(
                    crate::shell_ui::PREVIEW_CONTENT_PADDING,
                    0,
                ))
                .show(ui, |ui| {
                    /* WHY: Expand to available width to ensure the Resizable SidePanel doesn't collapse. */
                    ui.set_min_width(ui.available_width());

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

                    /* WHY: Synchronize hover state if we are not the source of scrolling. */
                    if is_interactive && scroll_sync && scroll.source != ScrollSource::Preview {
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
        });

        if scroll_sync {
            PreviewLogicOps::update_scroll_sync(
                scroll,
                preview,
                crate::shell::TREE_ROW_HEIGHT,
                output.content_size.y,
                output.inner_rect.height(),
                output.state.offset.y,
            );
        }

        /* WHY: Overlay the PreviewHeader (TOC toggle, etc.) on top of the content. */
        PreviewHeader::new(document.is_some(), toc_visible, show_toc, action).show(ui);

        download_req
    }
}
