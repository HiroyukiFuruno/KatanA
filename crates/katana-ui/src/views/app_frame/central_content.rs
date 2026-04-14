use super::types::*;
use crate::app_state::ViewMode;
use crate::preview_pane::DownloadRequest;
use crate::shell::KatanaApp;
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

        Self::render_toc_if_needed(ui, app);

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
            egui::CentralPanel::default().show_inside(ui, |ui| {
                crate::views::layout::split::PreviewOnly::new(ui, app).show();
            });
            return None;
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

        Self::render_single_mode(ui, app, current_mode);
        None
    }

    fn render_toc_if_needed(ui: &mut egui::Ui, app: &mut KatanaApp) {
        if !app.state.layout.show_toc || !app.state.config.settings.settings().layout.toc_visible {
            return;
        }

        let doc = match app.state.active_document() {
            Some(d) => d,
            None => return,
        };

        if let Some(preview) = app.tab_previews.iter_mut().find(|p| p.path == doc.path) {
            let (clicked_line, active_index) =
                crate::views::panels::toc::TocPanel::new(&mut preview.pane, &mut app.state)
                    .show(ui);

            if let Some(clicked) = clicked_line {
                app.state.scroll.scroll_to_line = Some(clicked);
                app.state.scroll.last_scroll_to_line = None;
            }
            app.state.active_toc_index = active_index;
        }
    }

    fn render_split_mode(ui: &mut egui::Ui, app: &mut KatanaApp) -> Option<DownloadRequest> {
        let split_dir = app.state.active_split_direction();
        let pane_order = app.state.active_pane_order();
        let ctx = ui.ctx().clone();
        crate::views::layout::split::SplitMode::new(&ctx, app, split_dir, pane_order).show(ui)
    }

    fn render_single_mode(ui: &mut egui::Ui, app: &mut KatanaApp, current_mode: ViewMode) {
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
                    crate::views::layout::split::PreviewOnly::new(ui, app).show();
                }
                ViewMode::Split => {}
            });
    }
}
