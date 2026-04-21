use super::ShortcutsTabOps;
use crate::app_state::AppState;
use crate::i18n::I18nOps;
use crate::icon::{Icon, IconSize};
use crate::state::command_inventory::CommandInventoryItem;
use eframe::egui;
use std::collections::HashMap;

const RECORDING_ICON_SIZE: f32 = 10.0;
const RECORDING_ICON: &str = "⏺";

/// Row column widths
/// Layout: [Label(fixed) | Shortcut(fixed) | Actions(fill, right-aligned to panel edge)]
pub(super) const ROW_LABEL_WIDTH: f32 = 220.0;
pub(super) const ROW_SHORTCUT_WIDTH: f32 = 180.0;
pub(super) const ROW_H: f32 = 28.0;

impl ShortcutsTabOps {
    /* WHY: Renders one row natively via TableRow, keeping exact columns.
    - Col 1 (Label):    Fixed ROW_LABEL_WIDTH
    - Col 2 (Shortcut): Fixed ROW_SHORTCUT_WIDTH
    - Col 3 (Buttons):  Fills remaining space, right-aligned */
    pub(super) fn render_command_row(
        row: &mut egui_extras::TableRow<'_, '_>,
        state: &mut AppState,
        cmd: &CommandInventoryItem,
        recording_id: &str,
        recording_id_salt: egui::Id,
        os_bindings: &HashMap<String, String>,
    ) {
        let is_recording = recording_id == cmd.id;
        let has_custom = os_bindings.contains_key(cmd.id);

        /* Table Column 1: Command label */
        row.col(|ui| {
            ui.add(egui::Label::new((cmd.label)()).truncate());
        });

        /* Table Column 2: Shortcut keys */
        row.col(|ui| {
            if is_recording {
                /* WHY: Show recording hint while modal is open */
                let accent = ui.visuals().selection.bg_fill;
                ui.label(
                    egui::RichText::new(RECORDING_ICON)
                        .color(accent)
                        .size(RECORDING_ICON_SIZE),
                );
            } else {
                let shortcut_str = os_bindings
                    .get(cmd.id)
                    .cloned()
                    .unwrap_or_else(|| cmd.default_shortcuts.join(", "));

                let i18n = I18nOps::get();
                if shortcut_str.is_empty() {
                    ui.label(egui::RichText::new(&i18n.settings.shortcuts.none).weak());
                } else {
                    crate::widgets::ShortcutWidget::new(&shortcut_str).ui(ui);
                }
            }
        });

        /* Table Column 3: Action buttons */
        row.col(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;

                let edit_btn =
                    ui.add(Icon::Edit.selected_button(ui, IconSize::Small, is_recording));
                if edit_btn.clicked() {
                    if is_recording {
                        ui.memory_mut(|mem| mem.data.remove::<String>(recording_id_salt));
                    } else {
                        ui.memory_mut(|mem| {
                            mem.data.insert_temp(recording_id_salt, cmd.id.to_string())
                        });
                    }
                }

                if has_custom {
                    let remove_btn = ui.add(Icon::Remove.button(ui, IconSize::Small));
                    if remove_btn.clicked() {
                        let s = state.config.settings.settings_mut();
                        let map = match std::env::consts::OS {
                            "macos" => &mut s.shortcuts.macos,
                            "windows" => &mut s.shortcuts.windows,
                            _ => &mut s.shortcuts.linux,
                        };
                        map.remove(cmd.id);
                        state.config.try_save_settings();
                    }
                }
            });
        });
    }
}
