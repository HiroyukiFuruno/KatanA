use super::{CommandGroup, CommandInventoryItem};
use crate::app_state::AppAction;
use crate::i18n::I18nOps;

pub struct AppCommands;

impl AppCommands {
    pub fn get() -> Vec<CommandInventoryItem> {
        vec![/* WHY: App Group */ CommandInventoryItem {
            id: "app.settings",
            action: AppAction::ToggleSettings,
            group: CommandGroup::App,
            label: || I18nOps::get().menu.settings.clone(),
            is_available: |_| true,
            default_shortcuts: &["primary+,"],
        }]
    }
}
