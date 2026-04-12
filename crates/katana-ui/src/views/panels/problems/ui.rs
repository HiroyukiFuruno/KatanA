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
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .content(|ui| {
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
                    })
                    .show(ui);
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
                            if let Some(action) = show_file_diagnostics(ui, path, diagnostics) {
                                *self.pending_action = action;
                            }
                            ui.add_space(SPACING);
                        }
                    }
                });
            });
    }
}

fn show_file_diagnostics(
    ui: &mut egui::Ui,
    path: &std::path::Path,
    diagnostics: &[katana_linter::markdown::MarkdownDiagnostic],
) -> Option<crate::app_state::AppAction> {
    let active_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.official_meta.is_some())
        .collect();

    if active_diags.is_empty() {
        return None;
    }

    let filename = path.file_name().unwrap_or_default().to_string_lossy();
    ui.label(egui::RichText::new(filename).strong());
    for diag in active_diags {
        if show_diagnostic_row(ui, diag) {
            return Some(crate::app_state::AppAction::SelectDocumentAndJump {
                path: path.to_path_buf(),
                line: diag.range.start_line,
                byte_range: 0..0,
            });
        }
    }
    None
}

fn show_diagnostic_row(
    ui: &mut egui::Ui,
    diag: &katana_linter::markdown::MarkdownDiagnostic,
) -> bool {
    let icon = match diag.severity {
        katana_linter::markdown::DiagnosticSeverity::Error => "🔴",
        katana_linter::markdown::DiagnosticSeverity::Warning => "🟡",
        katana_linter::markdown::DiagnosticSeverity::Info => "🔵",
    };

    let meta = diag.official_meta.as_ref().expect("hidden rules filtered");
    let is_experimental = meta.parity == katana_linter::markdown::RuleParityStatus::Experimental;

    let mut clicked = false;
    crate::widgets::AlignCenter::new()
        .shrink_to_fit(true)
        .content(|ui| {
            ui.label(icon);

            let rule_label = if is_experimental {
                format!("{} (Exp)", meta.code)
            } else {
                meta.code.to_string()
            };

            let location = format!("[{}:{}]", diag.range.start_line, diag.range.start_column);
            let msg = format!("{} {} {}", location, rule_label, diag.message);

            let mut button_text = egui::RichText::new(msg);
            if is_experimental {
                button_text = button_text.weak();
            }

            /* WHY: scroll list item; jump triggered on click */
            if ui
                .add(egui::Button::selectable(false, button_text).frame_when_inactive(true))
                .clicked()
            {
                clicked = true;
            }

            if !meta.docs_url.is_empty() {
                ui.hyperlink_to(
                    crate::i18n::I18nOps::get().about.documentation.as_str(),
                    meta.docs_url,
                );
            }
        })
        .show(ui);
    clicked
}
