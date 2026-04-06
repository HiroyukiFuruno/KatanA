use crate::app_state::AppAction;
use crate::state::document::TabGroup;
use eframe::egui;
use katana_core::document::Document;

const GROUP_DOT_SIZE: f32 = 12.0;
const GROUP_DOT_RADIUS: f32 = 4.0;
const DEFAULT_GROUP_COLOR: &str = "#4A90D9";

pub(crate) struct TabContextMenu<'a> {
    pub idx: usize,
    pub doc: &'a Document,
    pub tab_groups: &'a [TabGroup],
    pub recently_closed_tabs_empty: bool,
}

impl<'a> TabContextMenu<'a> {
    pub fn show(self, ui: &mut egui::Ui, tab_action: &mut Option<AppAction>) {
        let i18n = crate::i18n::I18nOps::get();
        self.render_close_actions(ui, tab_action, i18n);
        ui.separator();
        self.render_pin_action(ui, tab_action, i18n);
        if !self.doc.is_pinned {
            self.render_group_actions(ui, tab_action, i18n);
        }
        if !self.recently_closed_tabs_empty {
            ui.separator();
            if ui.button(&i18n.tab.restore_closed).clicked() {
                *tab_action = Some(AppAction::RestoreClosedDocument);
                ui.close();
            }
        }
    }

    fn render_close_actions(
        &self,
        ui: &mut egui::Ui,
        tab_action: &mut Option<AppAction>,
        i18n: &crate::i18n::I18nMessages,
    ) {
        if ui.button(&i18n.tab.close).clicked() {
            *tab_action = Some(AppAction::CloseDocument(self.idx));
            ui.close();
        }
        if ui.button(&i18n.tab.close_others).clicked() {
            *tab_action = Some(AppAction::CloseOtherDocuments(self.idx));
            ui.close();
        }
        if ui.button(&i18n.tab.close_all).clicked() {
            *tab_action = Some(AppAction::CloseAllDocuments);
            ui.close();
        }
        if ui.button(&i18n.tab.close_right).clicked() {
            *tab_action = Some(AppAction::CloseDocumentsToRight(self.idx));
            ui.close();
        }
        if ui.button(&i18n.tab.close_left).clicked() {
            *tab_action = Some(AppAction::CloseDocumentsToLeft(self.idx));
            ui.close();
        }
    }

    fn render_pin_action(
        &self,
        ui: &mut egui::Ui,
        tab_action: &mut Option<AppAction>,
        i18n: &crate::i18n::I18nMessages,
    ) {
        let pin_label = if self.doc.is_pinned {
            &i18n.tab.unpin
        } else {
            &i18n.tab.pin
        };
        if ui.button(pin_label).clicked() {
            *tab_action = Some(AppAction::TogglePinDocument(self.idx));
            ui.close();
        }
    }

    fn render_group_actions(
        &self,
        ui: &mut egui::Ui,
        tab_action: &mut Option<AppAction>,
        i18n: &crate::i18n::I18nMessages,
    ) {
        let doc_str = self.doc.path.display().to_string();
        let is_in_any = self.tab_groups.iter().any(|g| g.members.contains(&doc_str));
        if !is_in_any {
            self.render_create_group(ui, tab_action, i18n);
        } else {
            self.render_remove_from_group(ui, tab_action, i18n);
        }
        self.render_add_to_group_submenu(ui, tab_action, i18n, &doc_str, is_in_any);
    }

    fn render_create_group(
        &self,
        ui: &mut egui::Ui,
        tab_action: &mut Option<AppAction>,
        i18n: &crate::i18n::I18nMessages,
    ) {
        if ui.button(&i18n.tab.create_new_group).clicked() {
            *tab_action = Some(AppAction::CreateTabGroup {
                name: String::new(),
                color_hex: DEFAULT_GROUP_COLOR.to_string(),
                initial_member: self.doc.path.clone(),
            });
            ui.close();
        }
    }

    fn render_remove_from_group(
        &self,
        ui: &mut egui::Ui,
        tab_action: &mut Option<AppAction>,
        i18n: &crate::i18n::I18nMessages,
    ) {
        if ui.button(&i18n.tab.remove_from_group).clicked() {
            *tab_action = Some(AppAction::RemoveTabFromGroup(self.doc.path.clone()));
            ui.close();
        }
    }

    fn render_add_to_group_submenu(
        &self,
        ui: &mut egui::Ui,
        tab_action: &mut Option<AppAction>,
        i18n: &crate::i18n::I18nMessages,
        doc_str: &str,
        is_in_any: bool,
    ) {
        let has_other = self
            .tab_groups
            .iter()
            .any(|g| !g.members.contains(&doc_str.to_string()));
        if !has_other && !is_in_any {
            return;
        }
        egui::menu::menu_button(ui, &i18n.tab.add_to_group, |ui| {
            if is_in_any {
                self.render_create_group(ui, tab_action, i18n);
                ui.separator();
            }
            for g in self.tab_groups {
                if g.members.contains(&doc_str.to_string()) {
                    continue;
                }
                self.render_group_entry(ui, tab_action, g, doc_str);
            }
        });
    }

    fn render_group_entry(
        &self,
        ui: &mut egui::Ui,
        tab_action: &mut Option<AppAction>,
        g: &TabGroup,
        doc_str: &str,
    ) {
        crate::widgets::AlignCenter::new()
            .shrink_to_fit(true)
            .content(|ui: &mut egui::Ui| {
                let color32 = egui::Color32::from_hex(&g.color_hex)
                    .unwrap_or(ui.visuals().widgets.active.bg_fill);
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(GROUP_DOT_SIZE, GROUP_DOT_SIZE),
                    egui::Sense::hover(),
                );
                ui.painter()
                    .circle_filled(rect.center(), GROUP_DOT_RADIUS, color32);
                if ui
                    .add(egui::Button::new(&g.name).frame_when_inactive(true))
                    .clicked()
                {
                    *tab_action = Some(AppAction::AddTabToGroup {
                        group_id: g.id.clone(),
                        member: self.doc.path.clone(),
                    });
                    ui.close();
                }
            })
            .show(ui);
        let _ = doc_str;
    }
}
