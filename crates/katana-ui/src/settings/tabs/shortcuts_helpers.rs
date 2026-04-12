use crate::app_state::AppState;
use crate::i18n::I18nOps;
use crate::state::command_inventory::{CommandInventory, CommandInventoryItem};
use eframe::egui;
use std::collections::HashMap;

pub struct ShortcutsHelpersOps;

impl ShortcutsHelpersOps {
    /* WHY: Saves a new shortcut after verifying if it causes a conflict */
    pub(crate) fn check_and_save_shortcut(
        ui: &mut egui::Ui,
        state: &mut AppState,
        cmd: &CommandInventoryItem,
        new_shortcut: &str,
        recording_id_salt: egui::Id,
        os_bindings: &HashMap<String, String>,
    ) {
        let mut conflict_id = None;
        for other_cmd in CommandInventory::all() {
            if other_cmd.id == cmd.id {
                continue;
            }

            let other_shortcuts: Vec<String> = if let Some(custom) = os_bindings.get(other_cmd.id) {
                vec![custom.clone()]
            } else {
                other_cmd
                    .default_shortcuts
                    .iter()
                    .map(|&s| s.to_string())
                    .collect()
            };

            if other_shortcuts.contains(&new_shortcut.to_string()) {
                conflict_id = Some(other_cmd);
                break;
            }
        }

        if let Some(conflict_cmd) = conflict_id {
            let conflict_name = (conflict_cmd.label)();
            let msg = I18nOps::get()
                .settings
                .shortcuts
                .conflict_warning
                .replace("{command}", &conflict_name);
            ui.memory_mut(|mem| {
                mem.data
                    .insert_temp(egui::Id::new("shortcut_conflict"), msg)
            });
        } else {
            ui.memory_mut(|mem| {
                mem.data
                    .remove::<String>(egui::Id::new("shortcut_conflict"))
            });
        }

        let s = state.config.settings.settings_mut();
        let map = match std::env::consts::OS {
            "macos" => &mut s.shortcuts.macos,
            "windows" => &mut s.shortcuts.windows,
            _ => &mut s.shortcuts.linux,
        };
        map.insert(cmd.id.to_string(), new_shortcut.to_string());
        state.config.try_save_settings();

        ui.memory_mut(|mem| mem.data.remove::<String>(recording_id_salt));
    }

    /* WHY: Pushes a modifier part if the condition is true */
    pub(crate) fn add_modifier_if<'a>(cond: bool, name: &'a str, parts: &mut Vec<&'a str>) {
        if cond {
            parts.push(name);
        }
    }

    /* WHY: Maps an egui key enum to its string representation used in shortcut storage */
    pub(crate) fn key_to_string(key: egui::Key) -> &'static str {
        match key {
            egui::Key::ArrowDown => "down",
            egui::Key::ArrowLeft => "left",
            egui::Key::ArrowRight => "right",
            egui::Key::ArrowUp => "up",
            egui::Key::Escape => "esc",
            egui::Key::Tab => "tab",
            egui::Key::Backspace => "backspace",
            egui::Key::Enter => "enter",
            egui::Key::Space => "space",
            egui::Key::Delete => "delete",
            egui::Key::Comma => "comma",
            egui::Key::Period => ".",
            egui::Key::Num0 => "0",
            egui::Key::Num1 => "1",
            egui::Key::Num2 => "2",
            egui::Key::Num3 => "3",
            egui::Key::Num4 => "4",
            egui::Key::Num5 => "5",
            egui::Key::Num6 => "6",
            egui::Key::Num7 => "7",
            egui::Key::Num8 => "8",
            egui::Key::Num9 => "9",
            egui::Key::A => "a",
            egui::Key::B => "b",
            egui::Key::C => "c",
            egui::Key::D => "d",
            egui::Key::E => "e",
            egui::Key::F => "f",
            egui::Key::G => "g",
            egui::Key::H => "h",
            egui::Key::I => "i",
            egui::Key::J => "j",
            egui::Key::K => "k",
            egui::Key::L => "l",
            egui::Key::M => "m",
            egui::Key::N => "n",
            egui::Key::O => "o",
            egui::Key::P => "p",
            egui::Key::Q => "q",
            egui::Key::R => "r",
            egui::Key::S => "s",
            egui::Key::T => "t",
            egui::Key::U => "u",
            egui::Key::V => "v",
            egui::Key::W => "w",
            egui::Key::X => "x",
            egui::Key::Y => "y",
            egui::Key::Z => "z",
            _ => "",
        }
    }

    /* WHY: Handles key input recording when the edit button is active */
    pub(crate) fn handle_shortcut_recording(
        ui: &mut egui::Ui,
        state: &mut AppState,
        cmd: &CommandInventoryItem,
        recording_id_salt: egui::Id,
        os_bindings: &HashMap<String, String>,
    ) {
        let (should_cancel, keys, modifiers) = ui.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                (true, Vec::new(), i.modifiers)
            } else {
                let pressed_keys: Vec<egui::Key> = i
                    .events
                    .iter()
                    .filter_map(|e| {
                        if let egui::Event::Key {
                            key, pressed: true, ..
                        } = e
                        {
                            Some(*key)
                        } else {
                            None
                        }
                    })
                    .collect();
                (false, pressed_keys, i.modifiers)
            }
        });

        if should_cancel {
            ui.memory_mut(|mem| mem.data.remove::<String>(recording_id_salt));
        } else if let Some(&key) = keys.first() {
            let key_str = Self::key_to_string(key);

            if !key_str.is_empty() {
                let mut parts = Vec::new();
                Self::add_modifier_if(modifiers.command, "primary", &mut parts);
                Self::add_modifier_if(modifiers.shift, "shift", &mut parts);
                Self::add_modifier_if(modifiers.alt, "alt", &mut parts);
                Self::add_modifier_if(modifiers.mac_cmd, "mac_cmd", &mut parts);
                parts.push(key_str);

                let new_shortcut = parts.join("+");

                Self::check_and_save_shortcut(
                    ui,
                    state,
                    cmd,
                    &new_shortcut,
                    recording_id_salt,
                    os_bindings,
                );
            }
        }
    }
}
