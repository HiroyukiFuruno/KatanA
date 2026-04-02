use crate::app_state::AppState;
use eframe::egui;

pub struct ProblemsPanel<'a> {
    state: &'a mut AppState,
    pending_action: &'a mut crate::app_state::AppAction,
}

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

    pub fn show(self, ctx: &egui::Context) {
        const SPACING: f32 = 4.0;

        if !self.state.diagnostics.is_panel_open {
            return;
        }

        egui::TopBottomPanel::bottom("problems_panel")
            .resizable(true)
            .min_height(100.0)
            .show(ctx, |ui| {
                ui.add_space(SPACING);
                ui.horizontal(|ui| {
                    ui.heading(crate::i18n::get().status.problems_panel_title.clone());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(crate::i18n::get().status.problems_panel_close.clone())
                            .clicked()
                        {
                            self.state.diagnostics.is_panel_open = false;
                        }
                    });
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let total = self.state.diagnostics.total_problems();
                    if total == 0 {
                        ui.label(
                            egui::RichText::new(
                                crate::i18n::get().status.no_problems_found.clone(),
                            )
                            .weak(),
                        );
                    } else {
                        for (path, diagnostics) in &self.state.diagnostics.problems {
                            let filename = path.file_name().unwrap_or_default().to_string_lossy();
                            ui.label(egui::RichText::new(filename).strong());
                            for diag in diagnostics {
                                ui.horizontal(|ui| {
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
                                    if ui.selectable_label(false, msg).clicked() {
                                        *self.pending_action =
                                            crate::app_state::AppAction::SelectDocumentAndJump {
                                                path: path.clone(),
                                                line: diag.range.start_line,
                                                byte_range: 0..0, // Exact byte range can be supported later
                                            };
                                    }
                                });
                            }
                            ui.add_space(SPACING);
                        }
                    }
                });
            });
    }
}
