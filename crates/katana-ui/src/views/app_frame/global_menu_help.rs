use super::global_menu_context::GlobalMenuContext;
use crate::app_state::AppAction;
use eframe::egui;

pub(super) struct GlobalHelpMenu;

impl GlobalHelpMenu {
    pub(super) fn render(ui: &mut egui::Ui, context: &mut GlobalMenuContext<'_>) {
        let menu_label = context.i18n().menu.help.clone();
        crate::widgets::MenuButtonOps::show(ui, &menu_label, |ui| {
            let release_notes = context.i18n().menu.release_notes.clone();
            context.action_item(
                ui,
                "help.release_notes",
                &release_notes,
                AppAction::ShowReleaseNotes,
            );
            ui.separator();
            Self::render_documentation_items(ui, context);
            ui.separator();
            let github = context.i18n().menu.github.clone();
            context.action_item(ui, "help.github", &github, AppAction::OpenGitHub);
        });
    }

    fn render_documentation_items(ui: &mut egui::Ui, context: &mut GlobalMenuContext<'_>) {
        let welcome_screen = context.i18n().menu.welcome_screen.clone();
        context.action_item(
            ui,
            "help.welcome_screen",
            &welcome_screen,
            AppAction::OpenWelcomeScreen,
        );
        let user_guide = context.i18n().menu.user_guide.clone();
        context.action_item(ui, "help.user_guide", &user_guide, AppAction::OpenUserGuide);
        let demo = context.i18n().menu.demo.clone();
        context.shortcut_action_item(ui, "help.demo", &demo, "open_demo", AppAction::OpenHelpDemo);
    }
}
