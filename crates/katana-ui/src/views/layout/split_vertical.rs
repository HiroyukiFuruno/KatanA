use crate::app::preview::PreviewOps;
use crate::preview_pane::DownloadRequest;
use crate::shell::KatanaApp;
use crate::shell_ui::{SPLIT_HALF_RATIO, SPLIT_PANEL_MAX_RATIO};
use crate::theme_bridge;
use crate::views::panels::editor::EditorContent;
use crate::views::panels::preview::{PreviewContent, PreviewLogicOps};
use katana_platform::PaneOrder;

const LAYOUT_RATCHET_THRESHOLD: f32 = 0.5;

pub(crate) struct VerticalSplit<'a> {
    pub _ctx: &'a egui::Context,
    pub app: &'a mut KatanaApp,
    pub pane_order: PaneOrder,
}

impl<'a> VerticalSplit<'a> {
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
        /* WHY: Use local available height inside the split container, not global window height. */
        let available_height = ui.available_height();
        let half_height = available_height * SPLIT_HALF_RATIO;
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
                PreviewLogicOps::preview_panel_id(active_path.as_deref(), "preview_panel_v_bottom")
            }
            PaneOrder::PreviewFirst => {
                PreviewLogicOps::preview_panel_id(active_path.as_deref(), "preview_panel_v_top")
            }
        };

        let show_preview_top = pane_order == PaneOrder::PreviewFirst;
        let scroll_sync = app.state.scroll.sync_override.unwrap_or(
            app.state
                .config
                .settings
                .settings()
                .behavior
                .scroll_sync_enabled,
        );

        let panel = if show_preview_top {
            egui::Panel::top(panel_id)
        } else {
            egui::Panel::bottom(panel_id)
        };
        let prev_panel_state = egui::containers::panel::PanelState::load(ui.ctx(), panel_id);

        panel
            .resizable(true)
            .default_height(half_height)
            .max_height(available_height * SPLIT_PANEL_MAX_RATIO)
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                if let Some(path) = &active_path {
                    let pane = KatanaApp::get_preview_pane(&mut app.tab_previews, path.clone());
                    /* WHY: Render preview content in a fixed child rect so intrinsic-size
                     * widgets (tables/code blocks) cannot leak size back into the resizable
                     * split panel state. */
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
             * If panel height grows while the splitter is not being dragged, treat it
             * as layout ratchet and restore the previous height. */
            if !was_dragging && curr.rect.height() > prev.rect.height() + LAYOUT_RATCHET_THRESHOLD {
                ui.ctx().data_mut(|data| {
                    data.insert_persisted(
                        panel_id,
                        egui::containers::panel::PanelState {
                            rect: egui::Rect::from_min_size(
                                curr.rect.min,
                                egui::vec2(curr.rect.width(), prev.rect.height()),
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
                )
                .show(ui);
            });

        download_req
    }
}
