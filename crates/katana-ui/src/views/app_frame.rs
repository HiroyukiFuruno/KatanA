//! Main application frame layout rendering.
//!
//! Renders the top-level panels when the splash screen is not opaque:
//! menu bar, status bar, title bar, workspace sidebar, tab toolbar,
//! breadcrumbs, and central content area.

use crate::app::action::ActionOps;
use crate::app_state::{AppAction, ViewMode};
use crate::preview_pane::DownloadRequest;
use crate::shell::{KatanaApp, SIDEBAR_COLLAPSED_TOGGLE_WIDTH};
use crate::shell_ui::relative_full_path;
use crate::theme_bridge;
use eframe::egui;

const CHEVRON_ICON_SIZE: f32 = 10.0;

/// Renders the main application panels (everything inside the `if !splash_is_opaque` guard).
///
/// Returns an optional `DownloadRequest` produced by split preview rendering.
pub(crate) fn render_main_panels(
    ctx: &egui::Context,
    app: &mut KatanaApp,
    theme_colors: &katana_platform::theme::ThemeColors,
) -> Option<DownloadRequest> {
    // Menu bar & status bar
    crate::views::top_bar::render_menu_bar(ctx, &mut app.state, &mut app.pending_action);
    let export_filenames: Vec<String> = app
        .export_tasks
        .iter()
        .map(|t| t.filename.clone())
        .collect();
    crate::views::top_bar::render_status_bar(ctx, &app.state, &export_filenames);

    // Window title
    render_window_title(ctx, app);

    // In-app title bar
    render_title_bar(ctx, app, theme_colors);

    // Workspace sidebar
    render_workspace_sidebar(ctx, app);

    // Tab toolbar (tabs + breadcrumbs + view mode)
    render_tab_toolbar(ctx, app);

    // Central content area
    render_central_content(ctx, app)
}

fn render_window_title(ctx: &egui::Context, app: &mut KatanaApp) {
    let ws_root_for_title = app.state.workspace.data.as_ref().map(|ws| ws.root.clone());
    let title_text = match app.state.active_document() {
        Some(doc) => {
            let fname = doc.file_name().unwrap_or("");
            let rel = relative_full_path(&doc.path, ws_root_for_title.as_deref());
            crate::shell_logic::format_window_title(
                fname,
                &rel,
                &crate::i18n::get().menu.release_notes,
            )
        }
        None => "KatanA".to_string(),
    };
    if app.state.layout.last_window_title != title_text {
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title_text.clone()));
        app.state.layout.last_window_title = title_text;
    }
}

fn render_title_bar(
    ctx: &egui::Context,
    app: &KatanaApp,
    theme_colors: &katana_platform::theme::ThemeColors,
) {
    let title_text = &app.state.layout.last_window_title;
    egui::TopBottomPanel::top("app_title_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.centered_and_justified(|ui| {
                let title_color = theme_bridge::rgb_to_color32(theme_colors.system.title_bar_text);
                ui.label(egui::RichText::new(title_text).small().color(title_color));
            });
        });
    });
}

fn render_workspace_sidebar(ctx: &egui::Context, app: &mut KatanaApp) {
    if !app.state.layout.show_workspace {
        egui::SidePanel::left("workspace_collapsed")
            .resizable(false)
            .exact_width(SIDEBAR_COLLAPSED_TOGGLE_WIDTH)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    if ui
                        .add(egui::Button::image(
                            crate::Icon::ChevronRight.ui_image(ui, crate::icon::IconSize::Medium),
                        ))
                        .on_hover_text(crate::i18n::get().workspace.workspace_title.clone())
                        .clicked()
                    {
                        app.state.layout.show_workspace = true;
                    }
                });
            });
    } else {
        crate::views::panels::workspace::render_workspace_panel(
            ctx,
            &mut app.state,
            &mut app.pending_action,
        );
    }
}

fn render_tab_toolbar(ctx: &egui::Context, app: &mut KatanaApp) {
    egui::TopBottomPanel::top("tab_toolbar").show(ctx, |ui| {
        crate::views::top_bar::render_tab_bar(ui, &mut app.state, &mut app.pending_action);
        let active_doc_props = app.state.active_document();
        if let Some(doc) = active_doc_props {
            let d_path = doc.path.to_string_lossy();
            let is_changelog = d_path.starts_with("Katana://ChangeLog");

            if !is_changelog {
                let doc_path = doc.path.clone();
                let ws_root = app.state.workspace.data.as_ref().map(|ws| ws.root.clone());
                let rel = relative_full_path(&doc_path, ws_root.as_deref());
                let breadcrumb_action = render_breadcrumbs(ui, app, &rel, ws_root.as_deref());
                if let Some(a) = breadcrumb_action {
                    app.pending_action = a;
                }
            }
            crate::views::top_bar::render_view_mode_bar(
                ui,
                &mut app.state,
                &mut app.pending_action,
            );
        }
    });
}

