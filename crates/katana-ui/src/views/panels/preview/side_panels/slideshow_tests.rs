use super::super::{PreviewMenu, PreviewMenuAvailability};
use super::PreviewSidePanels;
use crate::app::action::ActionOps;
use crate::app_state::{AppAction, AppState};
use crate::shell::KatanaApp;
use eframe::egui;
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
use std::path::Path;

fn make_app() -> KatanaApp {
    KatanaApp::new(AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    ))
}

fn fullscreen_commands(path: Option<&str>, was_fullscreen: bool) -> Vec<egui::ViewportCommand> {
    let mut app = make_app();
    if let Some(path) = path {
        app.state
            .document
            .open_documents
            .push(katana_core::document::Document::new_empty(path));
        app.state.document.active_doc_idx = Some(0);
        app.state.layout.show_slideshow = true;
    }
    app.state.layout.was_os_fullscreen_before_slideshow = was_fullscreen;
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        app.process_action(ui.ctx(), AppAction::ToggleSlideshow);
        app.show_system_modals(ui.ctx());
    });
    output.textures_delta.clear();
    output
        .viewport_output
        .values()
        .flat_map(|viewport| viewport.commands.iter().cloned())
        .collect()
}

fn count_fullscreen(commands: &[egui::ViewportCommand], value: bool) -> usize {
    commands
        .iter()
        .filter(|command| {
            matches!(command, egui::ViewportCommand::Fullscreen(actual) if *actual == value)
        })
        .count()
}

#[test]
fn empty_workspace_disables_only_slideshow_menu() {
    assert!(!PreviewMenuAvailability::for_path(
        None,
        PreviewMenu::Slideshow
    ));
    for menu in [
        PreviewMenu::Toc,
        PreviewMenu::Export,
        PreviewMenu::Story,
        PreviewMenu::Tools,
    ] {
        assert!(PreviewMenuAvailability::for_path(None, menu));
    }
}

#[test]
fn toggle_slideshow_without_document_does_not_enter_fullscreen() {
    let commands = fullscreen_commands(None, false);
    assert_eq!(count_fullscreen(&commands, true), 0);
    assert_eq!(count_fullscreen(&commands, false), 0);
}

#[test]
fn unavailable_toggle_restores_only_when_not_already_fullscreen() {
    for path in [
        "report.html",
        "report.docx",
        "book.xlsx",
        "photo.png",
        "manual.pdf",
    ] {
        let commands = fullscreen_commands(Some(path), false);
        assert_eq!(count_fullscreen(&commands, true), 0, "{path}");
        assert_eq!(count_fullscreen(&commands, false), 1, "{path}");
    }
}

#[test]
fn unavailable_toggle_preserves_existing_fullscreen() {
    for path in [
        "report.html",
        "report.docx",
        "book.xlsx",
        "photo.png",
        "manual.pdf",
    ] {
        let commands = fullscreen_commands(Some(path), true);
        assert_eq!(count_fullscreen(&commands, true), 0, "{path}");
        assert_eq!(count_fullscreen(&commands, false), 0, "{path}");
    }
}

#[test]
fn repeated_unavailable_toggle_does_not_repeat_restore() {
    let mut app = make_app();
    app.state
        .document
        .open_documents
        .push(katana_core::document::Document::new_empty("report.html"));
    app.state.document.active_doc_idx = Some(0);
    app.state.layout.show_slideshow = true;
    app.state.layout.was_os_fullscreen_before_slideshow = false;
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        app.process_action(ui.ctx(), AppAction::ToggleSlideshow);
        app.process_action(ui.ctx(), AppAction::ToggleSlideshow);
    });
    output.textures_delta.clear();
    let commands: Vec<_> = output
        .viewport_output
        .values()
        .flat_map(|viewport| viewport.commands.iter().cloned())
        .collect();
    assert_eq!(count_fullscreen(&commands, false), 1);
}

#[test]
fn markdown_and_pptx_slideshow_remain_available() {
    for path in ["slides.md", "deck.pptx"] {
        assert!(PreviewMenuAvailability::for_path(
            Some(Path::new(path)),
            PreviewMenu::Slideshow
        ));
    }
}

#[test]
fn side_panel_unavailable_slideshow_restores_once_and_preserves_fullscreen() {
    for (path, label) in [
        (None, "no document"),
        (Some("report.html"), "report.html"),
        (Some("report.docx"), "report.docx"),
        (Some("book.xlsx"), "book.xlsx"),
        (Some("photo.png"), "photo.png"),
        (Some("manual.pdf"), "manual.pdf"),
    ] {
        for was_fullscreen in [false, true] {
            let mut app = make_app();
            if let Some(path) = path {
                app.state
                    .document
                    .open_documents
                    .push(katana_core::document::Document::new_empty(path));
                app.state.document.active_doc_idx = Some(0);
            }
            app.state.layout.show_slideshow = true;
            app.state.layout.was_os_fullscreen_before_slideshow = was_fullscreen;
            let ctx = egui::Context::default();
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                PreviewSidePanels::new(&mut app).show(ui);
            });
            output.textures_delta.clear();
            let commands: Vec<_> = output
                .viewport_output
                .values()
                .flat_map(|viewport| viewport.commands.iter().cloned())
                .collect();
            assert!(!app.state.layout.show_slideshow, "{label}");
            assert_eq!(
                count_fullscreen(&commands, false),
                usize::from(!was_fullscreen)
            );

            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                PreviewSidePanels::new(&mut app).show(ui);
            });
            output.textures_delta.clear();
            let commands: Vec<_> = output
                .viewport_output
                .values()
                .flat_map(|viewport| viewport.commands.iter().cloned())
                .collect();
            assert_eq!(count_fullscreen(&commands, false), 0, "{label}");
        }
    }
}
