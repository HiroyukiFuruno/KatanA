use katana_platform::SplitDirection;

use crate::app::preview::PreviewOps;
use crate::preview_pane::DownloadRequest;
use crate::shell::KatanaApp;
use crate::shell::SPLIT_PREVIEW_PANEL_MIN_WIDTH;
use crate::shell_ui::{SPLIT_HALF_RATIO, SPLIT_PANEL_MAX_RATIO};

use crate::theme_bridge;
use crate::views::panels::editor::EditorContent;
use crate::views::panels::preview::{PreviewContent, PreviewLogicOps};
use katana_platform::PaneOrder;
pub(crate) struct SplitMode<'a> {
    pub _ctx: &'a egui::Context,
    pub app: &'a mut KatanaApp,
    pub split_dir: SplitDirection,
    pub pane_order: PaneOrder,
}
impl<'a> SplitMode<'a> {
    pub fn new(
        _ctx: &'a egui::Context,
        app: &'a mut KatanaApp,
        split_dir: SplitDirection,
        pane_order: PaneOrder,
    ) -> Self {
        Self {
            _ctx,
            app,
            split_dir,
            pane_order,
        }
    }
    pub fn show(self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        let app = self.app;
        let split_dir = self.split_dir;
        let pane_order = self.pane_order;
        let ctx = ui.ctx().clone();
        match split_dir {
            SplitDirection::Horizontal => HorizontalSplit::new(&ctx, app, pane_order).show(ui),
            SplitDirection::Vertical => VerticalSplit::new(&ctx, app, pane_order).show(ui),
        }
    }
}

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

        panel_side
            .resizable(true)
            .min_size(SPLIT_PREVIEW_PANEL_MIN_WIDTH)
            .default_size(half_width)
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                if let Some(path) = &active_path {
                    let pane = crate::shell::KatanaApp::get_preview_pane(
                        &mut app.tab_previews,
                        path.clone(),
                    );
                    let toc_visible = app.state.config.settings.settings().layout.toc_visible;
                    let show_toc = app.state.layout.show_toc;
                    download_req = PreviewContent::new(
                        pane,
                        app.state.document.active_document(),
                        &mut app.state.scroll,
                        toc_visible,
                        show_toc,
                        &mut app.pending_action,
                        scroll_sync,
                        Some(app.state.search.doc_search_query.clone()),
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
                )
                .show(ui);
            });

        download_req
    }
}

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
        let available_height = ui.ctx().content_rect().height();
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

        if show_preview_top {
            egui::Panel::top(panel_id)
                .resizable(true)
                .default_size(half_height)
                .max_size(available_height * SPLIT_PANEL_MAX_RATIO)
                .frame(egui::Frame::NONE)
                .show_inside(ui, |ui| {
                    if let Some(path) = &active_path {
                        let pane = crate::shell::KatanaApp::get_preview_pane(
                            &mut app.tab_previews,
                            path.clone(),
                        );
                        let toc_visible = app.state.config.settings.settings().layout.toc_visible;
                        let show_toc = app.state.layout.show_toc;
                        download_req = PreviewContent::new(
                            pane,
                            app.state.document.active_document(),
                            &mut app.state.scroll,
                            toc_visible,
                            show_toc,
                            &mut app.pending_action,
                            scroll_sync,
                            Some(app.state.search.doc_search_query.clone()),
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
                    )
                    .show(ui);
                });
        } else {
            egui::Panel::bottom(panel_id)
                .resizable(true)
                .default_size(half_height)
                .max_size(available_height * SPLIT_PANEL_MAX_RATIO)
                .frame(egui::Frame::NONE)
                .show_inside(ui, |ui| {
                    if let Some(path) = &active_path {
                        let pane = crate::shell::KatanaApp::get_preview_pane(
                            &mut app.tab_previews,
                            path.clone(),
                        );
                        let toc_visible = app.state.config.settings.settings().layout.toc_visible;
                        let show_toc = app.state.layout.show_toc;
                        download_req = PreviewContent::new(
                            pane,
                            app.state.document.active_document(),
                            &mut app.state.scroll,
                            toc_visible,
                            show_toc,
                            &mut app.pending_action,
                            scroll_sync,
                            Some(app.state.search.doc_search_query.clone()),
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
                    )
                    .show(ui);
                });
        }

        download_req
    }
}

pub(crate) struct PreviewOnly<'a> {
    pub ui: &'a mut egui::Ui,
    pub app: &'a mut KatanaApp,
}
impl<'a> PreviewOnly<'a> {
    pub fn new(ui: &'a mut egui::Ui, app: &'a mut KatanaApp) -> Self {
        Self { ui, app }
    }
    pub fn show(self) {
        let ui = self.ui;
        let app = self.app;
        ui.painter().rect_filled(
            ui.max_rect(),
            0.0,
            theme_bridge::ThemeBridgeOps::rgb_to_color32(
                app.state
                    .config
                    .settings
                    .settings()
                    .effective_theme_colors()
                    .preview
                    .background,
            ),
        );
        let active_path = app.state.active_document().map(|d| d.path.clone());
        if let Some(path) = active_path {
            let pane = crate::shell::KatanaApp::get_preview_pane(&mut app.tab_previews, path);
            let toc_visible = app.state.config.settings.settings().layout.toc_visible;
            let show_toc = app.state.layout.show_toc;
            PreviewContent::new(
                pane,
                app.state.document.active_document(),
                &mut app.state.scroll,
                toc_visible,
                show_toc,
                &mut app.pending_action,
                false,
                Some(app.state.search.doc_search_query.clone()),
            )
            .show(ui);

            /* WHY: In PreviewOnly mode, there is no editor to consume the scroll_to_line request. */
            /* WHY: We consume it here right after PreviewContent has processed it. */
            app.state.scroll.scroll_to_line = None;
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(
                    crate::i18n::I18nOps::get()
                        .workspace
                        .no_document_selected
                        .clone(),
                );
            });
        }
    }
}
