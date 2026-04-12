pub mod types;
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
        commands
    }
}
