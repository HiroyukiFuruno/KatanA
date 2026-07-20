use crate::app_state::AppAction;
use crate::state::url_tab::UrlTabState;

const URL_ACTIONS_WIDTH: f32 = 92.0;
const MIN_URL_INPUT_WIDTH: f32 = 120.0;

pub(crate) struct UrlSourceBar;

impl UrlSourceBar {
    pub(crate) fn show(ui: &mut egui::Ui, state: &mut UrlTabState) -> Option<AppAction> {
        let mut action = None;
        crate::widgets::AlignCenter::new()
            .content(|ui| {
                let response = ui.add(egui::TextEdit::singleline(&mut state.input).desired_width(
                    (ui.available_width() - URL_ACTIONS_WIDTH).max(MIN_URL_INPUT_WIDTH),
                ));
                let open = ui
                    .button(crate::i18n::I18nOps::get().action.open.clone())
                    .clicked()
                    || (response.lost_focus()
                        && ui.input(|input| input.key_pressed(egui::Key::Enter)));
                if open && !state.input.trim().is_empty() {
                    action = Some(AppAction::OpenUrl(state.input.clone()));
                }

                if !state.history.is_empty() {
                    egui::ComboBox::from_id_salt("url_source_history")
                        .selected_text("...")
                        .show_ui(ui, |ui| {
                            for url in state.history.clone() {
                                if ui
                                    .add(
                                        egui::Button::selectable(false, &url)
                                            .frame_when_inactive(true),
                                    )
                                    .clicked()
                                {
                                    state.input = url;
                                }
                            }
                        });
                }
            })
            .show(ui);
        action
    }
}
