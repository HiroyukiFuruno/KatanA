use eframe::egui;

pub(crate) struct RowDiagnosticsRenderer;

impl RowDiagnosticsRenderer {
    pub(crate) fn render(
        ui: &mut egui::Ui,
        diagnostics: &[katana_markdown_linter::rules::markdown::MarkdownDiagnostic],
        p: usize,
        y: f32,
        ln_rect: &egui::Rect,
        row_height: f32,
        action: &mut crate::app_state::AppAction,
    ) {
        let line_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|d| {
                d.official_meta.is_some()
                    && d.range.start_line <= (p + 1)
                    && d.range.end_line >= (p + 1)
            })
            .collect();

        if line_diagnostics.is_empty() {
            return;
        }

        let has_fixable = line_diagnostics.iter().any(|d| d.fix_info.is_some());
        if !has_fixable {
            return;
        }

        const WEIGHT_ERROR: u8 = 3;
        const WEIGHT_WARNING: u8 = 2;
        const WEIGHT_INFO: u8 = 1;

        let icon_color = line_diagnostics
            .iter()
            .map(|d| d.severity)
            .max_by_key(|s| match s {
                katana_markdown_linter::rules::markdown::DiagnosticSeverity::Error => WEIGHT_ERROR,
                katana_markdown_linter::rules::markdown::DiagnosticSeverity::Warning => {
                    WEIGHT_WARNING
                }
                katana_markdown_linter::rules::markdown::DiagnosticSeverity::Info => WEIGHT_INFO,
            })
            .map(|s| match s {
                katana_markdown_linter::rules::markdown::DiagnosticSeverity::Error => {
                    ui.visuals().error_fg_color
                }
                katana_markdown_linter::rules::markdown::DiagnosticSeverity::Warning => {
                    ui.visuals().warn_fg_color
                }
                katana_markdown_linter::rules::markdown::DiagnosticSeverity::Info => {
                    ui.visuals().text_color()
                }
            })
            .unwrap_or(ui.visuals().warn_fg_color);

        const ICON_SIZE: f32 = 14.0;
        const ICON_MARGIN: f32 = 2.0;
        const TOOLTIP_SPACE: f32 = 4.0;

        let icon_rect = egui::Rect::from_min_size(
            egui::pos2(
                ln_rect.min.x + ICON_MARGIN,
                y + (row_height - ICON_SIZE) / 2.0,
            ),
            egui::vec2(ICON_SIZE, ICON_SIZE),
        );

        let icon_resp = ui.put(
            icon_rect,
            crate::icon::Icon::LightBulb
                .image(crate::icon::IconSize::Small)
                .tint(icon_color)
                .sense(egui::Sense::click()),
        );

        icon_resp.on_hover_ui(|ui| {
            for d in &line_diagnostics {
                let meta = d.official_meta.as_ref().unwrap();
                let sev_text = format!("{:?}", d.severity);
                let fmt_str = format!("[{}] {} ({})", sev_text, d.rule_id, meta.title);
                ui.label(egui::RichText::new(fmt_str).strong());
                ui.label(&d.message);

                if crate::linter_bridge::MarkdownLinterBridgeOps::has_applicable_fix(d) {
                    /* WHY: allow(horizontal_layout) - Standard egui pattern for side-by-side buttons */
                    ui.horizontal(|ui| {
                        let linter_msgs = &crate::i18n::I18nOps::get().linter;
                        if ui.button(&linter_msgs.fix).clicked() {
                            let fixes = vec![d.fix_info.clone().unwrap()];
                            *action = crate::app_state::AppAction::ApplyLintFixes(fixes);
                            ui.close_menu();
                        }
                        if ui.button(&linter_msgs.fix_all).clicked() {
                            let all_fixes = diagnostics
                                .iter()
                                .filter_map(|d| d.fix_info.clone())
                                .collect();
                            *action = crate::app_state::AppAction::ApplyLintFixes(all_fixes);
                            ui.close_menu();
                        }
                    });
                }
                let docs_text = &crate::i18n::I18nOps::get().linter.docs;
                if ui.link(format!("{} - {}", docs_text, meta.code)).clicked() {
                    *action = crate::app_state::AppAction::OpenLinterDoc(
                        meta.code.to_string(),
                        meta.docs_url.to_string(),
                    );
                    ui.close_menu();
                }
                ui.add_space(TOOLTIP_SPACE);
            }
        });
    }
}
