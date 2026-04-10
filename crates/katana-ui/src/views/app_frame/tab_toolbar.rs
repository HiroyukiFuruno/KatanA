use super::types::*;
use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use crate::shell_logic::ShellLogicOps;
use crate::shell_logic::utils::ShellUtils;
use eframe::egui;

const META_INFO_SPACING: f32 = 4.0;
const DOCUMENT_TOOLBAR_SPACING: f32 = 8.0;

impl<'a> TabToolbar<'a> {
    pub(crate) fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }

    pub(crate) fn show(self, ui: &mut egui::Ui) {
        let app = self.app;
        egui::Panel::top("tab_toolbar").show_inside(ui, |ui| {
            let ws_root = app
                .state
                .workspace
                .data
                .as_ref()
                .map(|ws| ws.root.as_path());

            let tab_action = crate::views::top_bar::TabBar::new(
                ws_root,
                &app.state.document.open_documents,
                app.state.document.active_doc_idx,
                &app.state.document.recently_closed_tabs,
                &app.state.document.tab_groups,
                &app.state.layout.inline_rename_group,
            )
            .show(ui);

            if let Some(a) = tab_action {
                app.pending_action = a;
            }

            let doc_info = app.state.active_document().map(|doc| {
                (
                    doc.path.clone(),
                    doc.path.to_string_lossy().starts_with("Katana://ChangeLog"),
                )
            });
            if let Some((doc_path, is_changelog)) = doc_info {
                Self::render_document_toolbar(ui, app, doc_path, is_changelog);
            }
        });
    }

    fn render_document_toolbar(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        doc_path: std::path::PathBuf,
        is_changelog: bool,
    ) {
        let mut out_action = None;
        let bar_height = ui.spacing().interact_size.y;

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), bar_height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                if !is_changelog && let Some(a) = Self::render_breadcrumbs(ui, app, &doc_path) {
                    out_action = Some(a);
                }

                if !is_changelog {
                    ui.add_space(DOCUMENT_TOOLBAR_SPACING);
                    Self::render_file_meta_info(ui, &doc_path);
                }

                if let Some(a) = Self::render_view_mode_bar(ui, app, is_changelog) {
                    out_action = Some(a);
                }
            },
        );

        if app.state.search.doc_search_open
            && let Some(a) = crate::views::top_bar::DocSearchBar::show(ui, &mut app.state.search)
        {
            out_action = Some(a);
        }

        if let Some(a) = out_action {
            app.pending_action = a;
        }
    }

    fn render_breadcrumbs(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        doc_path: &std::path::Path,
    ) -> Option<AppAction> {
        let ws_root = app.state.workspace.data.as_ref().map(|ws| ws.root.clone());
        let rel = ShellLogicOps::relative_full_path(doc_path, ws_root.as_deref());
        Breadcrumbs::new(app, &rel, ws_root.as_deref()).show(ui)
    }

    fn render_file_meta_info(ui: &mut egui::Ui, path: &std::path::Path) {
        let Ok(metadata) = path.metadata() else {
            return;
        };

        let modified = metadata.modified().ok();
        let size_and_date = if let Some(m) = modified {
            format!(
                "{} · {}",
                ShellUtils::format_file_size(metadata.len()),
                ShellUtils::format_modified_time(m)
            )
        } else {
            ShellUtils::format_file_size(metadata.len())
        };

        ui.add_space(META_INFO_SPACING);
        ui.label(egui::RichText::new(size_and_date).small().weak());
    }

    fn render_view_mode_bar(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        is_changelog: bool,
    ) -> Option<AppAction> {
        crate::views::top_bar::ViewModeBar::new(
            app.state.active_view_mode(),
            is_changelog,
            app.state.active_split_direction(),
            app.state.active_pane_order(),
            app.state
                .config
                .settings
                .settings()
                .behavior
                .scroll_sync_enabled,
            app.state.scroll.sync_override,
            app.state.update.available.is_some(),
            app.state.update.checking,
            true,
        )
        .show(ui, &mut app.state.search)
    }
}
