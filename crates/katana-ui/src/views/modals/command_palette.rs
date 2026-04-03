use crate::app_state::AppAction;
use crate::state::command_palette::{
    CommandPaletteExecutePayload, CommandPaletteProvider, CommandPaletteResult,
    CommandPaletteResultKind, CommandPaletteState,
};
use eframe::egui;

const COMMAND_PALETTE_WIDTH: f32 = 600.0;
const COMMAND_PALETTE_DEFAULT_HEIGHT: f32 = 600.0;
const COMMAND_PALETTE_MAX_HEIGHT: f32 = 1200.0;
const COMMAND_PALETTE_MARGIN: f32 = 8.0;
const COMMAND_PALETTE_INNER_MARGIN_Y: f32 = 4.0;

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

        egui::Window::new(crate::i18n::get().menu.command_palette.clone())
            .title_bar(false)
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_TOP, [0.0, 100.0])
            .default_size([COMMAND_PALETTE_WIDTH, COMMAND_PALETTE_DEFAULT_HEIGHT])
            .show(ctx, |ui| {
                ui.set_max_width(COMMAND_PALETTE_WIDTH);

                let text_edit = egui::TextEdit::singleline(&mut self.state.current_query)
                    .hint_text(crate::i18n::get().search.palette_query_hint.clone())
                    .desired_width(f32::INFINITY)
                    .margin(egui::vec2(COMMAND_PALETTE_MARGIN, COMMAND_PALETTE_MARGIN));

                let response = ui.add(text_edit);

                if response.changed() || self.state.results.is_empty() {
                    // Gather results from all providers
                    let mut gathered = Vec::new();
                    for provider in self.providers {
                        gathered.extend(provider.search(&self.state.current_query, self.workspace));
                    }

                    // Sort results
                    gathered.sort_by(|a, b| {
                        b.score
                            .partial_cmp(&a.score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                    self.state.update_results(gathered);
                }

                // Keyboard interactions
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    is_open = false;
                } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    self.state.move_down();
                } else if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    self.state.move_up();
                } else if ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && let Some(res) = self.state.results.get(self.state.selected_index)
                {
                    match &res.execute_payload {
                        CommandPaletteExecutePayload::DispatchAppAction(a) => {
                            *self.action = a.clone();
                        }
                        CommandPaletteExecutePayload::OpenFile(path) => {
                            *self.action = AppAction::SelectDocument(path.clone());
                        }
                        CommandPaletteExecutePayload::NavigateToContent {
                            path,
                            line,
                            byte_range,
                        } => {
                            *self.action = AppAction::SelectDocumentAndJump {
                                path: path.clone(),
                                line: *line,
                                byte_range: byte_range.clone(),
                            };
                        }
                    }
                    is_open = false; // dismiss after action
                }

                // If just opened, request focus
                if response.gained_focus() || !response.has_focus() {
                    response.request_focus();
                }

                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(COMMAND_PALETTE_MAX_HEIGHT)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        if self.state.results.is_empty() {
                            ui.label(
                                egui::RichText::new(&crate::i18n::get().search.palette_no_results)
                                    .weak(),
                            );
                        } else {
                            for (idx, result) in self.state.results.iter().enumerate() {
                                let is_selected = idx == self.state.selected_index;

                                let bg_color = if is_selected {
                                    ui.visuals().selection.bg_fill
                                } else {
                                    ui.ctx().data(|d| {
                                        d.get_temp::<katana_platform::theme::ThemeColors>(
                                            egui::Id::new("katana_theme_colors"),
                                        )
                                        .map(|tc| {
                                            crate::theme_bridge::rgb_to_color32(
                                                tc.system.panel_background,
                                            )
                                        })
                                        .unwrap_or_else(|| ui.visuals().window_fill())
                                    })
                                };

                                let text_color = if is_selected {
                                    ui.visuals().selection.stroke.color
                                } else {
                                    ui.visuals().text_color()
                                };

                                let frame =
                                    egui::Frame::NONE.fill(bg_color).inner_margin(egui::vec2(
                                        COMMAND_PALETTE_MARGIN,
                                        COMMAND_PALETTE_INNER_MARGIN_Y,
                                    ));

                                let response = frame
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            let icon = match result.kind {
                                                CommandPaletteResultKind::Action => {
                                                    crate::Icon::Action
                                                }
                                                CommandPaletteResultKind::File => {
                                                    crate::Icon::Document
                                                }
                                                CommandPaletteResultKind::MarkdownContent => {
                                                    crate::Icon::Markdown
                                                }
                                                CommandPaletteResultKind::RecentOrCommon => {
                                                    crate::Icon::Recent
                                                }
                                            };
                                            ui.add(
                                                icon.image(crate::icon::IconSize::Medium)
                                                    .tint(text_color),
                                            );
                                            ui.label(
                                                egui::RichText::new(&result.label)
                                                    .color(text_color)
                                                    .strong(),
                                            );
                                            if let Some(sec) = &result.secondary_label {
                                                ui.label(
                                                    egui::RichText::new(sec)
                                                        .color(text_color)
                                                        .weak(),
                                                );
                                            }
                                        });
                                    })
                                    .response;

                                // Hover or click interaction
                                let interact =
                                    ui.interact(response.rect, response.id, egui::Sense::click());
                                if interact.hovered() {
                                    self.state.selected_index = idx;
                                }
                                if interact.clicked() {
                                    // Execute action on click as well
                                    match &result.execute_payload {
                                        CommandPaletteExecutePayload::DispatchAppAction(a) => {
                                            *self.action = a.clone();
                                        }
                                        CommandPaletteExecutePayload::OpenFile(path) => {
                                            *self.action = AppAction::SelectDocument(path.clone());
                                        }
                                        CommandPaletteExecutePayload::NavigateToContent {
                                            path,
                                            line,
                                            byte_range,
                                        } => {
                                            *self.action = AppAction::SelectDocumentAndJump {
                                                path: path.clone(),
                                                line: *line,
                                                byte_range: byte_range.clone(),
                                            };
                                        }
                                    }
                                    is_open = false;
                                }

                                if is_selected {
                                    response.scroll_to_me(None);
                                }
                            }
                        }
                    });
            });

        self.state.is_open = is_open;
    }
}
