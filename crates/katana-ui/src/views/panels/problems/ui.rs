use super::types::*;
use crate::app_state::AppState;
use eframe::egui;

impl<'a> ProblemsPanel<'a> {
    pub fn new(
        state: &'a mut AppState,
        pending_action: &'a mut crate::app_state::AppAction,
    ) -> Self {
        Self {
            state,
            pending_action,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        const SPACING: f32 = 4.0;

        if !self.state.diagnostics.is_panel_open {
            return;
        }

        egui::Panel::bottom("problems_panel")
            .resizable(true)
            .min_size(100.0)
            .show_inside(ui, |ui| {
                ui.add_space(SPACING);
                // WHY: allow(horizontal_layout)
                crate::widgets::AlignCenter::new().shrink_to_fit(true).content(|ui| {
                    ui.heading(
                        crate::i18n::I18nOps::get()
                            .status
                            .problems_panel_title
                            .clone(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(
                                crate::i18n::I18nOps::get()
                                    .status
                                    .problems_panel_close
                                    .clone(),
                            )
                            .clicked()
                        {
                            self.state.diagnostics.is_panel_open = false;
                        }
                    });
                }).show(ui);
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let total = self.state.diagnostics.total_problems();
                    if total == 0 {
                        ui.label(
                            egui::RichText::new(
                                crate::i18n::I18nOps::get().status.no_problems_found.clone(),
                            )
                            .weak(),
                        );
                    } else {
                        for (path, diagnostics) in &self.state.diagnostics.problems {
                            let filename = path.file_name().unwrap_or_default().to_string_lossy();
                            ui.label(egui::RichText::new(filename).strong());
                            for diag in diagnostics {
                                // WHY: allow(horizontal_layout)
                                crate::widgets::AlignCenter::new().shrink_to_fit(true).content(|ui| {
                                    let icon = match diag.severity {
                                        katana_linter::markdown::DiagnosticSeverity::Error => "🔴",
                                        katana_linter::markdown::DiagnosticSeverity::Warning => {
                                            "🟡"
                                        }
                                        katana_linter::markdown::DiagnosticSeverity::Info => "🔵",
                                    };
                                    ui.label(icon);
                                    let msg = format!(
                                        "[{}:{}] {}",
                                        diag.range.start_line,
                                        diag.range.start_column,
                                        diag.message
                                    );
                                    // WHY: scroll list item; future: ClickableRowOps atom
                                    if ui.add(egui::Button::selectable(false, msg).frame_when_inactive(true)).clicked() {
                                        *self.pending_action =
                                            crate::app_state::AppAction::SelectDocumentAndJump {
                                                path: path.clone(),
                                                line: diag.range.start_line,
                                                byte_range: 0..0,
                                            };
                                    }
                                }).show(ui);
                            }
                            ui.add_space(SPACING);
                        }
                    }
                });
            });
    }
}
