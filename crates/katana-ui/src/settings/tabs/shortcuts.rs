use super::ShortcutsTabOps;
use super::shortcuts_helpers::ShortcutsHelpersOps;
use crate::app_state::AppState;
use crate::i18n::I18nOps;
use crate::state::command_inventory::types::CommandGroup;
use crate::state::command_inventory::{CommandInventory, CommandInventoryItem};
use eframe::egui;
use std::collections::HashMap;

const COLUMNS: usize = 3;
const GRID_SPACING_X: f32 = 16.0;
const GRID_SPACING_Y: f32 = 8.0;

impl ShortcutsTabOps {
    pub(crate) fn render_shortcuts_tab(ui: &mut egui::Ui, state: &mut AppState) {
        let os_bindings = state
            .config
            .settings
            .settings()
            .shortcuts
            .current_os_bindings()
            .clone();

        let groups = [
            CommandGroup::App,
            CommandGroup::File,
            CommandGroup::Edit,
            CommandGroup::View,
            CommandGroup::Help,
        ];

        let recording_id_salt = egui::Id::new("recording_shortcut_id");
        let recording_id = ui.memory(|mem| {
            mem.data
                .get_temp::<String>(recording_id_salt)
                .unwrap_or_default()
        });

        Self::render_conflict_warning(ui);

        for group in groups {
            ui.push_id(group, |ui| {
                crate::widgets::Accordion::new(
                    format!("shortcuts_accordion_{:?}", group),
                    egui::RichText::new(group.localized_name())
                        .strong()
                        .size(crate::settings::SECTION_HEADER_SIZE),
                    |ui| {
                        egui::Grid::new(format!("shortcuts_grid_{:?}", group))
                            .num_columns(COLUMNS)
                            .spacing([GRID_SPACING_X, GRID_SPACING_Y])
                            .show(ui, |ui| {
                                for cmd in
                                    CommandInventory::all().iter().filter(|c| c.group == group)
                                {
                                    Self::render_command_row(
                                        ui,
                                        state,
                                        cmd,
                                        &recording_id,
                                        recording_id_salt,
                                        &os_bindings,
                                    );
                                }
                            });
                    },
                )
                .default_open(true)
                .show(ui);
            });

            ui.add_space(crate::settings::SECTION_SPACING);
        }

        let i18n = I18nOps::get();
        if ui
            .button(&i18n.settings.shortcuts.restore_defaults)
            .clicked()
        {
            let s = state.config.settings.settings_mut();
            s.shortcuts.macos.clear();
            s.shortcuts.linux.clear();
            s.shortcuts.windows.clear();
            state.config.try_save_settings();
        }
    }

    /* WHY: Renders a warning message if a shortcut conflict was detected */
    fn render_conflict_warning(ui: &mut egui::Ui) {
        if let Some(conflict_msg) = ui.memory(|mem| {
            mem.data
                .get_temp::<String>(egui::Id::new("shortcut_conflict"))
        }) {
            let color = ui.visuals().error_fg_color;
            ui.label(egui::RichText::new(conflict_msg).color(color));

            let i18n = I18nOps::get();
            if ui.button(&i18n.common.close).clicked() {
                ui.memory_mut(|mem| {
                    mem.data
                        .remove::<String>(egui::Id::new("shortcut_conflict"))
                });
            }
            ui.add_space(crate::settings::SECTION_SPACING);
        }
    }

    /* WHY: Renders the table row for a specific command */
    fn render_command_row(
        ui: &mut egui::Ui,
        state: &mut AppState,
        cmd: &CommandInventoryItem,
        recording_id: &str,
        recording_id_salt: egui::Id,
        os_bindings: &HashMap<String, String>,
    ) {
        ui.label((cmd.label)());

        let i18n = I18nOps::get();

        let shortcut_str = if let Some(custom) = os_bindings.get(cmd.id) {
            custom.clone()
        } else {
            cmd.default_shortcuts.join(", ")
        };

        if shortcut_str.is_empty() {
            ui.label(&i18n.settings.shortcuts.none);
        } else {
            ui.label(&shortcut_str);
        }

        let mut edit_label = i18n.settings.shortcuts.edit.as_str();
        if recording_id == cmd.id {
            edit_label = i18n.settings.shortcuts.press_keys.as_str();
            ShortcutsHelpersOps::handle_shortcut_recording(
                ui,
                state,
                cmd,
                recording_id_salt,
                os_bindings,
            );
        }

        if ui.button(edit_label).clicked() {
            ui.memory_mut(|mem| mem.data.insert_temp(recording_id_salt, cmd.id.to_string()));
        }

        ui.end_row();
    }
}
