use super::helpers::ShortcutsHelpersOps;
use eframe::egui;

pub struct KeyEventsOps;

impl KeyEventsOps {
    /* WHY: Extracts the key press combination from egui events, consuming key events to
    prevent app-level shortcut reactions while the capture modal is open. */
    pub fn capture_key_from_events(
        ui: &mut egui::Ui,
        is_mac: bool,
    ) -> (bool, bool, Option<(String, egui::Modifiers)>) {
        ui.input_mut(|input| {
            let mut should_cancel = false;
            let mut should_confirm = false;
            let mut result_shortcut = None;

            for e in &input.events {
                match e {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers: ev_modifiers,
                        repeat: false,
                        ..
                    } => {
                        let (cancel, confirm, res) =
                            Self::process_key_event(*key, *ev_modifiers, is_mac);
                        should_cancel |= cancel;
                        should_confirm |= confirm;
                        if res.is_some() {
                            result_shortcut = res;
                        }
                    }
                    egui::Event::Text(text) => {
                        if let Some(res) = Self::process_text_event(text, input.modifiers, is_mac) {
                            result_shortcut = Some(res);
                        }
                    }
                    _ => {}
                }
            }

            /* WHY: Always consume key events to block app-level reactions during recording */
            input
                .events
                .retain(|e| !matches!(e, egui::Event::Key { .. }));

            (should_cancel, should_confirm, result_shortcut)
        })
    }

    fn process_key_event(
        key: egui::Key,
        m: egui::Modifiers,
        is_mac: bool,
    ) -> (bool, bool, Option<(String, egui::Modifiers)>) {
        if key == egui::Key::Escape {
            return (true, false, None);
        }
        if key == egui::Key::Enter && m.is_none() {
            return (false, true, None);
        }
        let ks = ShortcutsHelpersOps::key_to_string(key);
        if ks.is_empty() {
            return (false, false, None);
        }
        let mut p = Vec::new();
        if m.ctrl {
            p.push(String::from(if is_mac { "ctrl" } else { "primary" }));
        }
        if m.mac_cmd {
            p.push(String::from(if is_mac { "primary" } else { "win" }));
        }
        if m.shift {
            p.push(String::from("shift"));
        }
        if m.alt {
            p.push(String::from("alt"));
        }
        p.push(ks.to_string());
        (false, false, Some((p.join("+"), m)))
    }

    fn process_text_event(
        text: &str,
        m: egui::Modifiers,
        is_mac: bool,
    ) -> Option<(String, egui::Modifiers)> {
        if !matches!(
            text,
            "@" | "^" | "¥" | "|" | "_" | "[" | "]" | ":" | ";" | "\\" | "/"
        ) {
            return None;
        }
        let mut p = Vec::new();
        if m.ctrl {
            p.push(String::from(if is_mac { "ctrl" } else { "primary" }));
        }
        if m.mac_cmd {
            p.push(String::from(if is_mac { "primary" } else { "win" }));
        }
        if m.alt {
            p.push(String::from("alt"));
        }
        /* WHY: Since text gives the verbatim char (e.g. "@"), we often don't need Shift,
        but we provide the shift explicitly so it aligns with VSCode format. */
        if m.shift {
            p.push(String::from("shift"));
        }
        /* WHY: On JIS keyboards, ¥ (U+00A5) and | (Shift+¥) are the same physical key
        as Backslash. Normalize both to "\\" so that captured shortcuts are stored in
        canonical form and match against default_shortcuts. */
        let canonical = if text == "¥" || text == "|" {
            "\\"
        } else {
            text
        };
        p.push(canonical.to_string());
        Some((p.join("+"), m))
    }
}
