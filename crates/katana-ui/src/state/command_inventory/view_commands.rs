use super::{CommandGroup, CommandInventoryItem};
use crate::app_state::AppAction;
use crate::i18n::I18nOps;
use crate::state::shortcut_context::ShortcutContext;

pub struct ViewCommands;

impl ViewCommands {
    pub fn get() -> Vec<CommandInventoryItem> {
        vec![
            /* WHY: View Group */
            CommandInventoryItem {
                id: "view.katana_command_palette",
                action: AppAction::ToggleKatanaCommandPalette,
                group: CommandGroup::View,
                context: ShortcutContext::Global,
                label: || format!("Katana {}", I18nOps::get().menu.command_palette),
                is_available: |_| true,
                default_shortcuts: &["primary+Shift+P"],
            },
            CommandInventoryItem {
                id: "view.command_palette",
                action: AppAction::ToggleCommandPalette,
                group: CommandGroup::View,
                context: ShortcutContext::Global,
                label: || I18nOps::get().menu.command_palette.clone(),
                is_available: |_| true,
                default_shortcuts: &["primary+P"],
            },
            CommandInventoryItem {
                id: "view.explorer",
                action: AppAction::ToggleExplorer,
                group: CommandGroup::View,
                context: ShortcutContext::Global,
                label: || I18nOps::get().search.command_explorer.clone(),
                is_available: |_| true,
                default_shortcuts: &["primary+B"],
            },
            CommandInventoryItem {
                id: "view.refresh_explorer",
                action: AppAction::RefreshExplorer,
                group: CommandGroup::View,
                context: ShortcutContext::Global,
                label: || I18nOps::get().search.command_refresh_explorer.clone(),
                is_available: |state| state.workspace.data.is_some(),
                default_shortcuts: &["primary+Shift+R"],
            },
            CommandInventoryItem {
                id: "view.toggle_filter",
                action: AppAction::ToggleExplorerFilter,
                group: CommandGroup::View,
                context: ShortcutContext::Global,
                label: || I18nOps::get().search.command_toggle_filter.clone(),
                is_available: |state| state.workspace.data.is_some(),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "view.close_all",
                action: AppAction::CloseAllDocuments,
                group: CommandGroup::View,
                context: ShortcutContext::Global,
                label: || I18nOps::get().search.command_close_all.clone(),
                is_available: |state| !state.document.open_documents.is_empty(),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "view.search_modal",
                action: AppAction::ToggleSearchModal,
                group: CommandGroup::View,
                context: ShortcutContext::Global,
                label: || I18nOps::get().search.command_global_search.clone(),
                is_available: |_| true,
                default_shortcuts: &["primary+Shift+F"],
            },
            CommandInventoryItem {
                id: "view.doc_search",
                action: AppAction::ToggleDocSearch,
                group: CommandGroup::View,
                context: ShortcutContext::Global,
                label: || I18nOps::get().search.command_doc_search.clone(),
                is_available: |state| {
                    if state.document.active_doc_idx.is_none() {
                        return false;
                    }
                    /* WHY: Virtual docs (Welcome, Guide, ChangeLog) do not support in-doc search. */
                    !state.active_document().is_some_and(|d| {
                        let p = d.path.to_string_lossy();
                        p.starts_with("Katana://Welcome")
                            || p.starts_with("Katana://Guide")
                            || p.starts_with("Katana://ChangeLog")
                    })
                },
                default_shortcuts: &["primary+F"],
            },
            CommandInventoryItem {
                id: "view.refresh_document",
                action: AppAction::RefreshDocument { is_manual: true },
                group: CommandGroup::View,
                context: ShortcutContext::Global,
                label: || I18nOps::get().search.command_refresh_document.clone(),
                is_available: |state| state.document.active_doc_idx.is_some(),
                default_shortcuts: &["primary+R"],
            },
            CommandInventoryItem {
                id: "view.zoom_in",
                action: AppAction::ZoomIn,
                group: CommandGroup::View,
                context: ShortcutContext::Global,
                label: || I18nOps::get().search.command_zoom_in.clone(),
                is_available: |_| true,
                default_shortcuts: &["primary+Shift+Plus", "primary+Plus"],
            },
            CommandInventoryItem {
                id: "view.zoom_out",
                action: AppAction::ZoomOut,
                group: CommandGroup::View,
                context: ShortcutContext::Global,
                label: || I18nOps::get().search.command_zoom_out.clone(),
                is_available: |_| true,
                default_shortcuts: &["primary+Shift+Minus", "primary+Minus"],
            },
            CommandInventoryItem {
                id: "view.toggle_split_mode",
                action: AppAction::ToggleSplitMode,
                group: CommandGroup::Behavior,
                context: ShortcutContext::Global,
                label: || I18nOps::get().search.command_toggle_split.clone(),
                is_available: |_| true,
                default_shortcuts: &["primary+Shift+\\"],
            },
            CommandInventoryItem {
                id: "view.toggle_code_preview",
                action: AppAction::ToggleCodePreview,
                group: CommandGroup::Behavior,
                context: ShortcutContext::Global,
                label: || I18nOps::get().search.command_toggle_code_preview.clone(),
                is_available: |_| true,
                default_shortcuts: &["primary+\\"],
            },
            CommandInventoryItem {
                id: "view.tab_next",
                action: AppAction::SelectNextTab,
                group: CommandGroup::Behavior,
                context: ShortcutContext::Global,
                label: || I18nOps::get().tab.nav_next.clone(),
                is_available: |state| state.document.open_documents.len() > 1,
                default_shortcuts: &["primary+Right", "primary+Shift+]"],
            },
            CommandInventoryItem {
                id: "view.tab_prev",
                action: AppAction::SelectPrevTab,
                group: CommandGroup::Behavior,
                context: ShortcutContext::Global,
                label: || I18nOps::get().tab.nav_prev.clone(),
                is_available: |state| state.document.open_documents.len() > 1,
                default_shortcuts: &["primary+Left", "primary+Shift+["],
            },
            CommandInventoryItem {
                id: "view.toggle_slideshow",
                action: AppAction::ToggleSlideshow,
                group: CommandGroup::Behavior,
                context: ShortcutContext::Global,
                label: || I18nOps::get().preview.toggle_slideshow.clone(),
                is_available: |_| true,
                default_shortcuts: &["primary+alt+Enter", "F5"],
            },
        ]
    }
}
