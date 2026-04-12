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
    pub default_shortcuts: &'static [&'static str],
}
