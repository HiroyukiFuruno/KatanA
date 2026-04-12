use crate::app_state::{AppAction, AppState};
use crate::i18n::I18nOps;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandGroup {
    App,
    File,
    View,
    Help,
}

impl CommandGroup {
    pub fn localized_name(self) -> String {
        let i18n = I18nOps::get();
        match self {
            Self::App => "KatanA".to_string(), // WHY: Main app menu equivalent
            Self::File => i18n.menu.file.clone(),
            Self::View => i18n.menu.view.clone(),
            Self::Help => i18n.menu.help.clone(),
        }
    }
}

pub struct CommandInventoryItem {
    pub id: &'static str,
    pub action: AppAction,
    pub group: CommandGroup,
    pub label: fn() -> String,
    pub is_available: fn(&AppState) -> bool,
}

pub struct CommandInventory;

impl CommandInventory {
    pub fn all() -> Vec<CommandInventoryItem> {
        vec![
            /* WHY: App Group */
            CommandInventoryItem {
                id: "app.settings",
                action: AppAction::ToggleSettings,
                group: CommandGroup::App,
                label: || I18nOps::get().menu.settings.clone(),
                is_available: |_| true,
            },
            /* WHY: File Group */
            CommandInventoryItem {
                id: "file.open_workspace",
                action: AppAction::PickOpenWorkspace,
                group: CommandGroup::File,
                label: || I18nOps::get().menu.open_workspace.clone(),
                is_available: |_| true,
            },
            CommandInventoryItem {
                id: "file.close_workspace",
                action: AppAction::CloseWorkspace,
                group: CommandGroup::File,
                label: || I18nOps::get().menu.close_workspace.clone(),
                is_available: |state| state.workspace.data.is_some(),
            },
            CommandInventoryItem {
                id: "file.save",
                action: AppAction::SaveDocument,
                group: CommandGroup::File,
                label: || I18nOps::get().menu.save.clone(),
                is_available: |state| state.document.active_doc_idx.is_some(),
            },
            /* WHY: View Group */
            CommandInventoryItem {
                id: "view.command_palette",
                action: AppAction::ToggleCommandPalette,
                group: CommandGroup::View,
                label: || I18nOps::get().menu.command_palette.clone(),
                is_available: |_| true,
            },
            CommandInventoryItem {
                id: "view.explorer",
                action: AppAction::ToggleExplorer,
                group: CommandGroup::View,
                label: || I18nOps::get().search.command_explorer.clone(),
                is_available: |_| true,
            },
            CommandInventoryItem {
                id: "view.refresh_explorer",
                action: AppAction::RefreshExplorer,
                group: CommandGroup::View,
                label: || I18nOps::get().search.command_refresh_explorer.clone(),
                is_available: |state| state.workspace.data.is_some(),
            },
            CommandInventoryItem {
                id: "view.close_all",
                action: AppAction::CloseAllDocuments,
                group: CommandGroup::View,
                label: || I18nOps::get().search.command_close_all.clone(),
                is_available: |state| !state.document.open_documents.is_empty(),
            },
            /* WHY: Help Group */
            CommandInventoryItem {
                id: "help.about",
                action: AppAction::ToggleAbout,
                group: CommandGroup::Help,
                label: || I18nOps::get().menu.about.clone(),
                is_available: |_| true,
            },
            CommandInventoryItem {
                id: "help.check_updates",
                action: AppAction::CheckForUpdates,
                group: CommandGroup::Help,
                label: || I18nOps::get().menu.check_updates.clone(),
                is_available: |_| true,
            },
            CommandInventoryItem {
                id: "help.release_notes",
                action: AppAction::ShowReleaseNotes,
                group: CommandGroup::Help,
                label: || I18nOps::get().menu.release_notes.clone(),
                is_available: |_| true,
            },
            CommandInventoryItem {
                id: "help.welcome_screen",
                action: AppAction::OpenWelcomeScreen,
                group: CommandGroup::Help,
                label: || I18nOps::get().menu.welcome_screen.clone(),
                is_available: |_| true,
            },
            CommandInventoryItem {
                id: "help.user_guide",
                action: AppAction::OpenUserGuide,
                group: CommandGroup::Help,
                label: || I18nOps::get().menu.user_guide.clone(),
                is_available: |_| true,
            },
            CommandInventoryItem {
                id: "help.demo",
                action: AppAction::OpenHelpDemo,
                group: CommandGroup::Help,
                label: || I18nOps::get().menu.demo.clone(),
                is_available: |_| true,
            },
            CommandInventoryItem {
                id: "help.github",
                action: AppAction::OpenGitHub,
                group: CommandGroup::Help,
                label: || I18nOps::get().menu.github.clone(),
                is_available: |_| true,
            },
        ]
    }
}
