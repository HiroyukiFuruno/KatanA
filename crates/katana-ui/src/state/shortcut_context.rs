use crate::app_state::AppState;
use eframe::egui;

/// WHY: Defines the focus context in which a shortcut is valid.
/// This prevents context-unaware shortcut firing (e.g., primary+B firing
/// both Bold and Toggle Explorer simultaneously).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutContext {
    /// Fires anywhere unless a more specific context is active.
    Global,
    /// Active only when the text editor pane has keyboard focus.
    Editor,
    /// Active only when the preview pane is the primary focus area.
    Preview,
    /// Active only when the sidebar explorer has focus (future use).
    Explorer,
    /// Active only when any modal overlay is open (command palette,
    /// search modal, file ops, etc.). Suppresses Global shortcuts.
    Modal,
    /// Active during shortcut key recording in settings.
    /// ALL other shortcuts are suppressed while this is active.
    Recording,
}

pub struct ShortcutContextResolver;

impl ShortcutContextResolver {
    /// WHY: Returns the highest-priority active context for the current
    /// frame. Priority: Recording > Modal > Editor > Preview > Explorer > Global.
    pub fn resolve(state: &AppState, ctx: &egui::Context) -> ShortcutContext {
        if Self::is_recording(ctx) {
            return ShortcutContext::Recording;
        }
        if Self::is_modal_active(state) {
            return ShortcutContext::Modal;
        }
        if Self::is_editor_focused(state, ctx) {
            return ShortcutContext::Editor;
        }
        if Self::is_preview_focused(state) {
            return ShortcutContext::Preview;
        }
        ShortcutContext::Global
    }

    /// WHY: Checks whether a shortcut recording session is in progress.
    /// The recording state is stored in egui temporary memory.
    pub fn is_recording(ctx: &egui::Context) -> bool {
        ctx.memory(|mem| {
            mem.data
                .get_temp::<String>(egui::Id::new("recording_shortcut_id"))
                .is_some_and(|s| !s.is_empty())
        })
    }

    /// WHY: Any modal-family overlay suppresses Global shortcuts to prevent
    /// accidental command firing while the user interacts with a modal.
    fn is_modal_active(state: &AppState) -> bool {
        state.command_palette.is_open
            || state.layout.show_search_modal
            || state.layout.create_fs_node_modal.is_some()
            || state.layout.rename_modal.is_some()
            || state.layout.delete_modal.is_some()
            || state.layout.pending_close_confirm.is_some()
            || state.layout.rename_tab_group_modal.is_some()
    }

    /// WHY: Editor context is active when the text-edit widget within the
    /// editor pane has keyboard focus. We use the stable egui Id associated
    /// with the editor's TextEdit widget.
    fn is_editor_focused(state: &AppState, ctx: &egui::Context) -> bool {
        if state.document.active_doc_idx.is_none() {
            return false;
        }
        ctx.memory(|mem| {
            mem.focused()
                .is_some_and(|id| id == egui::Id::new("editor_text_edit"))
        })
    }