fn render_breadcrumbs(
    ui: &mut egui::Ui,
    app: &KatanaApp,
    rel: &str,
    ws_root: Option<&std::path::Path>,
) -> Option<AppAction> {
    let mut breadcrumb_action = None;
    ui.horizontal(|ui| {
        let segments: Vec<&str> = rel.split('/').collect();
        let mut current_path = ws_root.map(std::path::PathBuf::from).unwrap_or_default();
        for (i, seg) in segments.iter().enumerate() {
            if i > 0 {
                ui.add(
                    egui::Image::new(crate::Icon::ChevronRight.uri())
                        .tint(ui.visuals().text_color())
                        .max_height(CHEVRON_ICON_SIZE),
                );
            }

            if ws_root.is_none() {
                ui.label(egui::RichText::new(*seg).small());
                continue;
            }

            current_path = current_path.join(seg);
            let is_last = i == segments.len() - 1;

            if is_last {
                ui.add(
                    egui::Label::new(egui::RichText::new(*seg).small()).sense(egui::Sense::hover()),
                );
            } else {
                ui.menu_button(egui::RichText::new(*seg).small(), |ui| {
                    let mut ctx_action = crate::app_state::AppAction::None;

                    if let Some(ws) = &app.state.workspace.data {
                        if let Some(katana_core::workspace::TreeEntry::Directory {
                            children, ..
                        }) =
                            crate::views::panels::tree::find_node_in_tree(&ws.tree, &current_path)
                        {
                            crate::views::panels::workspace::render_breadcrumb_menu(
                                ui,
                                children,
                                &mut ctx_action,
                            );
                        }
                    }

                    if !matches!(ctx_action, crate::app_state::AppAction::None) {
                        breadcrumb_action = Some(ctx_action);
                        ui.close();
                    }
                });
            }
        }
    });
    breadcrumb_action
}

fn render_central_content(ctx: &egui::Context, app: &mut KatanaApp) -> Option<DownloadRequest> {
    let mut download_req: Option<DownloadRequest> = None;
    let current_mode = app.state.active_view_mode();
    let is_split = current_mode == ViewMode::Split;
    let mut is_changelog_tab = false;

    if let Some(doc) = app.state.active_document() {
        if doc.path.to_string_lossy().starts_with("Katana://ChangeLog") {
            is_changelog_tab = true;
        }
    }

    if app.state.layout.show_toc && app.state.config.settings.settings().layout.toc_visible {
        if let Some(doc) = app.state.active_document() {
            if let Some(preview) = app.tab_previews.iter_mut().find(|p| p.path == doc.path) {
                crate::views::panels::toc::render_toc_panel(ctx, &mut preview.pane, &app.state);
            }
        }
    }

    if is_changelog_tab {
        egui::CentralPanel::default().show(ctx, |ui| {
            crate::changelog::render_release_notes_tab(
                ui,
                &app.changelog_sections,
                app.changelog_rx.is_some(),
            );
        });
    } else {
        if is_split {
            let split_dir = app.state.active_split_direction();
            let pane_order = app.state.active_pane_order();
            download_req =
                crate::views::layout::split::render_split_mode(ctx, app, split_dir, pane_order);
        }

        if !is_split {
            egui::CentralPanel::default()
                .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
                .show(ctx, |ui| match current_mode {
                    ViewMode::CodeOnly => {
                        crate::views::panels::editor::render_editor_content(
                            ui,
                            &mut app.state,
                            &mut app.pending_action,
                            false,
                        );
                    }
                    ViewMode::PreviewOnly => {
                        crate::views::layout::split::render_preview_only(ui, app);
                    }
                    ViewMode::Split => {}
                });
        }
    }

    download_req
}

/// Intercepts URL opening requests from egui output commands.
///
/// External URLs (http/https/mailto) are passed through to the browser.
/// Internal file paths are resolved and dispatched as `SelectDocument` actions.
pub(crate) fn intercept_url_commands(ctx: &egui::Context, app: &mut KatanaApp) {
    let commands = ctx.output_mut(|o| std::mem::take(&mut o.commands));
    let mut unprocessed_commands = Vec::new();

    for cmd in commands {
        if let egui::OutputCommand::OpenUrl(open) = &cmd {
            let url = &open.url;
            if url.starts_with("http://")
                || url.starts_with("https://")
                || url.starts_with("mailto:")
            {
                unprocessed_commands.push(cmd);
            } else {
                let mut path = std::path::PathBuf::from(url);
                if path.is_relative() {
                    if let Some(doc) = app.state.active_document() {
                        if let Some(parent) = doc.path.parent() {
                            path = parent.join(path);
                        }
                    }
                }
                app.process_action(ctx, AppAction::SelectDocument(path));
            }
        } else {
            unprocessed_commands.push(cmd);
        }
    }

    if !unprocessed_commands.is_empty() {
        ctx.output_mut(|o| o.commands.extend(unprocessed_commands));
    }
}
