use super::ShortcutsTabOps;
use super::shortcuts_helpers::ShortcutsHelpersOps;
use crate::app_state::AppState;
use crate::i18n::I18nOps;
use crate::state::command_inventory::CommandInventory;
use eframe::egui;
use std::collections::HashMap;

pub(super) const CAPTURE_MODAL_ID: &str = "shortcut_capture_modal";
pub(super) const CAPTURE_MODAL_WIDTH: f32 = 340.0;
const CAPTURE_MODAL_BODY_SPACING_TOP: f32 = 12.0;
const CAPTURE_MODAL_BODY_SPACING_MID: f32 = 8.0;
const CAPTURE_MODAL_FONT_SIZE: f32 = 16.0;

impl ShortcutsTabOps {
    /* WHY: Shows the shortcut capture modal when recording is active.
    Keyboard events other than Esc are captured here, preventing
    accidental app-level shortcut triggers during recording. */
    pub(super) fn show_capture_modal(
        ui: &mut egui::Ui,
        state: &mut AppState,
        recording_id: &str,
        recording_id_salt: egui::Id,
        os_bindings: &HashMap<String, String>,
    ) {
        let i18n = I18nOps::get();

        /* WHY: Filter out all key events except Esc while the modal is open */
        let (should_cancel, pressed_shortcut) = Self::capture_key_from_events(ui);

        if should_cancel {
            ui.memory_mut(|mem| mem.data.remove::<String>(recording_id_salt));
            return;
        }

        let all_cmds = CommandInventory::all();
        let Some(cmd) = all_cmds.iter().find(|c| c.id == recording_id) else {
            /* WHY: If the command is no longer found, clear the recording state */
            ui.memory_mut(|mem| mem.data.remove::<String>(recording_id_salt));
            return;
        };

        if let Some((new_shortcut, _)) = pressed_shortcut {
            ShortcutsHelpersOps::check_and_save_shortcut(
                ui,
                state,
                cmd,
                &new_shortcut,
                recording_id_salt,
                os_bindings,
            );
            return;
        }

        /* WHY: Show a modal dialog while waiting for key input */
        let modal_title = (cmd.label)();
        crate::widgets::Modal::new(CAPTURE_MODAL_ID, &modal_title)
            .width(CAPTURE_MODAL_WIDTH)
            .show_body_only(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(CAPTURE_MODAL_BODY_SPACING_TOP);
                    ui.label(
                        egui::RichText::new(&i18n.settings.shortcuts.capture_prompt)
                            .size(CAPTURE_MODAL_FONT_SIZE),
                    );
                    ui.add_space(CAPTURE_MODAL_BODY_SPACING_MID);
                    ui.label(
                        egui::RichText::new(crate::i18n::I18nOps::tf(
                            "{confirm} / {cancel}",
                            &[
                                ("confirm", &i18n.settings.shortcuts.confirm_key),
                                ("cancel", &i18n.settings.shortcuts.cancel_key),
                            ],
                        ))
                        .weak(),
                    );
                    ui.add_space(CAPTURE_MODAL_BODY_SPACING_MID);
                    if ui.button(&i18n.common.close).clicked() {
                        ui.memory_mut(|mem| mem.data.remove::<String>(recording_id_salt));
                    }
                });
            });
    }

    /* WHY: Renders a warning message if a shortcut conflict was detected */
    pub(super) fn render_conflict_warning(ui: &mut egui::Ui) {
        let conflict_msg = ui.memory(|mem| {
            mem.data
                .get_temp::<String>(egui::Id::new("shortcut_conflict"))
        });

        let Some(msg) = conflict_msg else {
            return;
        };

        let color = ui.visuals().error_fg_color;
        ui.label(egui::RichText::new(msg).color(color));

        let i18n = I18nOps::get();
        if ui.button(&i18n.common.close).clicked() {
            ui.memory_mut(|mem| {
                mem.data
                    .remove::<String>(egui::Id::new("shortcut_conflict"))
            });
        }
        ui.add_space(crate::settings::SECTION_SPACING);
    }

    /* WHY: Extracts the key press combination from egui events, consuming key events to
    prevent app-level shortcut reactions while the capture modal is open. */
    fn capture_key_from_events(ui: &mut egui::Ui) -> (bool, Option<(String, egui::Modifiers)>) {
        ui.input_mut(|input| {
            if input.key_pressed(egui::Key::Escape) {
                return (true, None);
            }

            let result_shortcut = Self::find_key_combo_in_events(&input.events);

            /* WHY: Consume all key events to prevent app-level reactions */
            input
                .events
                .retain(|e| !matches!(e, egui::Event::Key { .. }));

            (false, result_shortcut)
        })
    }

    /* WHY: Scans events for a non-modifier key press and builds the combination string. */
    fn find_key_combo_in_events(events: &[egui::Event]) -> Option<(String, egui::Modifiers)> {
        for event in events {
            let egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } = event
            else {
                continue;
            };
            let key_str = ShortcutsHelpersOps::key_to_string(*key);
            if key_str.is_empty() {
                continue;
            }
            let mut parts = Vec::new();
            ShortcutsHelpersOps::add_modifier_if(modifiers.command, "primary", &mut parts);
            ShortcutsHelpersOps::add_modifier_if(modifiers.shift, "shift", &mut parts);
            ShortcutsHelpersOps::add_modifier_if(modifiers.alt, "alt", &mut parts);
            ShortcutsHelpersOps::add_modifier_if(modifiers.mac_cmd, "mac_cmd", &mut parts);
            parts.push(key_str);
            return Some((parts.join("+"), *modifiers));
        }
        None
    }
}
