use super::types::*;
use crate::app_state::ViewMode;
use crate::preview_pane::DownloadRequest;
use crate::shell::KatanaApp;
use crate::views::panels::preview::PreviewSidePanels;
use eframe::egui;

impl<'a> CentralContent<'a> {
    pub(crate) fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }

    pub(crate) fn show(self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        let app = self.app;
        let current_mode = app.state.active_view_mode();
        let is_split = current_mode == ViewMode::Split;
        let mut is_changelog_tab = false;
        let mut is_virtual_read_only = false;

        if let Some(doc) = app.state.active_document() {
            let p = doc.path.to_string_lossy();
            if p.starts_with("Katana://ChangeLog") {
                is_changelog_tab = true;
            } else if p.starts_with("Katana://Welcome") || p.starts_with("Katana://Guide") {
                /* WHY: Welcome / Guide are virtual read-only preview docs — no editor controls */
                is_virtual_read_only = true;
            }
        }

        Self::render_preview_side_panels(ui, app);

        if is_changelog_tab {
            egui::CentralPanel::default().show_inside(ui, |ui| {
                crate::changelog::ChangelogOps::render_release_notes_tab(
                    ui,
                    &app.changelog_sections,
                    app.changelog_rx.is_some(),
                    app.state
                        .config
                        .settings
                        .settings()
                        .layout
                        .accordion_vertical_line,
                );
            });
            return None;
        }

        if is_virtual_read_only {
            let mut req = None;
            egui::CentralPanel::default().show_inside(ui, |ui| {
                req = crate::views::layout::split::PreviewOnly::new(ui, app).show();
            });
            return req;
        }

        if app.state.active_document().is_none() && current_mode != ViewMode::PreviewOnly {
            egui::CentralPanel::default().show_inside(ui, |ui| {
                crate::views::panels::dashboard::DashboardView::new(app).show(ui, app);
            });
            return None;
        }

        if is_split {
            return Self::render_split_mode(ui, app);
        }

        Self::render_single_mode(ui, app, current_mode)
    }

    fn render_preview_side_panels(ui: &mut egui::Ui, app: &mut KatanaApp) {
        PreviewSidePanels::new(app).show(ui);
    }

    fn render_split_mode(ui: &mut egui::Ui, app: &mut KatanaApp) -> Option<DownloadRequest> {
        let split_dir = app.state.active_split_direction();
        let pane_order = app.state.active_pane_order();
        let ctx = ui.ctx().clone();
        crate::views::layout::split::SplitMode::new(&ctx, app, split_dir, pane_order).show(ui)
    }

    fn render_single_mode(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        current_mode: ViewMode,
    ) -> Option<DownloadRequest> {
        let mut download_req = None;
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ui.ctx().global_style()).inner_margin(0.0))
            .show_inside(ui, |ui| match current_mode {
                ViewMode::CodeOnly => {
                    crate::views::panels::editor::EditorContent::new(
                        app.state.document.active_document(),
                        &mut app.state.scroll,
                        &mut app.pending_action,
                        false,
                        &app.state.search.doc_search_matches,
                        app.state.search.doc_search_active_index,
                        &mut app.editor_cursor_range,
                        app.pending_editor_cursor.take(),
                    )
                    .show(ui);
                }
                ViewMode::PreviewOnly => {
                    download_req = crate::views::layout::split::PreviewOnly::new(ui, app).show();
                }
                ViewMode::Split => {}
            });
        download_req
    }
}
