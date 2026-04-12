use katana_platform::SplitDirection;

use crate::app::preview::PreviewOps;
use crate::preview_pane::DownloadRequest;
use crate::shell::KatanaApp;
use crate::theme_bridge;
use crate::views::panels::preview::PreviewContent;
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

pub(crate) use super::split_horizontal::HorizontalSplit;

pub(crate) use super::split_vertical::VerticalSplit;

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
