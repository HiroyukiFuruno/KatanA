use super::command_palette_results::{execute_payload, render_results};
use crate::app_state::AppAction;
use crate::state::command_palette::{
    CommandPaletteExecutePayload, CommandPaletteProvider, CommandPaletteResult, CommandPaletteState,
};
use eframe::egui;

fn handle_key_input(
    ui: &mut egui::Ui,
    state: &mut crate::state::command_palette::CommandPaletteState,
    action: &mut AppAction,
    is_open: &mut bool,
) {
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        *is_open = false;
        return;
    }
    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
        state.move_down();
        return;
    }
    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
        state.move_up();
        return;
    }
    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        let res = state.results.get(state.selected_index).cloned();
        if let Some(res) = res {
            execute_payload(action, &res.execute_payload);
            *is_open = false;
        }
    }
}

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
        }
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

                if response.changed() || self.state.results.is_empty() {
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
                }

                /* WHY: Keyboard interactions */
                handle_key_input(ui, self.state, self.action, &mut is_open);

                ui.separator();
                render_results(ui, self.state, self.action, &mut is_open);
            });

        self.state.is_open = is_open;
    }
}

#[cfg(test)]
include!("command_palette_tests.rs");
