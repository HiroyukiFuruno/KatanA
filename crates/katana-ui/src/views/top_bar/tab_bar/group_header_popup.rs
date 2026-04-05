use crate::app_state::AppAction;
use crate::state::document::TabGroup;
use eframe::egui;

const GROUP_POPUP_MIN_WIDTH: f32 = 200.0;
const PALETTE_SIZE: f32 = 16.0;
const PALETTE_RADIUS: f32 = 8.0;
const PALETTE_STROKE: f32 = 2.0;
const POPUP_SPACING: f32 = 4.0;

pub(crate) struct GroupHeaderPopup<'a> {
    pub g: &'a TabGroup,
    pub inline_rename_group: Option<&'a String>,
    pub tab_action: &'a mut Option<AppAction>,
    pub popup_id: egui::Id,
}

impl<'a> GroupHeaderPopup<'a> {
    #[allow(deprecated)]
    pub fn show(mut self, ui: &mut egui::Ui) -> egui::Rect {
        ui.set_min_width(GROUP_POPUP_MIN_WIDTH);
        let i18n = crate::i18n::I18nOps::get();
        let mut new_name = self.g.name.clone();
        let mut new_color = self.g.color_hex.clone();

        self.render_name_field(ui, &mut new_name);
        ui.add_space(POPUP_SPACING);
        self.render_palette(ui, &mut new_color);
        ui.add_space(POPUP_SPACING);
        self.apply_changes(ui, &new_name, &new_color, &i18n);
        ui.min_rect()
    }

    fn render_name_field(&self, ui: &mut egui::Ui, new_name: &mut String) {
        let i18n = crate::i18n::I18nOps::get();
        crate::widgets::AlignCenter::new()
            .shrink_to_fit(true)
            .content(|ui: &mut egui::Ui| {
                let resp = ui.add(
                    egui::TextEdit::singleline(new_name)
                        .hint_text(&i18n.tab.group_name_placeholder),
                );
                if self.inline_rename_group == Some(&self.g.id) {
                    resp.request_focus();
                }
            })
            .show(ui);
    }

    fn render_palette(&self, ui: &mut egui::Ui, new_color: &mut String) {
        let colors = [
            "#4A90D9", "#D94A4A", "#4AD97A", "#D9A04A", "#9B59B6", "#F1C40F", "#1ABC9C",
        ];
        crate::widgets::AlignCenter::new()
            .shrink_to_fit(true)
            .content(|ui: &mut egui::Ui| {
                for c in colors {
                    let color32 = egui::Color32::from_hex(c).unwrap_or_default();
                    let (rect, resp) = ui.allocate_exact_size(
                        egui::vec2(PALETTE_SIZE, PALETTE_SIZE),
                        egui::Sense::click(),
                    );
                    ui.painter().circle_filled(rect.center(), PALETTE_RADIUS, color32);
                    if *new_color == c {
                        ui.painter().circle_stroke(
                            rect.center(),
                            PALETTE_RADIUS,
                            egui::Stroke::new(PALETTE_STROKE, ui.visuals().text_color()),
                        );
                    }
                    if resp.clicked() {
                        *new_color = c.to_string();
                    }
                }
            })
            .show(ui);
    }

    fn apply_changes(
        &mut self,
        ui: &mut egui::Ui,
        new_name: &str,
        new_color: &str,
        i18n: &crate::i18n::I18nMessages,
    ) {
        if new_name != self.g.name {
            *self.tab_action = Some(AppAction::RenameTabGroup {
                group_id: self.g.id.clone(),
                new_name: new_name.to_string(),
            });
        }
        if new_color != self.g.color_hex {
            *self.tab_action = Some(AppAction::RecolorTabGroup {
                group_id: self.g.id.clone(),
                new_color: new_color.to_string(),
            });
        }
        ui.separator();
        if ui.button(&i18n.tab.ungroup).clicked() {
            *self.tab_action = Some(AppAction::UngroupTabGroup(self.g.id.clone()));
            ui.memory_mut(|mem| mem.close_popup(self.popup_id));
        }
        if ui.button(&i18n.tab.close_group).clicked() {
            *self.tab_action = Some(AppAction::CloseTabGroup(self.g.id.clone()));
            ui.memory_mut(|mem| mem.close_popup(self.popup_id));
        }
    }
}
