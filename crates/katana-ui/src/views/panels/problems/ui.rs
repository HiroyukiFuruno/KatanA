use super::types::*;
use crate::app_state::AppState;
/* WHY: allow(file_length) */
/* WHY: allow(nesting_depth) */
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
                        let mut expand_action = None;
                        /* WHY: allow(horizontal_layout) */
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    crate::Icon::ExpandAll.button(ui, crate::icon::IconSize::Small),
                                )
                                .clicked()
                            {
                                expand_action = Some(true);
                            }
                            if ui
                                .add(
                                    crate::Icon::CollapseAll
                                        .button(ui, crate::icon::IconSize::Small),
                                )
                                .clicked()
                            {
                                expand_action = Some(false);
                            }

                            if expand_action.is_some() {
                                self.state.diagnostics.expand_all = expand_action;
                            }

                            const TITLE_BOTTOM_MARGIN: f32 = 8.0;
                            ui.add_space(TITLE_BOTTOM_MARGIN);

                            ui.heading(
                                crate::i18n::I18nOps::get()
                                    .status
                                    .problems_panel_title
                                    .clone(),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
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
                                    let batches =
                                        super::bulk_fixes::ProblemBulkFixOps::workspace_batches(
                                            &self.state.diagnostics.problems,
                                            &self.state.diagnostics,
                                        );
                                    if !batches.is_empty()
                                        && ui
                                            .button(
                                                crate::i18n::I18nOps::get()
                                                    .status
                                                    .fix_all_detected_problems
                                                    .clone(),
                                            )
                                            .clicked()
                                    {
                                        *self.pending_action =
                                            crate::app_state::AppAction::ApplyLintFixesForFiles(
                                                batches,
                                            );
                                    }
                                },
                            );
                        });
                    })
                    .show(ui);
                ui.separator();

                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    let total = self.state.diagnostics.total_problems();
                    if total == 0 {
                        ui.label(
                            egui::RichText::new(
                                crate::i18n::I18nOps::get().status.no_problems_found.clone(),
                            )
                            .weak(),
                        );
                    } else {
                        let expand_all = self.state.diagnostics.expand_all;
                        for (path, diagnostics) in &self.state.diagnostics.problems {
                            let open_content = self.state.document.open_documents.iter()
                                .find(|d| &d.path == path)
                                .map(|d| d.buffer.as_str());
                            let content = self
                                .state
                                .diagnostics
                                .content_snapshot(path)
                                .or(open_content);

                            if let Some(action) =
                                super::diagnostics_renderer::DiagnosticsRendererOps::show_file_diagnostics(ui, path, diagnostics, expand_all, content)
                            {
                                *self.pending_action = action;
                            }
                            ui.add_space(SPACING);
                        }
                        self.state.diagnostics.expand_all = None;
                    }
                });
            });
    }
}
