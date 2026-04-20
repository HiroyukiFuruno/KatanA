use super::ShortcutsTabOps;
use crate::app_state::AppState;
use crate::i18n::I18nOps;
use crate::state::command_inventory::types::CommandGroup;
use crate::state::command_inventory::{CommandInventory, CommandInventoryItem};
use eframe::egui;
use std::collections::HashMap;

const GRID_SPACING_X: f32 = 16.0;
const GRID_SPACING_Y: f32 = 8.0;
const SEARCH_FILTER_ID: &str = "shortcut_search_filter";
const SHORTCUT_GRID_COLUMNS: usize = 3;

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

        /* WHY: If recording is active, show the capture modal and filter out key events */
        if !recording_id.is_empty() {
            Self::show_capture_modal(ui, state, &recording_id, recording_id_salt, &os_bindings);
        }

        Self::render_conflict_warning(ui);

        /* WHY: Search bar to filter shortcuts by command name */
        let mut search_query = ui.memory(|mem| {
            mem.data
                .get_temp::<String>(egui::Id::new(SEARCH_FILTER_ID))
                .unwrap_or_default()
        });

        let i18n = I18nOps::get();
        let search_response = ui.add(
            egui::TextEdit::singleline(&mut search_query)
                .hint_text(&i18n.settings.shortcuts.search_placeholder)
                .desired_width(f32::INFINITY)
                .id(egui::Id::new(SEARCH_FILTER_ID)),
        );

        if search_response.changed() {
            let q = search_query.clone();
            ui.memory_mut(|mem| {
                mem.data.insert_temp(egui::Id::new(SEARCH_FILTER_ID), q);
            });
        }

        ui.add_space(crate::settings::SECTION_SPACING);

        let accordion_line = state
            .config
            .settings
            .settings()
            .layout
            .accordion_vertical_line;
        let search_lower = search_query.to_lowercase();

        for group in groups {
            Self::render_group(
                ui,
                state,
                group,
                &search_lower,
                &recording_id,
                recording_id_salt,
                &os_bindings,
                accordion_line,
            );
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

    /* WHY: Renders one accordion group of shortcuts, filtered by the search query */
    #[allow(clippy::too_many_arguments)]
    fn render_group(
        ui: &mut egui::Ui,
        _state: &mut AppState,
        group: CommandGroup,
        search_lower: &str,
        recording_id: &str,
        recording_id_salt: egui::Id,
        os_bindings: &HashMap<String, String>,
        accordion_line: bool,
    ) {
        let all_cmds = CommandInventory::all();
        let cmds_in_group: Vec<&CommandInventoryItem> = all_cmds
            .iter()
            .filter(|c| {
                c.group == group
                    && (search_lower.is_empty()
                        || (c.label)().to_lowercase().contains(search_lower))
            })
            .collect();

        if cmds_in_group.is_empty() {
            return;
        }

        ui.push_id(group, |ui| {
            crate::widgets::Accordion::new(
                format!("shortcuts_accordion_{:?}", group),
                egui::RichText::new(group.localized_name())
                    .strong()
                    .size(crate::settings::SECTION_HEADER_SIZE),
                |ui| {
                    egui::Grid::new(format!("shortcuts_grid_{:?}", group))
                        .num_columns(SHORTCUT_GRID_COLUMNS)
                        .spacing([GRID_SPACING_X, GRID_SPACING_Y])
                        .show(ui, |ui| {
                            for cmd in &cmds_in_group {
                                Self::render_command_row(
                                    ui,
                                    cmd,
                                    recording_id,
                                    recording_id_salt,
                                    os_bindings,
                                );
                            }
                        });
                },
            )
            .default_open(true)
            .show_vertical_line(accordion_line)
            .show(ui);
        });

        ui.add_space(crate::settings::SECTION_SPACING);
    }
}