    /// WHY: Preview / fullscreen / slideshow modes are treated as a dedicated
    /// Preview context so that presentation-specific shortcuts can fire.
    fn is_preview_focused(state: &AppState) -> bool {
        state.layout.show_slideshow
    }
    /// WHY: Determines whether the given command context is compatible with the
    /// currently active context. Global commands fire in any non-Recording context.
    /// Other commands fire only when active_context matches exactly.
    pub fn context_allows(
        command_context: ShortcutContext,
        active_context: ShortcutContext,
    ) -> bool {
        match active_context {
            ShortcutContext::Recording => false,
            ShortcutContext::Modal => {
                /* WHY: In Modal context only Modal-scoped commands fire.
                Currently no commands have Modal context; the palette handles
                its own key input directly. */
                command_context == ShortcutContext::Modal
            }
            other => command_context == ShortcutContext::Global || command_context == other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use katana_core::document::Document;

    fn state_with_active_doc() -> AppState {
        let mut state = AppState::new(
            Default::default(),
            Default::default(),
            Default::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        state
            .document
            .open_documents
            .push(Document::new("/tmp/editor.md", ""));
        state.document.active_doc_idx = Some(0);
        state
    }

    #[test]
    fn global_context_fires_in_global_active() {
        assert!(ShortcutContextResolver::context_allows(
            ShortcutContext::Global,
            ShortcutContext::Global
        ));
    }

    #[test]
    fn global_context_fires_in_editor_active() {
        assert!(ShortcutContextResolver::context_allows(
            ShortcutContext::Global,
            ShortcutContext::Editor
        ));
    }

    #[test]
    fn global_context_fires_in_preview_active() {
        assert!(ShortcutContextResolver::context_allows(
            ShortcutContext::Global,
            ShortcutContext::Preview
        ));
    }

    #[test]
    fn global_does_not_fire_in_recording() {
        assert!(!ShortcutContextResolver::context_allows(
            ShortcutContext::Global,
            ShortcutContext::Recording
        ));
    }

    #[test]
    fn global_does_not_fire_in_modal() {
        assert!(!ShortcutContextResolver::context_allows(
            ShortcutContext::Global,
            ShortcutContext::Modal
        ));
    }

    #[test]
    fn editor_context_fires_in_editor_active() {
        assert!(ShortcutContextResolver::context_allows(
            ShortcutContext::Editor,
            ShortcutContext::Editor
        ));
    }

    #[test]
    fn editor_context_does_not_fire_in_global_active() {
        assert!(!ShortcutContextResolver::context_allows(
            ShortcutContext::Editor,
            ShortcutContext::Global
        ));
    }

    #[test]
    fn editor_context_does_not_fire_in_recording() {
        assert!(!ShortcutContextResolver::context_allows(
            ShortcutContext::Editor,
            ShortcutContext::Recording
        ));
    }

    #[test]
    fn editor_context_does_not_fire_in_modal() {
        assert!(!ShortcutContextResolver::context_allows(
            ShortcutContext::Editor,
            ShortcutContext::Modal
        ));
    }

    #[test]
    fn preview_context_fires_in_preview_active() {
        assert!(ShortcutContextResolver::context_allows(
            ShortcutContext::Preview,
            ShortcutContext::Preview
        ));
    }

    #[test]
    fn preview_context_does_not_fire_in_editor_active() {
        assert!(!ShortcutContextResolver::context_allows(
            ShortcutContext::Preview,
            ShortcutContext::Editor
        ));
    }

    #[test]
    fn modal_context_fires_in_modal_active() {
        assert!(ShortcutContextResolver::context_allows(
            ShortcutContext::Modal,
            ShortcutContext::Modal
        ));
    }

    #[test]
    fn recording_context_never_fires() {
        assert!(!ShortcutContextResolver::context_allows(
            ShortcutContext::Recording,
            ShortcutContext::Recording
        ));
        assert!(!ShortcutContextResolver::context_allows(
            ShortcutContext::Recording,
            ShortcutContext::Global
        ));
    }

    #[test]
    fn resolve_returns_editor_when_text_edit_has_stable_focus_id() {
        let state = state_with_active_doc();
        let ctx = egui::Context::default();
        ctx.memory_mut(|mem| mem.request_focus(egui::Id::new("editor_text_edit")));

        assert_eq!(
            ShortcutContextResolver::resolve(&state, &ctx),
            ShortcutContext::Editor
        );
    }

    #[test]
    fn resolve_ignores_editor_focus_without_active_document() {
        let state = AppState::new(
            Default::default(),
            Default::default(),
            Default::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        let ctx = egui::Context::default();
        ctx.memory_mut(|mem| mem.request_focus(egui::Id::new("editor_text_edit")));

        assert_eq!(
            ShortcutContextResolver::resolve(&state, &ctx),
            ShortcutContext::Global
        );
    }
}
