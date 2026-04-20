use super::ShortcutsTabOps;
use crate::i18n::I18nOps;
use crate::icon::{Icon, IconSize};
use crate::state::command_inventory::CommandInventoryItem;
use eframe::egui;
use std::collections::HashMap;

impl ShortcutsTabOps {
    /* WHY: Renders one row: [command label] [shortcut keys with OS symbols] [edit icon button] */
    pub(super) fn render_command_row(
        ui: &mut egui::Ui,
        cmd: &CommandInventoryItem,
        recording_id: &str,
        recording_id_salt: egui::Id,
        os_bindings: &HashMap<String, String>,
    ) {
        let is_recording = recording_id == cmd.id;

        ui.label((cmd.label)());

        /* WHY: Display the shortcut with OS-native key symbols from OsCommandOps */
        let shortcut_str = os_bindings
            .get(cmd.id)
            .cloned()
            .unwrap_or_else(|| cmd.default_shortcuts.join(", "));

        let i18n = I18nOps::get();
        if shortcut_str.is_empty() {
            ui.label(&i18n.settings.shortcuts.none);
        } else if is_recording {
            ui.label(egui::RichText::new(&i18n.settings.shortcuts.capture_prompt).weak());
        } else {
            let display = Self::format_shortcut_with_symbols(&shortcut_str);
            ui.label(egui::RichText::new(display).monospace());
        }

        /* WHY: Replace the old text "Edit" button with an icon button for cleaner UI */
        let edit_btn = ui.add(
            Icon::Edit
                .button(ui, IconSize::Small)
                .selected(is_recording),
        );

        if edit_btn.clicked() {
            if is_recording {
                ui.memory_mut(|mem| mem.data.remove::<String>(recording_id_salt));
            } else {
                ui.memory_mut(|mem| mem.data.insert_temp(recording_id_salt, cmd.id.to_string()));
            }
        }

        ui.end_row();
    }

    /* WHY: Converts internal shortcut token strings (primary, shift, alt) to OS-native symbols.
    Uses OsCommandOps to avoid hardcoding platform-specific characters directly in source. */
    pub(super) fn format_shortcut_with_symbols(shortcut: &str) -> String {
        let primary = crate::os_command::OsCommandOps::get("modifier_primary");
        let shift = crate::os_command::OsCommandOps::get("modifier_shift");
        let alt = crate::os_command::OsCommandOps::get("modifier_alt");

        shortcut
            .replace("primary", &primary)
            .replace("mac_cmd", &primary)
            .replace("shift", &shift)
            .replace("alt", &alt)
    }
}
