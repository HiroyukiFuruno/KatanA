use eframe::egui;

pub(crate) struct DiagnosticsRendererOps;

impl DiagnosticsRendererOps {
    /* WHY: allow(horizontal_layout) */
    /* WHY: allow(nesting_depth) */
    pub(crate) fn show_file_diagnostics(
        ui: &mut egui::Ui,
        path: &std::path::Path,
        diagnostics: &[katana_linter::rules::markdown::MarkdownDiagnostic],
        expand_all: Option<bool>,
    ) -> Option<crate::app_state::AppAction> {
        let active_diags: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.official_meta.is_some())
            .collect();

        if active_diags.is_empty() {
            return None;
        }

        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        let id = ui.make_persistent_id(path);
        let mut state =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);
        if let Some(expand) = expand_all {
            state.set_open(expand);
        }

        let mut action = None;
        state
            .show_header(ui, |ui| {
                ui.label(egui::RichText::new(filename).strong());
            })
            .body(|ui| {
                const GRID_COLS: usize = 3;
                const GRID_SPACE_X: f32 = 16.0;
                const GRID_SPACE_Y: f32 = 4.0;
                egui::Grid::new(id.with("grid"))
                    .num_columns(GRID_COLS)
                    .spacing(egui::vec2(GRID_SPACE_X, GRID_SPACE_Y))
                    .show(ui, |ui| {
                        for diag in active_diags {
                            if let Some(act) = Self::show_diagnostic_row(ui, diag, path) {
                                action = Some(act);
                            }
                        }
                    });
            });

        action
    }

    /* WHY: allow(horizontal_layout) */
    pub(crate) fn show_diagnostic_row(
        ui: &mut egui::Ui,
        diag: &katana_linter::rules::markdown::MarkdownDiagnostic,
        path: &std::path::Path,
    ) -> Option<crate::app_state::AppAction> {
        /* WHY: Severity icons use #FFFFFF base SVG from system/ and are tinted here
         * with the semantic color sourced from ThemeColors (system.error_text etc.),
         * so the hue follows the active theme rather than hardcoded values. */
        let tc = ui.data(|d| {
            d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new("katana_theme_colors"))
        });
        let (sev_icon, sev_tint) = match diag.severity {
            katana_linter::rules::markdown::DiagnosticSeverity::Error => (
                crate::icon::Icon::CircleFilled,
                tc.as_ref().map(|c| {
                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(c.system.error_text)
                }),
            ),
            katana_linter::rules::markdown::DiagnosticSeverity::Warning => (
                crate::icon::Icon::CircleFilled,
                tc.as_ref().map(|c| {
                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(c.system.warning_text)
                }),
            ),
            katana_linter::rules::markdown::DiagnosticSeverity::Info => (
                crate::icon::Icon::CircleFilled,
                tc.as_ref().map(|c| {
                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(c.system.success_text)
                }),
            ),
        };

        let meta = diag.official_meta.as_ref().expect("hidden rules filtered");
        let is_experimental =
            meta.parity == katana_linter::rules::markdown::RuleParityStatus::Experimental;

        let mut action = None;

        let rule_label = if is_experimental {
            format!("{} (Exp)", meta.code)
        } else {
            meta.code.to_string()
        };

        let location = format!("[{}:{}]", diag.range.start_line, diag.range.start_column);
        let msg = format!("{} {}", location, rule_label);

        let mut button_text = egui::RichText::new(msg);
        if is_experimental {
            button_text = button_text.weak();
        }

        /* WHY: allow(horizontal_layout) */
        ui.horizontal(|ui| {
            let img = sev_icon.image(crate::icon::IconSize::Small);
            let img = if let Some(tint) = sev_tint {
                img.tint(tint)
            } else {
                img
            };
            ui.add(img);
            /* WHY: scroll list item; jump triggered on click */
            if egui::Widget::ui(
                egui::Button::selectable(false, button_text).frame_when_inactive(true),
                ui,
            )
            .clicked()
            {
                action = Some(crate::app_state::AppAction::SelectDocumentAndJump {
                    path: path.to_path_buf(),
                    line: diag.range.start_line,
                    byte_range: 0..0,
                });
            }
        });

        let localized_msg = crate::i18n::I18nOps::get()
            .linter
            .rule_descriptions
            .get(&diag.rule_id.to_lowercase())
            .cloned()
            .unwrap_or_else(|| diag.message.clone());
        ui.label(egui::RichText::new(localized_msg));

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 0.0),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                if !meta.docs_url.is_empty() {
                    let docs_text = &crate::i18n::I18nOps::get().linter.docs;
                    if ui.link(docs_text).clicked() {
                        action = Some(crate::app_state::AppAction::OpenLinterDoc(
                            meta.code.to_string(),
                            meta.docs_url.to_string(),
                        ));
                    }
                }

                if let Some(fix_info) = &diag.fix_info
                    && ui.button(&crate::i18n::I18nOps::get().linter.fix).clicked()
                {
                    action = Some(crate::app_state::AppAction::ApplyLintFixes(vec![
                        fix_info.clone(),
                    ]));
                }
            },
        );

        ui.end_row();

        action
    }
}
