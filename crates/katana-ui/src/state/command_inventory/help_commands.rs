use super::{CommandGroup, CommandInventoryItem};
use crate::app_state::AppAction;
use crate::i18n::I18nOps;
use crate::state::shortcut_context::ShortcutContext;

pub struct HelpCommands;

impl HelpCommands {
    pub fn get() -> Vec<CommandInventoryItem> {
        vec![
            /* WHY: Help Group */
            CommandInventoryItem {
                id: "help.about",
                action: AppAction::ToggleAbout,
                group: CommandGroup::Help,
                context: ShortcutContext::Global,
                label: || I18nOps::get().menu.about.clone(),
                is_available: |_| true,
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "help.check_updates",
                action: AppAction::CheckForUpdates,
                group: CommandGroup::Help,
                context: ShortcutContext::Global,
                label: || I18nOps::get().menu.check_updates.clone(),
                is_available: |_| true,
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "help.release_notes",
                action: AppAction::ShowReleaseNotes,
                group: CommandGroup::Help,
                context: ShortcutContext::Global,
                label: || I18nOps::get().menu.release_notes.clone(),
                is_available: |_| true,
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "help.welcome_screen",
                action: AppAction::OpenWelcomeScreen,
                group: CommandGroup::Help,
                context: ShortcutContext::Global,
                label: || I18nOps::get().menu.welcome_screen.clone(),
                is_available: |_| true,
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "help.user_guide",
                action: AppAction::OpenUserGuide,
                group: CommandGroup::Help,
                context: ShortcutContext::Global,
                label: || I18nOps::get().menu.user_guide.clone(),
                is_available: |_| true,
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "help.demo",
                action: AppAction::OpenHelpDemo,
                group: CommandGroup::Help,
                context: ShortcutContext::Global,
                label: || I18nOps::get().menu.demo.clone(),
                is_available: |_| true,
                default_shortcuts: &["primary+alt+D"],
            },
            CommandInventoryItem {
                id: "help.github",
                action: AppAction::OpenGitHub,
                group: CommandGroup::Help,
                context: ShortcutContext::Global,
                label: || I18nOps::get().menu.github.clone(),
                is_available: |_| true,
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "help.website",
                action: AppAction::OpenOfficialWebsite,
                group: CommandGroup::Help,
                context: ShortcutContext::Global,
                label: || I18nOps::get().menu.website.clone(),
                is_available: |_| true,
                default_shortcuts: &[],
            },
        ]
    }
}
