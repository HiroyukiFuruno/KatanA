use super::types::*;
use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use eframe::egui;

const CHEVRON_ICON_SIZE: f32 = 10.0;

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

        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
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
                        seg,
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

            if let Some(ws) = &app.state.workspace.data
                && let Some(katana_core::workspace::TreeEntry::Directory { children, .. }) =
                    crate::views::panels::tree::TreeLogicOps::find_node_in_tree(
                        &ws.tree,
                        current_path,
                    )
            {
                crate::views::panels::explorer::BreadcrumbMenu::new(children, &mut ctx_action)
                    .show(ui);
            }

            if !matches!(ctx_action, crate::app_state::AppAction::None) {
                *breadcrumb_action = Some(ctx_action);
                ui.close();
            }
        });
    }
}
