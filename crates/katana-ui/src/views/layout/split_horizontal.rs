use crate::app::preview::PreviewOps;
use crate::preview_pane::DownloadRequest;
use crate::shell::KatanaApp;
use crate::shell::SPLIT_PREVIEW_PANEL_MIN_WIDTH;
use crate::shell_ui::SPLIT_HALF_RATIO;
use crate::theme_bridge;
use crate::views::panels::editor::EditorContent;
use crate::views::panels::preview::{PreviewContent, PreviewLogicOps};
use katana_platform::PaneOrder;

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
        let available_width = ui.ctx().content_rect().width();
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

        let scroll_sync = app.state.scroll.sync_override.unwrap_or(
            app.state
                .config
                .settings
                .settings()
                .behavior
                .scroll_sync_enabled,
        );

        let is_first_frame = egui::containers::panel::PanelState::load(ui.ctx(), panel_id).is_none();

        panel_side
            .resizable(true)
            .min_size(SPLIT_PREVIEW_PANEL_MIN_WIDTH)
            .default_size(half_width)
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                if is_first_frame {
                    /* WHY: Force the panel to perfectly snap to half width on first load. */
                    ui.set_min_width(half_width);
                }
                if let Some(path) = &active_path {
                    let pane = crate::shell::KatanaApp::get_preview_pane(
                        &mut app.tab_previews,
                        path.clone(),
                    );
                    download_req = PreviewContent::new(
                        pane,
                        app.state.document.active_document(),
                        &mut app.state.scroll,
                        &mut app.pending_action,
                        scroll_sync,
                        Some(app.state.search.doc_search.query.clone()),
                        Some(app.state.search.doc_search_active_index),
                    )
                    .show(ui);
                }
            });

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
