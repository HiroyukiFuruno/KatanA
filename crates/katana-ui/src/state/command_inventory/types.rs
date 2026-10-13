use crate::app_state::{AppAction, AppState};
use crate::i18n::I18nOps;
use crate::state::shortcut_context::ShortcutContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandGroup {
    App,
    Edit,
    File,
    View,
    Behavior,
    Help,
}

impl CommandGroup {
    pub fn localized_name(self) -> String {
        let i18n = I18nOps::get();
        match self {
            Self::App => "KatanA".to_string(), // WHY: Main app menu equivalent
            /* WHY: "Edit" group is used for Markdown authoring commands. */
            Self::Edit => i18n.settings.shortcuts.edit.clone(),
            Self::File => i18n.menu.file.clone(),
            Self::View => i18n.menu.view.clone(),
            Self::Behavior => i18n.settings.behavior.section_title.clone(),
            Self::Help => i18n.menu.help.clone(),
        }
    }
}

pub struct CommandInventoryItem {
    pub id: &'static str,
    pub action: AppAction,
    pub group: CommandGroup,
    /// WHY: Defines the UI context in which this command's shortcut is active.
    /// Commands with Global context fire in any non-Recording, non-Modal context.
    /// Commands with Editor context fire only when the text editor has focus.
    pub context: ShortcutContext,
    pub label: fn() -> String,
    pub is_available: fn(&AppState) -> bool,
    pub default_shortcuts: &'static [&'static str],
}
