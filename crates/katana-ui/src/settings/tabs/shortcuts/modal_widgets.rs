use super::helpers::ShortcutsHelpersOps;
use crate::app_state::AppState;
use crate::state::command_inventory::CommandInventoryItem;
use eframe::egui;
use std::collections::HashMap;

const MODAL_SPACING_MID: f32 = 10.0;
const MODAL_FONT_SIZE: f32 = 16.0;
const CONFLICT_FONT_SCALE: f32 = 0.9;

pub struct ModalWidgets;

impl ModalWidgets {
    /* WHY: Save = apply, Close = ESC/cancel (discard in-progress recording). */
    pub(super) fn render_action_buttons(
        ui: &mut egui::Ui,
        state: &mut AppState,
        cmd: &CommandInventoryItem,
        temp_shortcut: &Option<String>,
        recording_id_salt: egui::Id,
        os_bindings: &HashMap<String, String>,
        i18n: &crate::i18n::I18nMessages,
    ) {
        crate::widgets::AlignCenter::new()
            .shrink_to_fit(true)
            .content(|ui| {
                if ui.button(&i18n.common.close).clicked() {
                    ui.memory_mut(|mem| {
                        mem.data.remove::<String>(recording_id_salt);
                        mem.data.remove::<String>(
                            egui::Id::new("shortcut_temp").with(recording_id_salt),
                        );
                        mem.data
                            .remove::<String>(egui::Id::new("shortcut_conflict"));
                    });
                }

                let can_save = temp_shortcut.is_some();
                ui.add_enabled_ui(can_save, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(&i18n.common.save).clicked() {
                            let Some(new_shortcut) = temp_shortcut else {
                                return;
                            };
                            ShortcutsHelpersOps::check_and_save_shortcut(
                                ui,
                                state,
                                cmd,
                                new_shortcut,
                                recording_id_salt,
                                os_bindings,
                            );
                        }
                    });
                });
            })
            .show(ui);
    }

    /* WHY: Shows a warning label if a conflict key was stored in egui memory. */
    pub(super) fn render_conflict_warning(ui: &mut egui::Ui) {
        if let Some(msg) = ui.memory(|mem| {
            mem.data
                .get_temp::<String>(egui::Id::new("shortcut_conflict"))
        }) {
            crate::widgets::AlignCenter::new()
                .shrink_to_fit(true)
                .content(|ui| {
                    ui.label(
                        egui::RichText::new(msg)
                            .color(ui.visuals().error_fg_color)
                            .size(MODAL_FONT_SIZE * CONFLICT_FONT_SCALE),
                    );
                })
                .show(ui);
            ui.add_space(MODAL_SPACING_MID);
        }
    }
}
