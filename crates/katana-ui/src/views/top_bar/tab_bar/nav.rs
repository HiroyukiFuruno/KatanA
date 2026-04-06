use crate::app_state::AppAction;
use eframe::egui;

pub(crate) struct TabNavButtons<'a> {
    pub icon_bg: egui::Color32,
    pub doc_count: usize,
    pub active_doc_idx: Option<usize>,
    pub open_documents: &'a [katana_core::document::Document],
}

impl<'a> TabNavButtons<'a> {
    pub fn show(self, ui: &mut egui::Ui, tab_action: &mut Option<AppAction>) {
        let nav_enabled = self.doc_count > 1;
        self.render_prev_button(ui, nav_enabled, tab_action);
        self.render_next_button(ui, nav_enabled, tab_action);
    }

    fn render_prev_button(
        &self,
        ui: &mut egui::Ui,
        nav_enabled: bool,
        tab_action: &mut Option<AppAction>,
    ) {
        let label = crate::i18n::I18nOps::get().tab.nav_prev.clone();
        let resp = ui
            .add_enabled(
                nav_enabled,
                egui::Button::image(
                    crate::Icon::TriangleLeft.ui_image(ui, crate::icon::IconSize::Small),
                )
                .fill(self.icon_bg),
            )
            .on_hover_text(label.clone());
        resp.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, nav_enabled, label.clone())
        });

        if resp.clicked()
            && let Some(idx) = self.active_doc_idx
        {
            let new_idx = crate::shell_logic::ShellLogicOps::prev_tab_index(idx, self.doc_count);
            *tab_action = Some(AppAction::SelectDocument(
                self.open_documents[new_idx].path.clone(),
            ));
            ui.memory_mut(|m| m.data.insert_temp(egui::Id::new("scroll_tab_req"), true));
        }
    }

    fn render_next_button(
        &self,
        ui: &mut egui::Ui,
        nav_enabled: bool,
        tab_action: &mut Option<AppAction>,
    ) {
        let label = crate::i18n::I18nOps::get().tab.nav_next.clone();
        let resp = ui
            .add_enabled(
                nav_enabled,
                egui::Button::image(
                    crate::Icon::TriangleRight.ui_image(ui, crate::icon::IconSize::Small),
                )
                .fill(self.icon_bg),
            )
            .on_hover_text(label.clone());
        resp.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, nav_enabled, label.clone())
        });

        if resp.clicked()
            && let Some(idx) = self.active_doc_idx
        {
            let new_idx = crate::shell_logic::ShellLogicOps::next_tab_index(idx, self.doc_count);
            *tab_action = Some(AppAction::SelectDocument(
                self.open_documents[new_idx].path.clone(),
            ));
            ui.memory_mut(|m| m.data.insert_temp(egui::Id::new("scroll_tab_req"), true));
        }
    }
}
