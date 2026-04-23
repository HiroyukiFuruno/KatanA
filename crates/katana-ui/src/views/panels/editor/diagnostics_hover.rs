use eframe::egui;

pub(crate) struct DiagnosticsHoverOps;

impl DiagnosticsHoverOps {
    pub(crate) fn draw_wave(
        ui: &mut egui::Ui,
        min_x: f32,
        max_x: f32,
        y_mid: f32,
        color: egui::Color32,
    ) {
        let (mut points, mut x, mut up) = (vec![], min_x, true);
        const AMPLITUDE: f32 = 1.5;
        const PERIOD: f32 = 4.0;

        while x < max_x {
            points.push(egui::pos2(
                x,
                y_mid + if up { -AMPLITUDE } else { AMPLITUDE },
            ));
            x += PERIOD;
            up = !up;
        }
        if points.last().is_some_and(|l| l.x < max_x) {
            points.push(egui::pos2(
                max_x,
                y_mid + if up { -AMPLITUDE } else { AMPLITUDE },
            ));
        }
        ui.painter()
            .add(egui::Shape::Path(egui::epaint::PathShape::line(
                points,
                egui::Stroke::new(1.0, color),
            )));
    }

    pub(crate) fn show_hover_ui(
        ui: &mut egui::Ui,
        diag: &katana_linter::rules::markdown::MarkdownDiagnostic,
        meta: &katana_linter::rules::markdown::OfficialRuleMeta,
        all_diagnostics: &[katana_linter::rules::markdown::MarkdownDiagnostic],
        action: &mut crate::app_state::AppAction,
    ) {
        /* WHY: Collect all diagnostics on the same line (including the hovered one) so
         * we can render a complete list instead of just the first-hit diagnostic.
         * The hovered `diag` is always rendered first (primary), followed by any
         * additional same-line diagnostics as secondary sections. */
        let same_line_others: Vec<_> = all_diagnostics
            .iter()
            .filter(|d| {
                d.range.start_line == diag.range.start_line
                    && d.rule_id != diag.rule_id
                    && d.official_meta.is_some()
            })
            .collect();

        /* WHY: Primary diagnostic section */
        Self::render_single_diag_section(ui, diag, meta, all_diagnostics, action);

        /* WHY: Additional same-line diagnostics — each gets its own separator + section */
        for other in same_line_others {
            if let Some(other_meta) = other.official_meta.as_ref() {
                ui.separator();
                Self::render_single_diag_section(ui, other, other_meta, all_diagnostics, action);
            }
        }
    }

    fn render_single_diag_section(
        ui: &mut egui::Ui,
        diag: &katana_linter::rules::markdown::MarkdownDiagnostic,
        meta: &katana_linter::rules::markdown::OfficialRuleMeta,
        all_diagnostics: &[katana_linter::rules::markdown::MarkdownDiagnostic],
        action: &mut crate::app_state::AppAction,
    ) {
        let sev_text = format!("{:?}", diag.severity);
        let label_text = format!("[{}] {} ({})", sev_text, diag.rule_id, meta.title);
        ui.label(egui::RichText::new(label_text).strong());
        let localized_msg = crate::i18n::I18nOps::get()
            .linter
            .rule_descriptions
            .get(&diag.rule_id.to_lowercase())
            .cloned()
            .unwrap_or_else(|| diag.message.clone());
        ui.label(localized_msg);

        if meta.is_fixable && diag.fix_info.is_some() {
            /* WHY: allow(horizontal_layout) */
            ui.horizontal(|ui| {
                let linter_msgs = &crate::i18n::I18nOps::get().linter;
                if ui.button(&linter_msgs.fix).clicked() {
                    *action = crate::app_state::AppAction::ApplyLintFixes(vec![
                        diag.fix_info.clone().unwrap(),
                    ]);
                    ui.close_menu();
                }
                if ui.button(&linter_msgs.fix_all).clicked() {
                    let all_fixes = all_diagnostics
                        .iter()
                        .filter_map(|d| d.fix_info.clone())
                        .collect();
                    *action = crate::app_state::AppAction::ApplyLintFixes(all_fixes);
                    ui.close_menu();
                }
            });
        }
        if ui
            .link(format!(
                "{} - {}",
                &crate::i18n::I18nOps::get().linter.docs,
                meta.code
            ))
            .clicked()
        {
            *action = crate::app_state::AppAction::OpenLinterDoc(
                meta.code.to_string(),
                meta.docs_url.to_string(),
            );
            ui.close_menu();
        }
    }
}
