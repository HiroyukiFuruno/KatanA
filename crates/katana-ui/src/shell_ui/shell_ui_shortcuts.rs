use crate::shell::KatanaApp;
use crate::state::command_inventory::CommandInventory;
use eframe::egui;

fn parse_shortcut(s: &str) -> Option<egui::KeyboardShortcut> {
    let mut modifiers = egui::Modifiers::NONE;
    let mut key = None;

    for part in s.split('+') {
        let p = part.trim().to_lowercase();
        match p.as_str() {
            "primary" | "cmd" | "command" | "ctrl" => modifiers.command = true,
            "shift" => modifiers.shift = true,
            "alt" | "option" => modifiers.alt = true,
            "mac_cmd" => modifiers.mac_cmd = true,
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
        "space" => Some(egui::Key::Space),
        "enter" => Some(egui::Key::Enter),
        "esc" | "escape" => Some(egui::Key::Escape),
        "tab" => Some(egui::Key::Tab),
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
        let os_bindings = self
            .state
            .config
            .settings
            .settings()
            .shortcuts
            .current_os_bindings();

        for cmd in CommandInventory::all() {
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
                /* WHY: If a shortcut for this command matched, stop processing others to prevent ambiguous multi-fire */
                break;
            }
        }
    }
}
