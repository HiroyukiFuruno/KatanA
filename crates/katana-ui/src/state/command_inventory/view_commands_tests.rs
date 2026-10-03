use super::ViewCommands;
use crate::app_state::AppState;
use crate::state::command_inventory::CommandInventoryItem;
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
use katana_platform::SettingsService;
use std::sync::Arc;

fn state_with_active_path(path: &str) -> AppState {
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        SettingsService::default(),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state
        .document
        .open_documents
        .push(katana_core::document::Document::new_empty(path));
    state.document.active_doc_idx = Some(0);
    state
}

fn view_command(id: &str) -> CommandInventoryItem {
    ViewCommands::get()
        .into_iter()
        .find(|command| command.id == id)
        .unwrap_or_else(|| panic!("missing command {id}"))
}

#[test]
fn registered_view_tools_commands_are_unavailable_for_office_and_html_documents() {
    for path in [
        "report.html",
        "REPORT.HTM",
        "report.docx",
        "book.xlsx",
        "deck.pptx",
    ] {
        let state = state_with_active_path(path);
        for id in ["view.toggle_split_mode", "view.toggle_code_preview"] {
            assert!(
                !(view_command(id).is_available)(&state),
                "{id} must be unavailable for {path}"
            );
        }
    }
}

#[test]
fn registered_view_tools_commands_remain_available_for_pdf_and_markdown_documents() {
    for path in ["report.pdf", "readme.md"] {
        let state = state_with_active_path(path);
        for id in ["view.toggle_split_mode", "view.toggle_code_preview"] {
            assert!(
                (view_command(id).is_available)(&state),
                "{id} must remain available for {path}"
            );
        }
    }
}
