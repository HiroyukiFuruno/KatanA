use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use eframe::egui;

impl KatanaApp {
    pub(super) fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let cmd_shift_t = egui::KeyboardShortcut::new(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::T,
        );
        if ctx.input_mut(|i| i.consume_shortcut(&cmd_shift_t))
            && !self.state.document.recently_closed_tabs.is_empty()
        {
            self.pending_action = AppAction::RestoreClosedDocument;
        }

        let cmd_p = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::P);
        let cmd_shift_p = egui::KeyboardShortcut::new(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::P,
        );
        let cmd_k = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::K);
        if ctx.input_mut(|i| {
            i.consume_shortcut(&cmd_p)
                || i.consume_shortcut(&cmd_shift_p)
                || i.consume_shortcut(&cmd_k)
        }) {
            self.pending_action = AppAction::ToggleCommandPalette;
        }

        let cmd_f = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::F);
        if ctx.input_mut(|i| i.consume_shortcut(&cmd_f)) {
            /* WHY: Virtual docs (Welcome, Guide, ChangeLog) do not support in-doc search. */
            let is_katana_virtual = self.state.active_document().is_some_and(|d| {
                let p = d.path.to_string_lossy();
                p.starts_with("Katana://Welcome")
                    || p.starts_with("Katana://Guide")
                    || p.starts_with("Katana://ChangeLog")
            });
            if !is_katana_virtual {
                if !self.state.search.doc_search_open {
                    self.state.search.doc_search_open = true;
                    ctx.memory_mut(|m| {
                        m.data
                            .insert_temp(egui::Id::new("search_newly_opened"), true)
                    });
                    self.trigger_action(AppAction::DocSearchQueryChanged);
                } else {
                    self.state.search.doc_search_open = false;
                    self.state.search.doc_search_matches.clear();
                }
            }
        }

        let cmd_w = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::W);
        if ctx.input_mut(|i| i.consume_shortcut(&cmd_w))
            && let Some(idx) = self.state.document.active_doc_idx
        {
            self.pending_action = AppAction::CloseDocument(idx);
        }

        let cmd_b = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::B);
        if ctx.input_mut(|i| i.consume_shortcut(&cmd_b)) {
            self.pending_action = AppAction::ToggleWorkspacePanel;
        }

        let cmd_shift_f = egui::KeyboardShortcut::new(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::F,
        );
        if ctx.input_mut(|i| i.consume_shortcut(&cmd_shift_f)) {
            self.pending_action = AppAction::ToggleSearchModal;
        }

        let cmd_s = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::S);
        if ctx.input_mut(|i| i.consume_shortcut(&cmd_s)) {
            self.pending_action = AppAction::SaveDocument;
        }

        let cmd_o = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::O);
        if ctx.input_mut(|i| i.consume_shortcut(&cmd_o)) {
            self.pending_action = AppAction::PickOpenWorkspace;
        }

        let cmd_r = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::R);
        if ctx.input_mut(|i| i.consume_shortcut(&cmd_r)) && self.state.active_document().is_some() {
            self.pending_action = AppAction::RefreshDocument { is_manual: true };
        }

        /* WHY: Command+Option-D opens the demo workspace. COMMAND | ALT correctly maps
         * to macOS Command-Option — using struct literal misses mac_cmd flag. */
        let cmd_opt_d = egui::KeyboardShortcut::new(
            egui::Modifiers::COMMAND | egui::Modifiers::ALT,
            egui::Key::D,
        );
        if ctx.input_mut(|i| i.consume_shortcut(&cmd_opt_d)) {
            self.pending_action = AppAction::OpenHelpDemo;
        }
    }
}
