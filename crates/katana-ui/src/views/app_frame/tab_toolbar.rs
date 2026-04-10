use super::types::*;
use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use crate::shell_logic::ShellLogicOps;
use eframe::egui;

const TAB_TOOLBAR_CENTER_WIDTH_RATIO: f32 = 0.5;

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
        let rect = ui.available_rect_before_wrap();
        let total_width = rect.width();

        ui.allocate_ui_at_rect(rect, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let Some(a) = Self::render_view_mode_bar(ui, app, is_changelog) else {
                    return;
                };
                out_action = Some(a);
            });
        });

        ui.allocate_ui_at_rect(rect, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                if !is_changelog {
                    let icon_size = crate::icon::IconSize::Medium.to_vec2();
                    let info_icon = egui::Image::new(crate::Icon::Info.uri())
                        .tint(ui.visuals().weak_text_color())
                        .fit_to_exact_size(icon_size);

                    if ui.add(egui::ImageButton::new(info_icon)).clicked() {
                        out_action = Some(AppAction::ShowMetaInfo(doc_path.clone()));
                    }
                }
            });
        });

        let center_rect = egui::Rect::from_center_size(
            rect.center(),
            egui::vec2(total_width * TAB_TOOLBAR_CENTER_WIDTH_RATIO, rect.height()),
        );

        ui.allocate_ui_at_rect(center_rect, |ui| {
            ui.centered_and_justified(|ui| {
                if is_changelog {
                    return;
                }
                let Some(a) = Self::render_breadcrumbs(ui, app, &doc_path) else {
                    return;
                };
                out_action = Some(a);
            });
        });

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
