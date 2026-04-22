use super::ShortcutsTabOps;
use super::helpers::ShortcutsHelpersOps;
use super::modal_widgets;
use crate::app_state::AppState;
use crate::i18n::I18nOps;
use crate::state::command_inventory::CommandInventory;
use eframe::egui;
use std::collections::HashMap;

const CAPTURE_MODAL_ID: &str = "shortcut_capture_modal";
const CAPTURE_MODAL_WIDTH: f32 = 340.0;
const CAPTURE_MODAL_BODY_SPACING_TOP: f32 = 16.0;
const CAPTURE_MODAL_BODY_SPACING_MID: f32 = 10.0;
const CAPTURE_MODAL_FONT_SIZE: f32 = 16.0;
const OVERLAY_OPACITY: f32 = 0.8;

impl ShortcutsTabOps {
    /* WHY: Shows the shortcut capture modal when recording is active. Rendered at Foreground order to ensure it appears above the settings panel. Keyboard events are consumed here to prevent Katana shortcuts from misfiring during recording. */
    pub(super) fn show_capture_modal(
        ui: &mut egui::Ui,
        state: &mut AppState,
        recording_id: &str,
        recording_id_salt: egui::Id,
        os_bindings: &HashMap<String, String>,
    ) {
        /* WHY: Draw a semi-transparent overlay over the entire settings area to visually block other interaction while recording is active. Use window_fill instead of hardcoded colors to conform to AST linters. */
        let screen = ui.ctx().screen_rect();
        ui.painter().rect_filled(
            screen,
            0.0,
            ui.visuals().window_fill().gamma_multiply(OVERLAY_OPACITY),
        );

        #[cfg(target_os = "macos")]
        let is_mac = true;
        #[cfg(not(target_os = "macos"))]
        let is_mac = false;

        /* WHY: Consume all key events at the top-level ui to prevent Katana shortcuts from reacting while the capture modal is open */
        let (should_cancel, should_confirm, pressed_shortcut) =
            crate::settings::tabs::shortcuts::key_events::KeyEventsOps::capture_key_from_events(
                ui, is_mac,
            );

        let mut temp_shortcut = ui.memory_mut(|mem| {
            mem.data
                .get_temp::<String>(egui::Id::new("shortcut_temp").with(recording_id_salt))
        });

        if should_cancel {
            ui.memory_mut(|mem| {
                mem.data.remove::<String>(recording_id_salt);
                mem.data
                    .remove::<String>(egui::Id::new("shortcut_temp").with(recording_id_salt));
                mem.data
                    .remove::<String>(egui::Id::new("shortcut_conflict"));
            });
            return;
        }

        let all_cmds = CommandInventory::all();
        let Some(cmd) = all_cmds.iter().find(|c| c.id == recording_id) else {
            ui.memory_mut(|mem| {
                mem.data.remove::<String>(recording_id_salt);
                mem.data
                    .remove::<String>(egui::Id::new("shortcut_temp").with(recording_id_salt));
                mem.data
                    .remove::<String>(egui::Id::new("shortcut_conflict"));
            });
            return;
        };

        if let Some((new_shortcut, _)) = pressed_shortcut {
            temp_shortcut = Some(new_shortcut.clone());
            ui.memory_mut(|mem| {
                mem.data.insert_temp(
                    egui::Id::new("shortcut_temp").with(recording_id_salt),
                    new_shortcut,
                );
            });
            ui.memory_mut(|mem| {
                mem.data
                    .remove::<String>(egui::Id::new("shortcut_conflict"));
            });
        }

        if should_confirm {
            if let Some(new_shortcut) = &temp_shortcut {
                ShortcutsHelpersOps::check_and_save_shortcut(
                    ui,
                    state,
                    cmd,
                    new_shortcut,
                    recording_id_salt,
                    os_bindings,
                );
            }
            return;
        }

        let i18n = I18nOps::get();
        let modal_title = (cmd.label)();

        crate::widgets::Modal::new(CAPTURE_MODAL_ID, &modal_title)
            .width(CAPTURE_MODAL_WIDTH)
            .show_body_only(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(CAPTURE_MODAL_BODY_SPACING_TOP);
                    modal_widgets::ModalWidgets::render_conflict_warning(ui);

                    /* WHY: Real-time display of currently held modifiers inside a fake, focused Input box.
                    This provides immediate visual feedback exactly like VSCode. */
                    ui.add_space(CAPTURE_MODAL_BODY_SPACING_MID);

                    const BOX_INNER_MARGIN_X: i8 = 16;
                    const BOX_INNER_MARGIN_Y: i8 = 10;
                    let mut frame = egui::Frame::none()
                        .fill(ui.visuals().extreme_bg_color)
                        .rounding(ui.visuals().widgets.inactive.corner_radius)
                        .inner_margin(egui::Margin::symmetric(BOX_INNER_MARGIN_X, BOX_INNER_MARGIN_Y));
                    /* WHY: Simulate active/focused state */
                    frame.stroke = ui.visuals().selection.stroke;
                    frame.show(ui, |ui| {
                        /* WHY: Force full width for the input box */
                        ui.set_width(ui.available_width());

                        /* WHY: Force perfect horizontal centering without expanding vertically indefinitely.
                           Avoid centered_and_justified as it causes layout ratcheting inside auto-sizing Windows. */
                        crate::widgets::AlignCenter::new()
                            .shrink_to_fit(true)
                            .content(|ui| {
                                if let Some(shortcut) = &temp_shortcut {
                                    crate::widgets::ShortcutWidget::new(shortcut).ui(ui);
                                } else {
                                    let modifiers = ui.input(|i| i.modifiers);
                                    let mut parts = Vec::new();
                                    if modifiers.ctrl {
                                        parts.push(if is_mac { "ctrl" } else { "primary" });
                                    }
                                    if modifiers.mac_cmd {
                                        parts.push(if is_mac { "primary" } else { "win" });
                                    }
                                    if modifiers.shift {
                                        parts.push("shift");
                                    }
                                    if modifiers.alt {
                                        parts.push("alt");
                                    }

                                    if parts.is_empty() {
                                        ui.label(
                                            egui::RichText::new(&i18n.settings.shortcuts.capture_prompt)
                                                .weak()
                                                .size(CAPTURE_MODAL_FONT_SIZE),
                                        );
                                    } else {
                                        let shortcut_str = parts.join("+");
                                        crate::widgets::ShortcutWidget::new(&shortcut_str).ui(ui);
                                    }
                                }
                            })
                            .show(ui);
                    });

                    ui.add_space(CAPTURE_MODAL_BODY_SPACING_MID);
                    crate::widgets::AlignCenter::new()
                        .shrink_to_fit(true)
                        .content(|ui| {
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
                        })
                        .show(ui);
                    ui.add_space(CAPTURE_MODAL_BODY_SPACING_MID);
                    modal_widgets::ModalWidgets::render_action_buttons(
                        ui, state, cmd, &temp_shortcut, recording_id_salt, os_bindings, i18n,
                    );
                });
            });
    }
}
