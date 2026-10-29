use crate::app_state::AppAction;
use crate::state::document::TabGroup;
use eframe::egui;

const GROUP_POPUP_MIN_WIDTH: f32 = 200.0;
const PALETTE_SIZE: f32 = 16.0;
const PALETTE_RADIUS: f32 = 8.0;
const PALETTE_STROKE: f32 = 2.0;
const POPUP_SPACING: f32 = 4.0;
const GROUP_COLORS: [&str; 7] = [
    "#4A90D9", "#D94A4A", "#4AD97A", "#D9A04A", "#9B59B6", "#F1C40F", "#1ABC9C",
];

pub(super) use super::group_header_popup_state::GroupHeaderPopupState;

#[derive(Debug, Clone, PartialEq, Eq)]
enum RenameDecision {
    Confirm(String),
    Cancel,
}

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
        let state = GroupHeaderPopupState::from_popup_id(self.popup_id);
        let i18n = crate::i18n::I18nOps::get();
        let mut draft = Self::load_name_draft(ui.ctx(), state.name_draft_id, &self.g.name);
        let response = self.render_name_field(ui, state, &mut draft);
        Self::store_name_draft(ui.ctx(), state.name_draft_id, &draft);
        let keyboard_decision = Self::keyboard_decision(ui, &response, &draft);
        ui.add_space(POPUP_SPACING);
        self.render_palette(ui);
        ui.add_space(POPUP_SPACING);
        let button_decision =
            Self::render_completion_buttons(ui, &i18n.action.confirm, &i18n.action.cancel, &draft);
        self.finish_rename(ui, state, button_decision.or(keyboard_decision));
        self.render_group_actions(ui, i18n);
        ui.min_rect()
    }

    fn load_name_draft(ctx: &egui::Context, id: egui::Id, saved_name: &str) -> String {
        ctx.data_mut(|data| data.get_temp::<String>(id))
            .unwrap_or_else(|| saved_name.to_owned())
    }

    fn store_name_draft(ctx: &egui::Context, id: egui::Id, name: &str) {
        ctx.data_mut(|data| data.insert_temp(id, name.to_owned()));
    }

    fn render_name_field(
        &self,
        ui: &mut egui::Ui,
        state: GroupHeaderPopupState,
        draft: &mut String,
    ) -> egui::Response {
        let i18n = crate::i18n::I18nOps::get();
        let focus =
            state.request_initial_focus(ui.ctx(), self.inline_rename_group == Some(&self.g.id));
        let response = ui.add(
            egui::TextEdit::singleline(draft)
                .id(self.popup_id.with("group_name_field"))
                .hint_text(&i18n.tab.group_name_placeholder),
        );
        if focus {
            response.request_focus();
        }
        response
    }

    fn keyboard_decision(
        ui: &egui::Ui,
        response: &egui::Response,
        draft: &str,
    ) -> Option<RenameDecision> {
        let has_focus = response.has_focus() || response.lost_focus();
        if has_focus && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
            Some(RenameDecision::Confirm(draft.to_owned()))
        } else if has_focus && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            Some(RenameDecision::Cancel)
        } else {
            None
        }
    }

    fn render_completion_buttons(
        ui: &mut egui::Ui,
        confirm_label: &str,
        cancel_label: &str,
        draft: &str,
    ) -> Option<RenameDecision> {
        let mut decision = None;
        crate::widgets::AlignCenter::new()
            .content(|ui| {
                if ui.button(confirm_label).clicked() {
                    decision = Some(RenameDecision::Confirm(draft.to_owned()));
                }
                if ui.button(cancel_label).clicked() {
                    decision = Some(RenameDecision::Cancel);
                }
            })
            .show(ui);
        decision
    }

    fn finish_rename(
        &mut self,
        ui: &mut egui::Ui,
        state: GroupHeaderPopupState,
        decision: Option<RenameDecision>,
    ) {
        let Some(decision) = decision else {
            return;
        };
        let new_name = match decision {
            RenameDecision::Confirm(name) => Some(name),
            RenameDecision::Cancel => None,
        };
        *self.tab_action = Some(AppAction::FinishInlineRename {
            group_id: self.g.id.clone(),
            new_name,
        });
        state.clear_editor_state(ui.ctx());
        egui::Popup::close_id(ui.ctx(), state.popup_id);
    }

    fn render_palette(&mut self, ui: &mut egui::Ui) {
        let mut selected = None;
        let current = self.g.color_hex.clone();
        crate::widgets::AlignCenter::new()
            .shrink_to_fit(true)
            .content(|ui| selected = Self::render_palette_chips(ui, &current))
            .show(ui);
        if let Some(new_color) = selected {
            *self.tab_action = Some(AppAction::RecolorTabGroup {
                group_id: self.g.id.clone(),
                new_color,
            });
        }
    }

    fn render_palette_chips(ui: &mut egui::Ui, current: &str) -> Option<String> {
        GROUP_COLORS
            .into_iter()
            .find(|color| Self::render_palette_chip(ui, color, current))
            .map(str::to_owned)
    }

    fn render_palette_chip(ui: &mut egui::Ui, color: &str, current: &str) -> bool {
        let color32 = egui::Color32::from_hex(color).unwrap_or_default();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(PALETTE_SIZE, PALETTE_SIZE), egui::Sense::click());
        ui.painter()
            .circle_filled(rect.center(), PALETTE_RADIUS, color32);
        if current == color {
            ui.painter().circle_stroke(
                rect.center(),
                PALETTE_RADIUS,
                egui::Stroke::new(PALETTE_STROKE, ui.visuals().text_color()),
            );
        }
        response.clicked()
    }

    fn render_group_actions(&mut self, ui: &mut egui::Ui, i18n: &crate::i18n::I18nMessages) {
        ui.separator();
        self.render_ungroup_button(ui, &i18n.tab.ungroup);
        self.render_close_button(ui, &i18n.tab.close_group);
    }

    fn render_ungroup_button(&mut self, ui: &mut egui::Ui, label: &str) {
        if ui.button(label).clicked() {
            *self.tab_action = Some(AppAction::UngroupTabGroup(self.g.id.clone()));
            egui::Popup::close_id(ui.ctx(), self.popup_id);
        }
    }

    fn render_close_button(&mut self, ui: &mut egui::Ui, label: &str) {
        if ui.button(label).clicked() {
            *self.tab_action = Some(AppAction::CloseTabGroup(self.g.id.clone()));
            egui::Popup::close_id(ui.ctx(), self.popup_id);
        }
    }
}
