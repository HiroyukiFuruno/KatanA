use super::shortcut_keys::ShortcutKeyOps;
use crate::shell::KatanaApp;
use crate::state::command_inventory::{CommandInventory, CommandInventoryItem};
use crate::state::shortcut_context::{ShortcutContext, ShortcutContextResolver};
use eframe::egui;

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

        /* WHY: egui's consume_shortcut uses matches_logically, which ignores extra
        Shift/Alt modifiers. This means a shortcut like "primary+\" would also match
        when the user presses "primary+Shift+\". By sorting commands so that shortcuts
        with more modifiers are evaluated first, we ensure the more-specific shortcut
        is consumed before the less-specific one can match. */
        let mut commands = CommandInventory::all();
        commands.sort_by(|a, b| {
            let count_mods = |cmd: &CommandInventoryItem| -> usize {
                let shortcuts: Vec<String> = if let Some(custom) = os_bindings.get(cmd.id) {
                    vec![custom.clone()]
                } else {
                    cmd.default_shortcuts
                        .iter()
                        .map(|&s| s.to_string())
                        .collect()
                };
                shortcuts
                    .iter()
                    .map(|s| s.split('+').count())
                    .max()
                    .unwrap_or(0)
            };
            count_mods(b).cmp(&count_mods(a))
        });

        for cmd in commands {
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

            if shortcuts_to_try
                .iter()
                .any(|raw_shortcut| command_shortcut_consumed(ctx, active_context, raw_shortcut))
            {
                self.pending_action = cmd.action.clone();
                /* WHY: Stop processing after the first match to prevent
                ambiguous multi-fire within the same frame. */
                break;
            }
        }
    }
}

fn command_shortcut_consumed(
    ctx: &egui::Context,
    active_context: ShortcutContext,
    raw_shortcut: &str,
) -> bool {
    let Some(parsed) = ShortcutKeyOps::parse_shortcut(raw_shortcut) else {
        return false;
    };
    if active_context == ShortcutContext::Editor && ShortcutKeyOps::editor_keeps_shortcut(&parsed) {
        return false;
    }
    ctx.input_mut(|i| i.consume_shortcut(&parsed))
}
