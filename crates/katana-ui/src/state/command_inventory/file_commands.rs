use super::{CommandGroup, CommandInventoryItem};
use crate::app_state::AppAction;
use crate::i18n::I18nOps;
use crate::state::shortcut_context::ShortcutContext;

pub struct FileCommands;

impl FileCommands {
    pub fn get() -> Vec<CommandInventoryItem> {
        vec![
            /* WHY: File Group */
            CommandInventoryItem {
                id: "file.open_workspace",
                action: AppAction::PickOpenWorkspace,
                group: CommandGroup::File,
                context: ShortcutContext::Global,
                label: || I18nOps::get().menu.open_workspace.clone(),
                is_available: |_| true,
                default_shortcuts: &["primary+O"],
            },
            CommandInventoryItem {
                id: "file.close_workspace",
                action: AppAction::CloseWorkspace,
                group: CommandGroup::File,
                context: ShortcutContext::Global,
                label: || I18nOps::get().menu.close_workspace.clone(),
                is_available: |state| state.workspace.data.is_some(),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "file.save",
                action: AppAction::SaveDocument,
                group: CommandGroup::File,
                context: ShortcutContext::Global,
                label: || I18nOps::get().menu.save.clone(),
                is_available: |state| state.document.active_doc_idx.is_some(),
                default_shortcuts: &["primary+S"],
            },
            CommandInventoryItem {
                id: "file.close_document",
                action: AppAction::CloseActiveDocument,
                group: CommandGroup::File,
                context: ShortcutContext::Global,
                label: || I18nOps::get().menu.close_document.clone(),
                is_available: |state| state.document.active_doc_idx.is_some(),
                default_shortcuts: &["primary+W"],
            },
            CommandInventoryItem {
                id: "file.restore_closed",
                action: AppAction::RestoreClosedDocument,
                group: CommandGroup::File,
                context: ShortcutContext::Global,
                label: || I18nOps::get().menu.restore_closed.clone(),
                is_available: |state| !state.document.recently_closed_tabs.is_empty(),
                default_shortcuts: &["primary+Shift+T"],
            },
        ]
    }
}
