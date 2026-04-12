use crate::app_state::AppAction;
use crate::i18n::I18nOps;

pub mod types;
pub use types::*;
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
                default_shortcuts: &["primary+,"],
            },
            /* WHY: File Group */
            CommandInventoryItem {
                id: "file.open_workspace",
                action: AppAction::PickOpenWorkspace,
                group: CommandGroup::File,
                label: || I18nOps::get().menu.open_workspace.clone(),
                is_available: |_| true,
                default_shortcuts: &["primary+O"],
            },
            CommandInventoryItem {
                id: "file.close_workspace",
                action: AppAction::CloseWorkspace,
                group: CommandGroup::File,
                label: || I18nOps::get().menu.close_workspace.clone(),
                is_available: |state| state.workspace.data.is_some(),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "file.save",
                action: AppAction::SaveDocument,
                group: CommandGroup::File,
                label: || I18nOps::get().menu.save.clone(),
                is_available: |state| state.document.active_doc_idx.is_some(),
                default_shortcuts: &["primary+S"],
            },
            CommandInventoryItem {
                id: "file.close_document",
                action: AppAction::CloseActiveDocument,
                group: CommandGroup::File,
                label: || I18nOps::get().menu.close_document.clone(),
                is_available: |state| state.document.active_doc_idx.is_some(),
                default_shortcuts: &["primary+W"],
            },
            CommandInventoryItem {
                id: "file.restore_closed",
                action: AppAction::RestoreClosedDocument,
                group: CommandGroup::File,
                label: || I18nOps::get().menu.restore_closed.clone(),
                is_available: |state| !state.document.recently_closed_tabs.is_empty(),
                default_shortcuts: &["primary+Shift+T"],
            },
            /* WHY: View Group */
            CommandInventoryItem {
                id: "view.command_palette",
                action: AppAction::ToggleCommandPalette,
                group: CommandGroup::View,
                label: || I18nOps::get().menu.command_palette.clone(),
                is_available: |_| true,
                default_shortcuts: &["primary+P", "primary+Shift+P", "primary+K"],
            },
            CommandInventoryItem {
                id: "view.explorer",
                action: AppAction::ToggleExplorer,
                group: CommandGroup::View,
                label: || I18nOps::get().search.command_explorer.clone(),
                is_available: |_| true,
                default_shortcuts: &["primary+B"],
            },
            CommandInventoryItem {
                id: "view.refresh_explorer",
                action: AppAction::RefreshExplorer,
                group: CommandGroup::View,
                label: || I18nOps::get().search.command_refresh_explorer.clone(),
                is_available: |state| state.workspace.data.is_some(),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "view.close_all",
                action: AppAction::CloseAllDocuments,
                group: CommandGroup::View,
                label: || I18nOps::get().search.command_close_all.clone(),
                is_available: |state| !state.document.open_documents.is_empty(),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "view.search_modal",
                action: AppAction::ToggleSearchModal,
                group: CommandGroup::View,
                label: || I18nOps::get().search.command_global_search.clone(),
                is_available: |_| true,
                default_shortcuts: &["primary+Shift+F"],
            },
            CommandInventoryItem {
                id: "view.doc_search",
                action: AppAction::ToggleDocSearch,
                group: CommandGroup::View,
                label: || I18nOps::get().search.command_doc_search.clone(),
                is_available: |state| state.document.active_doc_idx.is_some(),
                default_shortcuts: &["primary+F"],
            },
            CommandInventoryItem {
                id: "view.refresh_document",
                action: AppAction::RefreshDocument { is_manual: true },
                group: CommandGroup::View,
                label: || I18nOps::get().search.command_refresh_document.clone(),
                is_available: |state| state.document.active_doc_idx.is_some(),
                default_shortcuts: &["primary+R"],
            },
            /* WHY: Help Group */
            CommandInventoryItem {
                id: "help.about",
                action: AppAction::ToggleAbout,
                group: CommandGroup::Help,
                label: || I18nOps::get().menu.about.clone(),
                is_available: |_| true,
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "help.check_updates",
                action: AppAction::CheckForUpdates,
                group: CommandGroup::Help,
                label: || I18nOps::get().menu.check_updates.clone(),
                is_available: |_| true,
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "help.release_notes",
                action: AppAction::ShowReleaseNotes,
                group: CommandGroup::Help,
                label: || I18nOps::get().menu.release_notes.clone(),
                is_available: |_| true,
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "help.welcome_screen",
                action: AppAction::OpenWelcomeScreen,
                group: CommandGroup::Help,
                label: || I18nOps::get().menu.welcome_screen.clone(),
                is_available: |_| true,
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "help.user_guide",
                action: AppAction::OpenUserGuide,
                group: CommandGroup::Help,
                label: || I18nOps::get().menu.user_guide.clone(),
                is_available: |_| true,
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "help.demo",
                action: AppAction::OpenHelpDemo,
                group: CommandGroup::Help,
                label: || I18nOps::get().menu.demo.clone(),
                is_available: |_| true,
                default_shortcuts: &["primary+alt+D"],
            },
            CommandInventoryItem {
                id: "help.github",
                action: AppAction::OpenGitHub,
                group: CommandGroup::Help,
                label: || I18nOps::get().menu.github.clone(),
                is_available: |_| true,
                default_shortcuts: &[],
            },
        ]
    }
}
