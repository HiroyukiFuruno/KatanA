pub mod types;
#[cfg(debug_assertions)]
use crate::state::shortcut_context::ShortcutContext;
pub use types::*;
pub mod app_commands;
pub mod edit_commands;
pub mod file_commands;
pub mod help_commands;
pub mod view_commands;

pub struct CommandInventory;

impl CommandInventory {
    pub fn all() -> Vec<CommandInventoryItem> {
        let mut commands = Vec::new();
        commands.extend(app_commands::AppCommands::get());
        commands.extend(file_commands::FileCommands::get());
        commands.extend(view_commands::ViewCommands::get());
        commands.extend(help_commands::HelpCommands::get());
        commands.extend(edit_commands::EditCommands::get());

        #[cfg(debug_assertions)]
        Self::validate_shortcuts(&commands);

        commands
    }

    #[cfg(debug_assertions)]
    fn validate_shortcuts(commands: &[CommandInventoryItem]) {
        use std::collections::HashMap;
        let mut shortcuts: HashMap<(ShortcutContext, String), &str> = HashMap::new();
        for cmd in commands {
            Self::validate_command_shortcuts(cmd, &mut shortcuts);
        }
    }

    #[cfg(debug_assertions)]
    fn validate_command_shortcuts<'a>(
        cmd: &'a CommandInventoryItem,
        shortcuts: &mut std::collections::HashMap<(ShortcutContext, String), &'a str>,
    ) {
        for shortcut in cmd.default_shortcuts {
            let key = (cmd.context, shortcut.to_string());
            if let Some(existing_id) = shortcuts.get(&key) {
                panic!(
                    "Duplicate default shortcut '{}' found for commands '{}' and '{}' in context {:?}",
                    shortcut, cmd.id, existing_id, cmd.context
                );
            }
            shortcuts.insert(key, cmd.id);

            /* WHY: Cross-check: Global shortcuts also occupy all other contexts except Modal/Recording */
            if cmd.context == ShortcutContext::Global {
                Self::validate_global_cross_context(cmd, shortcut, shortcuts);
            } else if cmd.context != ShortcutContext::Modal
                && cmd.context != ShortcutContext::Recording
            {
                Self::validate_specific_cross_context(cmd, shortcut, shortcuts);
            }
        }
    }

    #[cfg(debug_assertions)]
    fn validate_global_cross_context<'a>(
        cmd: &'a CommandInventoryItem,
        shortcut: &str,
        shortcuts: &std::collections::HashMap<(ShortcutContext, String), &'a str>,
    ) {
        for other_ctx in [
            ShortcutContext::Editor,
            ShortcutContext::Preview,
            ShortcutContext::Explorer,
        ] {
            let other_key = (other_ctx, shortcut.to_string());
            if let Some(existing_id) = shortcuts.get(&other_key) {
                panic!(
                    "Global shortcut '{}' (command '{}') conflicts with command '{}' in context {:?}",
                    shortcut, cmd.id, existing_id, other_ctx
                );
            }
        }
    }

    #[cfg(debug_assertions)]
    fn validate_specific_cross_context<'a>(
        cmd: &'a CommandInventoryItem,
        shortcut: &str,
        shortcuts: &std::collections::HashMap<(ShortcutContext, String), &'a str>,
    ) {
        /* WHY: Specific context shortcut must not conflict with Global */
        let global_key = (ShortcutContext::Global, shortcut.to_string());
        if let Some(existing_id) = shortcuts.get(&global_key) {
            panic!(
                "Shortcut '{}' (command '{}') in context {:?} conflicts with global command '{}'",
                shortcut, cmd.id, cmd.context, existing_id
            );
        }
    }
}
