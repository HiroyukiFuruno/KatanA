use crate::app_state::AppAction;
use crate::state::command_palette::{
    CommandPaletteExecutePayload, CommandPaletteResultKind, CommandPaletteState,
};
use eframe::egui;

const COMMAND_PALETTE_MAX_HEIGHT: f32 = 1200.0;
const COMMAND_PALETTE_MARGIN: f32 = 8.0;
const COMMAND_PALETTE_INNER_MARGIN_Y: f32 = 4.0;

pub(super) fn render_results(
    ui: &mut egui::Ui,
    state: &mut CommandPaletteState,
    action: &mut AppAction,
    is_open: &mut bool,
) {
    egui::ScrollArea::vertical()
        .max_height(COMMAND_PALETTE_MAX_HEIGHT)
        .auto_shrink([false; 2])
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
                let frame = egui::Frame::NONE.inner_margin(egui::vec2(
                    COMMAND_PALETTE_MARGIN,
                    COMMAND_PALETTE_INNER_MARGIN_Y,
                ));
                let response = frame
                    .show(ui, |ui| {
                        crate::widgets::AlignCenter::new()
                            .shrink_to_fit(true)
                            .content(|ui| {
                                let icon = match result.kind {
                                    CommandPaletteResultKind::Action => crate::Icon::Action,
                                    CommandPaletteResultKind::File => crate::Icon::Document,
                                    CommandPaletteResultKind::MarkdownContent => {
                                        crate::Icon::Markdown
                                    }
                                    CommandPaletteResultKind::RecentOrCommon => crate::Icon::Recent,
                                };
                                ui.add(icon.image(crate::icon::IconSize::Medium).tint(text_color));
                                ui.label(
                                    egui::RichText::new(&result.label)
                                        .color(text_color)
                                        .strong(),
                                );
                                if let Some(sec) = &result.secondary_label {
                                    ui.label(egui::RichText::new(sec).color(text_color).weak());
                                }
                            })
                            .show(ui);
                    })
                    .response;

                let interact = ui.interact(response.rect, response.id, egui::Sense::click());
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
