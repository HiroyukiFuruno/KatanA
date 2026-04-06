use crate::app_state::{AppAction, ViewMode};
use eframe::egui;

pub(super) struct ModeButtons<'a> {
    pub mode: &'a mut ViewMode,
    pub is_split: bool,
}

impl<'a> ModeButtons<'a> {
    pub fn show(self, ui: &mut egui::Ui) {
        let i18n = crate::i18n::I18nOps::get();
        let ModeButtons { mode, is_split } = self;
        if ui
            .add(
                egui::Button::selectable(is_split, i18n.view_mode.split.clone())
                    .frame_when_inactive(true),
            )
            .clicked()
            && !is_split
        {
            *mode = ViewMode::Split;
        }
        if ui
            .add(
                egui::Button::selectable(*mode == ViewMode::CodeOnly, i18n.view_mode.code.clone())
                    .frame_when_inactive(true),
            )
            .clicked()
        {
            *mode = ViewMode::CodeOnly;
        }
        if ui
            .add(
                egui::Button::selectable(
                    *mode == ViewMode::PreviewOnly,
                    i18n.view_mode.preview.clone(),
                )
                .frame_when_inactive(true),
            )
            .clicked()
        {
            *mode = ViewMode::PreviewOnly;
        }
    }
}

pub(super) fn render_refresh_button(
    ui: &mut egui::Ui,
    icon_bg: egui::Color32,
    action: &mut Option<AppAction>,
) {
    const REFRESH_BTN_SIZE: f32 = 24.0;
    const REFRESH_BTN_OFFSET_Y: f32 = 2.0;
    let refresh_btn =
        egui::Button::image(crate::Icon::Refresh.ui_image(ui, crate::icon::IconSize::Medium))
            .fill(icon_bg);
    ui.allocate_ui(egui::vec2(REFRESH_BTN_SIZE, REFRESH_BTN_SIZE), |ui| {
        ui.add_space(REFRESH_BTN_OFFSET_Y);
        if ui
            .add(refresh_btn)
            .on_hover_text(crate::i18n::I18nOps::get().action.refresh_document.clone())
            .clicked()
        {
            *action = Some(AppAction::RefreshDocument { is_manual: true });
        }
    });
}
