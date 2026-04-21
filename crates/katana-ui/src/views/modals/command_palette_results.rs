use crate::app_state::AppAction;
use crate::state::command_palette::{
    CommandPaletteExecutePayload, CommandPaletteResultKind, CommandPaletteState,
};
use eframe::egui;

const COMMAND_PALETTE_MAX_HEIGHT: f32 = 400.0;
const COMMAND_PALETTE_MARGIN: f32 = 8.0;
const COMMAND_PALETTE_INNER_MARGIN_Y: f32 = 4.0;

pub(super) fn render_results(
    ui: &mut egui::Ui,
    state: &mut CommandPaletteState,
    action: &mut AppAction,
    is_open: &mut bool,
) {
    /* WHY: Capture the panel width BEFORE the ScrollArea.
    This is the authoritative width of the command palette window content area. */
    let panel_width = ui.available_width();

    egui::ScrollArea::vertical()
        .max_height(COMMAND_PALETTE_MAX_HEIGHT)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            if state.results.is_empty() {
                ui.label(
                    egui::RichText::new(&crate::i18n::I18nOps::get().search.palette_no_results)
                        .weak(),
                );
                return;
            }
            for (idx, result) in state.results.iter().enumerate() {
                let is_selected = idx == state.selected_index;
                let text_color = if is_selected {
                    ui.visuals().selection.stroke.color
                } else {
                    ui.visuals().text_color()
                };

                /* WHY: Use allocate_ui_with_layout to reserve the FULL panel width
                for each row. This is the proven pattern (see status_bar.rs) that
                ensures right_to_left layout has enough space to push content to
                the right edge. ui.horizontal() shrink-wraps and breaks right-align. */
                let row_height =
                    ui.spacing().interact_size.y + COMMAND_PALETTE_INNER_MARGIN_Y * 2.0;
                let response = ui
                    .allocate_ui_with_layout(
                        egui::vec2(panel_width, row_height),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.add_space(COMMAND_PALETTE_MARGIN);

                            let icon = match result.kind {
                                CommandPaletteResultKind::Action => crate::Icon::Action,
                                CommandPaletteResultKind::File => crate::Icon::Document,
                                CommandPaletteResultKind::MarkdownContent => crate::Icon::Markdown,
                                CommandPaletteResultKind::RecentOrCommon => crate::Icon::Recent,
                            };
                            ui.visuals_mut().override_text_color = Some(text_color);
                            let img = icon.ui_image(ui, crate::icon::IconSize::Medium);
                            ui.add(img);
                            ui.label(
                                egui::RichText::new(&result.label)
                                    .color(text_color)
                                    .strong(),
                            );
                            if let Some(sec) = &result.secondary_label {
                                ui.label(egui::RichText::new(sec).color(text_color).weak());
                            }

                            /* WHY: Switch to right_to_left layout for the remaining
                            space. This pushes shortcut badges to the right edge.
                            add_space(COMMAND_PALETTE_MARGIN) creates right padding. */
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.add_space(COMMAND_PALETTE_MARGIN);
                                    if let Some(shortcut) = &result.shortcut {
                                        crate::widgets::ShortcutWidget::new(shortcut).ui(ui);
                                    }
                                },
                            );
                        },
                    )
                    .response;

                let row_id = ui.id().with(&result.id);
                let interact = ui.interact(response.rect, row_id, egui::Sense::click());
                if interact.hovered() {
                    state.selected_index = idx;
                }
                if interact.clicked() {
                    execute_payload(action, &result.execute_payload);
                    *is_open = false;
                }
                if is_selected {
                    response.scroll_to_me(None);
                }
            }
        });
}

pub(super) fn execute_payload(action: &mut AppAction, payload: &CommandPaletteExecutePayload) {
    match payload {
        CommandPaletteExecutePayload::DispatchAppAction(a) => {
            *action = a.clone();
        }
        CommandPaletteExecutePayload::OpenFile(path) => {
            *action = AppAction::SelectDocument(path.clone());
        }
        CommandPaletteExecutePayload::NavigateToContent {
            path,
            line,
            byte_range,
        } => {
            *action = AppAction::SelectDocumentAndJump {
                path: path.clone(),
                line: *line,
                byte_range: byte_range.clone(),
            };
        }
    }
}
