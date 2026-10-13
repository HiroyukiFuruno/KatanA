use super::global_menu_context::GlobalMenuContext;
use crate::app_state::AppAction;
use eframe::egui;

pub(super) struct GlobalAppMenu;

impl GlobalAppMenu {
    pub(super) fn render(ui: &mut egui::Ui, context: &mut GlobalMenuContext<'_>) {
        crate::widgets::MenuButtonOps::show(ui, "KatanA", |ui| {
            let about = context.i18n().menu.about.clone();
            context.action_item(ui, "help.about", &about, AppAction::ToggleAbout);
            let check_updates = context.i18n().menu.check_updates.clone();
            context.action_item(
                ui,
                "help.check_updates",
                &check_updates,
                AppAction::CheckForUpdates,
            );
            ui.separator();
            let quit = context.i18n().menu.quit.clone();
            if ui.button(quit).clicked() {
                context.set_action(ui, AppAction::Quit);
            }
        });
    }
}
