use super::ShortcutsTabOps;
use crate::app_state::AppState;
use crate::state::command_inventory::CommandInventory;
use crate::state::command_inventory::CommandInventoryItem;
use crate::state::command_inventory::types::CommandGroup;
use eframe::egui;
use std::collections::HashMap;

impl ShortcutsTabOps {
    /* WHY: Renders one accordion group of shortcuts, filtered by the search query */
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_group(
        ui: &mut egui::Ui,
        _state: &mut AppState,
        group: CommandGroup,
        search_lower: &str,
        recording_id: &str,
        recording_id_salt: egui::Id,
        os_bindings: &HashMap<String, String>,
        accordion_line: bool,
        force_open: Option<bool>,
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
                    egui_extras::TableBuilder::new(ui)
                        .resizable(false)
                        .vscroll(false)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(egui_extras::Column::exact(
                            crate::settings::tabs::shortcuts::row::ROW_LABEL_WIDTH,
                        ))
                        .column(
                            egui_extras::Column::remainder().at_least(
                                crate::settings::tabs::shortcuts::row::ROW_SHORTCUT_WIDTH,
                            ),
                        )
                        .column(egui_extras::Column::exact(
                            crate::settings::tabs::shortcuts::row::ROW_ACTIONS_WIDTH,
                        ))
                        .body(|body| {
                            body.rows(
                                crate::settings::tabs::shortcuts::row::ROW_H,
                                cmds_in_group.len(),
                                |mut row| {
                                    let cmd = cmds_in_group[row.index()];
                                    Self::render_command_row(
                                        &mut row,
                                        _state,
                                        cmd,
                                        recording_id,
                                        recording_id_salt,
                                        os_bindings,
                                    );
                                },
                            );
                        });
                },
            )
            .default_open(true)
            .force_open(force_open)
            .show_vertical_line(accordion_line)
            .show(ui);
        });

        ui.add_space(crate::settings::SECTION_SPACING);
    }
}
