/* WHY: Refactored modal orchestration to maintain architectural line limits and separated concerns. */

use crate::shell::KatanaApp;
use eframe::egui;

mod file_modals;
mod search_modals;
mod system_modals;
mod workspace_modals;

impl KatanaApp {
    pub fn show_modals(&mut self, ctx: &egui::Context) {
        self.show_workspace_visibility_modals(ctx);
        self.show_search_and_palette_modals(ctx);
        self.show_file_operations_modals(ctx);
        self.show_system_modals(ctx);
    }
}
