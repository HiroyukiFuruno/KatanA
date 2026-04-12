use super::{CommandGroup, CommandInventoryItem};
use crate::app_state::AppAction;
use crate::i18n::I18nOps;
use crate::state::shortcut_context::ShortcutContext;

pub struct AppCommands;

impl AppCommands {
    pub fn get() -> Vec<CommandInventoryItem> {
        /* WHY: App Group */
        vec![CommandInventoryItem {
            id: "app.settings",
            action: AppAction::ToggleSettings,
            group: CommandGroup::App,
            context: ShortcutContext::Global,
            label: || I18nOps::get().menu.settings.clone(),
            is_available: |_| true,
            default_shortcuts: &["primary+,"],
        }]
    }
}
