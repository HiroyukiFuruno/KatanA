use crate::shell::KatanaApp;
use crate::state::command_inventory::CommandInventory;
use crate::state::shortcut_context::{ShortcutContext, ShortcutContextResolver};
use eframe::egui;

/// WHY: Strips the legacy `[editor]` context suffix from shortcut strings.
/// This suffix was used before ShortcutContext was introduced to hint that
/// a shortcut belongs to the editor. The suffix is now deprecated — context
/// is encoded in CommandInventoryItem::context — but we still strip it to
/// avoid parse failures for any user-customized shortcuts that may still
/// carry the old format in their persisted settings.
fn strip_context_suffix(s: &str) -> &str {
    s.strip_suffix("[editor]").unwrap_or(s)
}

fn parse_shortcut(s: &str) -> Option<egui::KeyboardShortcut> {
    let clean = strip_context_suffix(s);
    let mut modifiers = egui::Modifiers::NONE;
    let mut key = None;

    for part in clean.split('+') {
        let p = part.trim().to_lowercase();
        match p.as_str() {
            "primary" | "cmd" | "command" => modifiers.command = true,
            "ctrl" => modifiers.ctrl = true,
            "shift" => modifiers.shift = true,
            "alt" | "option" => modifiers.alt = true,
            "mac_cmd" | "win" | "super" | "meta" => modifiers.mac_cmd = true,
            _ => {
                key = parse_key(&p);
            }
        }
    }

    key.map(|k| egui::KeyboardShortcut::new(modifiers, k))
}

fn parse_key(s: &str) -> Option<egui::Key> {
    match s {
        "a" => Some(egui::Key::A),
        "b" => Some(egui::Key::B),
        "c" => Some(egui::Key::C),
        "d" => Some(egui::Key::D),
        "e" => Some(egui::Key::E),
        "f" => Some(egui::Key::F),
        "g" => Some(egui::Key::G),
        "h" => Some(egui::Key::H),
        "i" => Some(egui::Key::I),
        "j" => Some(egui::Key::J),
        "k" => Some(egui::Key::K),
        "l" => Some(egui::Key::L),
        "m" => Some(egui::Key::M),
        "n" => Some(egui::Key::N),
        "o" => Some(egui::Key::O),
        "p" => Some(egui::Key::P),
        "q" => Some(egui::Key::Q),
        "r" => Some(egui::Key::R),
        "s" => Some(egui::Key::S),
        "t" => Some(egui::Key::T),
        "u" => Some(egui::Key::U),
        "v" => Some(egui::Key::V),
        "w" => Some(egui::Key::W),
        "x" => Some(egui::Key::X),
        "y" => Some(egui::Key::Y),
        "z" => Some(egui::Key::Z),
        "0" => Some(egui::Key::Num0),
        "1" => Some(egui::Key::Num1),
        "2" => Some(egui::Key::Num2),
        "3" => Some(egui::Key::Num3),
        "4" => Some(egui::Key::Num4),
        "5" => Some(egui::Key::Num5),
        "6" => Some(egui::Key::Num6),
        "7" => Some(egui::Key::Num7),
        "8" => Some(egui::Key::Num8),
        "9" => Some(egui::Key::Num9),
        "," => Some(egui::Key::Comma),
        "." => Some(egui::Key::Period),
        "/" => Some(egui::Key::Slash),
        "\\" | "¥" => Some(egui::Key::Backslash),
        "-" => Some(egui::Key::Minus),
        "=" => Some(egui::Key::Equals),
        ";" | "semicolon" => Some(egui::Key::Semicolon),
        ":" | "colon" => Some(egui::Key::Colon),
        "{" => Some(egui::Key::OpenCurlyBracket),
        "}" => Some(egui::Key::CloseCurlyBracket),
        "[" => Some(egui::Key::OpenBracket),
        "]" => Some(egui::Key::CloseBracket),
        "`" => Some(egui::Key::Backtick),
        "@" => Some(egui::Key::OpenBracket),
        "^" => Some(egui::Key::Equals),
        "_" => Some(egui::Key::Minus),
        "space" => Some(egui::Key::Space),
        "enter" => Some(egui::Key::Enter),
        "esc" | "escape" => Some(egui::Key::Escape),
        "tab" => Some(egui::Key::Tab),
        "|" => Some(egui::Key::Pipe),
        "?" => Some(egui::Key::Questionmark),
        "!" => Some(egui::Key::Exclamationmark),
        "\"" | "'" | "quote" => Some(egui::Key::Quote),
        "backspace" => Some(egui::Key::Backspace),
        "delete" => Some(egui::Key::Delete),
        "up" => Some(egui::Key::ArrowUp),
        "down" => Some(egui::Key::ArrowDown),
        "left" => Some(egui::Key::ArrowLeft),
        "right" => Some(egui::Key::ArrowRight),
        _ => None,
    }
}

impl KatanaApp {
    pub(super) fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let active_context = ShortcutContextResolver::resolve(&self.state, ctx);

        /* WHY: During shortcut recording the user is intentionally pressing
        keys to assign them. We must not intercept those inputs as commands. */
        if active_context == ShortcutContext::Recording {
            return;
        }

        let os_bindings = self
            .state
            .config
            .settings
            .settings()
            .shortcuts
            .current_os_bindings();

        for cmd in CommandInventory::all() {
            /* WHY: Skip commands whose context does not match the active context.
            Global commands fire anywhere except Recording/Modal.
            Editor commands fire only when the text editor has focus. */
            if !ShortcutContextResolver::context_allows(cmd.context, active_context) {
                continue;
            }

            if !(cmd.is_available)(&self.state) {
                continue;
            }

            let shortcuts_to_try: Vec<String> = if let Some(custom) = os_bindings.get(cmd.id) {
                vec![custom.clone()]
            } else {
                cmd.default_shortcuts
                    .iter()
                    .map(|&s| s.to_string())
                    .collect()
            };

            let mut matched = false;
            for raw_shortcut in shortcuts_to_try {
                if let Some(parsed) = parse_shortcut(&raw_shortcut)
                    && ctx.input_mut(|i| i.consume_shortcut(&parsed))
                {
                    self.pending_action = cmd.action.clone();
                    matched = true;
                    break;
                }
            }
            if matched {
                /* WHY: Stop processing after the first match to prevent
                ambiguous multi-fire within the same frame. */
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_context_suffix_removes_editor_tag() {
        assert_eq!(strip_context_suffix("primary+B[editor]"), "primary+B");
    }

    #[test]
    fn strip_context_suffix_leaves_plain_shortcut_unchanged() {
        assert_eq!(strip_context_suffix("primary+S"), "primary+S");
    }

    #[test]
    fn parse_shortcut_handles_backtick() {
        let result = parse_shortcut("primary+`");
        assert!(
            result.is_some(),
            "backtick shortcut must parse successfully"
        );
    }

    #[test]
    fn parse_shortcut_ignores_unknown_key() {
        let result = parse_shortcut("primary+@@@");
        assert!(result.is_none());
    }
}
