use crate::app::preview::PreviewOps;
use crate::preview_pane::DownloadRequest;
use crate::shell::KatanaApp;
use crate::shell::SPLIT_PREVIEW_PANEL_MIN_WIDTH;
use crate::shell_ui::SPLIT_HALF_RATIO;
use crate::theme_bridge;
use crate::views::panels::editor::EditorContent;
use crate::views::panels::preview::{PreviewContent, PreviewLogicOps};
use katana_platform::PaneOrder;

const LAYOUT_RATCHET_THRESHOLD: f32 = 0.5;

pub(crate) struct HorizontalSplit<'a> {
    pub _ctx: &'a egui::Context,
    pub app: &'a mut KatanaApp,
    pub pane_order: PaneOrder,
}
impl<'a> HorizontalSplit<'a> {
    pub fn new(_ctx: &'a egui::Context, app: &'a mut KatanaApp, pane_order: PaneOrder) -> Self {
        Self {
            _ctx,
            app,
            pane_order,
        }
    }
    pub fn show(self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        let app = self.app;
        let pane_order = self.pane_order;
        /* WHY: Use local available width inside the split container, not global window width. */
        let available_width = ui.available_width();
        let half_width = (available_width * SPLIT_HALF_RATIO).max(SPLIT_PREVIEW_PANEL_MIN_WIDTH);
        let _preview_bg = theme_bridge::ThemeBridgeOps::rgb_to_color32(
            app.state
                .config
                .settings
                .settings()
                .effective_theme_colors()
                .preview
                .background,
        );
        let active_path = app.state.active_document().map(|d| d.path.clone());
        let mut download_req = None;
        let panel_id = match pane_order {
            PaneOrder::EditorFirst => {
                PreviewLogicOps::preview_panel_id(active_path.as_deref(), "preview_panel_h_right")
            }
            PaneOrder::PreviewFirst => {
                PreviewLogicOps::preview_panel_id(active_path.as_deref(), "preview_panel_h_left")
            }
        };

        let panel_side = match pane_order {
            PaneOrder::EditorFirst => egui::Panel::right(panel_id),
            PaneOrder::PreviewFirst => egui::Panel::left(panel_id),
        };
        let prev_panel_state = egui::containers::panel::PanelState::load(ui.ctx(), panel_id);

        let scroll_sync = app.state.scroll.sync_override.unwrap_or(
            app.state
                .config
                .settings
                .settings()
                .behavior
                .scroll_sync_enabled,
        );

        panel_side
            .resizable(true)
            .min_width(SPLIT_PREVIEW_PANEL_MIN_WIDTH)
            .max_width(available_width * crate::shell_ui::SPLIT_PANEL_MAX_RATIO)
            .default_width(half_width)
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                if let Some(path) = &active_path {
                    let pane = crate::shell::KatanaApp::get_preview_pane(
                        &mut app.tab_previews,
                        path.clone(),
                    );
                    /* WHY: Render preview content in a fixed child rect so intrinsic-size
                     * widgets (tables/code blocks) cannot leak width back into the resizable
                     * panel state and cause expand-without-shrink behavior. */
                    let preview_rect = ui.max_rect();
                    let out = ui.allocate_ui_at_rect(preview_rect, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                            PreviewContent::new(
                                pane,
                                app.state.document.active_document(),
                                &mut app.state.scroll,
                                &mut app.pending_action,
                                scroll_sync,
                                Some(app.state.search.doc_search.query.clone()),
                                Some(app.state.search.doc_search_active_index),
                            )
                            .show(ui)
                        })
                        .inner
                    });
                    download_req = out.inner;
                }
            });
        let current_panel_state = egui::containers::panel::PanelState::load(ui.ctx(), panel_id);
        if let (Some(prev), Some(curr)) = (prev_panel_state, current_panel_state) {
            let was_dragging = ui.ctx().input(|i| i.pointer.primary_down());
            /* WHY: Guard against intrinsic-size leaks from preview contents.
             * If panel width grows while the splitter is not being dragged, treat it
             * as layout ratchet and restore the previous width. */
            if !was_dragging && curr.rect.width() > prev.rect.width() + LAYOUT_RATCHET_THRESHOLD {
                ui.ctx().data_mut(|data| {
                    data.insert_persisted(
                        panel_id,
                        egui::containers::panel::PanelState {
                            rect: egui::Rect::from_min_size(
                                curr.rect.min,
                                egui::vec2(prev.rect.width(), curr.rect.height()),
                            ),
                        },
                    );
                });
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ui.ctx().global_style()).inner_margin(0.0))
            .show_inside(ui, |ui| {
                EditorContent::new(
                    app.state.document.active_document(),
                    &mut app.state.scroll,
                    &mut app.pending_action,
                    scroll_sync,
                    &app.state.search.doc_search_matches,
                    app.state.search.doc_search_active_index,
                    &mut app.editor_cursor_range,
                    app.pending_editor_cursor.take(),
                    app.state
                        .document
                        .active_document()
                        .map(|doc| app.state.diagnostics.get_file_diagnostics(&doc.path))
                        .unwrap_or(&[]),
                )
                .show(ui);
            });

        download_req
    }
}
