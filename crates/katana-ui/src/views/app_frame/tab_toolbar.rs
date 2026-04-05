use super::types::*;
use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use crate::shell_logic::ShellLogicOps;
use eframe::egui;

const CHEVRON_ICON_SIZE: f32 = 10.0;

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
                let d_path = doc.path.to_string_lossy();
                let is_changelog = d_path.starts_with("Katana://ChangeLog");
                (doc.path.clone(), is_changelog)
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
                if !is_changelog {
                    if let Some(a) = Self::render_breadcrumbs(ui, app, &doc_path) {
                        out_action = Some(a);
                    }
                }

                if let Some(a) = Self::render_view_mode_bar(ui, app, is_changelog) {
                    out_action = Some(a);
                }
            },
        );

        if app.state.search.doc_search_open {
            if let Some(a) = crate::views::top_bar::DocSearchBar::show(ui, &mut app.state.search) {
                out_action = Some(a);
            }
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

impl<'a> Breadcrumbs<'a> {
    pub(crate) fn new(
        app: &'a KatanaApp,
        rel: &'a str,
        ws_root: Option<&'a std::path::Path>,
    ) -> Self {
        Self { app, rel, ws_root }
    }

    pub(crate) fn show(self, ui: &mut egui::Ui) -> Option<AppAction> {
        let app = self.app;
        let rel = self.rel;
        let ws_root = self.ws_root;
        let mut breadcrumb_action = None;

        ui.horizontal_centered(|ui| {
            let segments: Vec<&str> = rel.split('/').collect();
            let mut current_path = ws_root.map(std::path::PathBuf::from).unwrap_or_default();

            for (i, seg) in segments.iter().enumerate() {
                if i > 0 {
                    ui.add(
                        egui::Image::new(crate::Icon::ChevronRight.uri())
                            .tint(ui.visuals().text_color())
                            .fit_to_exact_size(egui::vec2(CHEVRON_ICON_SIZE, CHEVRON_ICON_SIZE)),
                    );
                }

                if ws_root.is_none() {
                    ui.label(egui::RichText::new(*seg).small());
                    continue;
                }

                current_path = current_path.join(seg);
                let is_last = i == segments.len() - 1;

                if is_last {
                    ui.add(
                        egui::Label::new(egui::RichText::new(*seg).small())
                            .sense(egui::Sense::hover()),
                    );
                } else {
                    Self::render_breadcrumb_segment(
                        ui,
                        *seg,
                        app,
                        &current_path,
                        &mut breadcrumb_action,
                    );
                }
            }
        });

        breadcrumb_action
    }

    fn render_breadcrumb_segment(
        ui: &mut egui::Ui,
        seg: &str,
        app: &KatanaApp,
        current_path: &std::path::Path,
        breadcrumb_action: &mut Option<AppAction>,
    ) {
        crate::widgets::MenuButtonOps::show(ui, egui::RichText::new(seg).small(), |ui| {
            let mut ctx_action = crate::app_state::AppAction::None;

            if let Some(ws) = &app.state.workspace.data {
                if let Some(katana_core::workspace::TreeEntry::Directory { children, .. }) =
                    crate::views::panels::tree::TreeLogicOps::find_node_in_tree(
                        &ws.tree,
                        current_path,
                    )
                {
                    crate::views::panels::workspace::BreadcrumbMenu::new(children, &mut ctx_action)
                        .show(ui);
                }
            }

            if !matches!(ctx_action, crate::app_state::AppAction::None) {
                *breadcrumb_action = Some(ctx_action);
                ui.close();
            }
        });
    }
}
