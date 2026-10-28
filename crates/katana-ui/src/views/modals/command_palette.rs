use super::command_palette_results::{execute_payload, render_results};
use crate::app_state::AppAction;
use crate::state::command_palette::{
    CommandPaletteExecutePayload, CommandPaletteProvider, CommandPaletteResult, CommandPaletteState,
};
use eframe::egui;

use super::command_palette_keyboard::handle_key_input;

const COMMAND_PALETTE_WIDTH: f32 = 600.0;
const COMMAND_PALETTE_DEFAULT_HEIGHT: f32 = 600.0;
const COMMAND_PALETTE_MAX_HEIGHT: f32 = 1200.0;
const COMMAND_PALETTE_MARGIN: f32 = 8.0;
const COMMAND_PALETTE_INNER_MARGIN_Y: f32 = 4.0;
const COMMAND_PALETTE_HINT_OPACITY: f32 = 0.4;
const COMMAND_PALETTE_PREFIX_OPACITY: f32 = 0.0;

fn command_result_visible_in_normal_palette(result: &CommandPaletteResult) -> bool {
    matches!(
        &result.execute_payload,
        CommandPaletteExecutePayload::DispatchAppAction(
            AppAction::IngestImageFile | AppAction::IngestClipboardImage
        )
    )
}

pub(crate) struct CommandPaletteModal<'a> {
    pub state: &'a mut CommandPaletteState,
    pub workspace: Option<&'a katana_core::workspace::Workspace>,
    pub action: &'a mut AppAction,
    pub providers: &'a [Box<dyn CommandPaletteProvider>],
    operation_context: Option<(u64, crate::state::document::ViewMode)>,
}

impl<'a> CommandPaletteModal<'a> {
    pub fn new(
        state: &'a mut CommandPaletteState,
        workspace: Option<&'a katana_core::workspace::Workspace>,
        action: &'a mut AppAction,
        providers: &'a [Box<dyn CommandPaletteProvider>],
    ) -> Self {
        Self {
            state,
            workspace,
            action,
            providers,
            operation_context: None,
        }
    }

    pub(crate) fn with_operation_context(
        mut self,
        context: (u64, crate::state::document::ViewMode),
    ) -> Self {
        self.operation_context = Some(context);
        self
    }

    pub fn show(self, ctx: &egui::Context) {
        if !self.state.is_open {
            return;
        }

        let mut is_open = self.state.is_open;

        egui::Window::new(crate::i18n::I18nOps::get().menu.command_palette.clone())
            .title_bar(false)
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_TOP, [0.0, 100.0])
            .default_size([COMMAND_PALETTE_WIDTH, COMMAND_PALETTE_DEFAULT_HEIGHT])
            .show(ctx, |ui| {
                ui.set_max_width(COMMAND_PALETTE_WIDTH);

                let text_edit = egui::TextEdit::singleline(&mut self.state.current_query)
                    .desired_width(f32::INFINITY)
                    .margin(egui::vec2(COMMAND_PALETTE_MARGIN, COMMAND_PALETTE_MARGIN));

                let mut output = text_edit.show(ui);
                let response = output.response;

                if !response.has_focus() {
                    response.request_focus();
                }

                if self.state.request_cursor_eof && response.has_focus() {
                    let cursor = egui::text::CCursor::new(self.state.current_query.chars().count());
                    output.state.cursor.set_char_range(Some(egui::text::CCursorRange::one(cursor)));
                    output.state.store(ui.ctx(), response.id);
                    self.state.request_cursor_eof = false;
                }

                /* WHY: Draw i18n placeholders manually so they can appear even if '> ' is prefilled */
                let is_empty = self.state.current_query.is_empty();
                let is_action_mode_empty = self.state.current_query == "> " || self.state.current_query == ">";

                if is_empty || is_action_mode_empty {
                    let hint_text = if is_action_mode_empty {
                        crate::i18n::I18nOps::get().search.palette_action_query_hint.clone()
                    } else {
                        crate::i18n::I18nOps::get().search.palette_query_hint.clone()
                    };

                    let font_id = egui::TextStyle::Body.resolve(ui.style());
                    let mut pos = response.rect.left_center();
                    pos.x += COMMAND_PALETTE_MARGIN; // internal margin of text_edit

                    if is_action_mode_empty {
                        /* WHY: Offset by the width of the prefilled '> ' text */
                        let prefix_galley = ui.painter().layout_no_wrap(
                            self.state.current_query.clone(),
                            font_id.clone(),
                            ui.visuals().window_fill().gamma_multiply(COMMAND_PALETTE_PREFIX_OPACITY),
                        );
                        pos.x += prefix_galley.size().x;
                    }

                    let hint_color = ui.visuals().text_color().gamma_multiply(COMMAND_PALETTE_HINT_OPACITY);
                    let galley = ui.painter().layout_no_wrap(hint_text, font_id, hint_color);
                    ui.painter().galley(pos - egui::vec2(0.0, galley.size().y / 2.0), galley, ui.visuals().text_color());
                }

                let operation_context_changed = self.operation_context.is_some_and(|context| {
                    self.state.last_operation_context != Some(context)
                });
                if response.changed() || self.state.results.is_empty() || operation_context_changed {
                    /* WHY: Gather results from providers based on the query prefix.
                    If the query starts with '>', it only searches Katana commands.
                    Otherwise, it excludes Katana commands. */
                    let mut gathered = Vec::new();
                    let is_action_mode = self.state.current_query.starts_with('>');
                    let actual_query = if is_action_mode {
                        self.state.current_query[1..].trim_start().to_string()
                    } else {
                        self.state.current_query.clone()
                    };

                    for provider in self.providers {
                        if is_action_mode && provider.name() != "Commands" {
                            continue;
                        }
                        let results = provider.search(&actual_query, self.workspace, None);
                        if !is_action_mode && provider.name() == "Commands" {
                            gathered.extend(
                                results
                                    .into_iter()
                                    .filter(command_result_visible_in_normal_palette),
                            );
                        } else {
                            gathered.extend(results);
                        }
                    }
                    gathered.sort_by(|a, b| {
                        b.score
                            .partial_cmp(&a.score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    self.state.update_results(gathered);
                    self.state.last_operation_context = self.operation_context;
                }

                /* WHY: Keyboard interactions */
                let consumed_by_keyboard =
                    handle_key_input(ui, self.state, self.action, &mut is_open);

                ui.separator();
                render_results(
                    ui,
                    self.state,
                    self.action,
                    &mut is_open,
                    consumed_by_keyboard,
                );
            });

        self.state.is_open = is_open;
    }
}

#[cfg(test)]
include!("command_palette_tests.rs");
