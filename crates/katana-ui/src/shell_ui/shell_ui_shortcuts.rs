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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::{AppAction, AppState};
    use crate::editor_undo::EditorUndoIdentity;
    use katana_core::{ai::AiProviderRegistry, document::Document, plugin::PluginRegistry};
    use std::path::Path;
    use std::sync::Arc;

    fn app_with_focused_editor(ctx: &egui::Context) -> KatanaApp {
        let mut state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            Default::default(),
            Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        state
            .document
            .open_documents
            .push(Document::new("/tmp/editor.md", ""));
        state.document.active_doc_idx = Some(0);

        let editor_id = EditorUndoIdentity::text_edit_id(None, Path::new("/tmp/editor.md"));
        ctx.memory_mut(|mem| {
            mem.request_focus(editor_id);
        });

        KatanaApp::new(state)
    }

    fn shifted_primary_v_input() -> egui::RawInput {
        let mut modifiers = egui::Modifiers::NONE;
        modifiers.command = true;
        modifiers.shift = true;
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Key {
            key: egui::Key::V,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        });
        input
    }

    #[test]
    fn handle_shortcuts_queues_clipboard_image_for_shifted_primary_v_in_editor() {
        let ctx = egui::Context::default();
        let mut app = app_with_focused_editor(&ctx);

        let _ = ctx.run(shifted_primary_v_input(), |ctx| {
            app.handle_shortcuts(ctx);
        });

        assert!(matches!(
            app.pending_action,
            AppAction::IngestClipboardImage
        ));
    }
}
