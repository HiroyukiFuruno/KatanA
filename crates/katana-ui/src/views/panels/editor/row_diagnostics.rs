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
        let line_number = p + 1;
        let line_diagnostics = Self::action_icon_diagnostics(diagnostics, line_number);

        if line_diagnostics.is_empty() {
            return;
        }

        if !Self::has_applicable_fix(&line_diagnostics) {
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
            for (index, diagnostic) in line_diagnostics.iter().enumerate() {
                if index > 0 {
                    ui.separator();
                }
                if let Some(meta) = diagnostic.official_meta.as_ref() {
                    super::diagnostics_hover::DiagnosticsHoverOps::show_single_diagnostic_ui(
                        ui,
                        diagnostic,
                        meta,
                        diagnostics,
                        action,
                    );
                }
                ui.add_space(TOOLTIP_SPACE);
            }
        });
    }

    fn action_icon_diagnostics(
        diagnostics: &[katana_markdown_linter::rules::markdown::MarkdownDiagnostic],
        line_number: usize,
    ) -> Vec<&katana_markdown_linter::rules::markdown::MarkdownDiagnostic> {
        diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.official_meta.is_some() && diagnostic.range.start_line == line_number
            })
            .collect()
    }

    fn has_applicable_fix(
        diagnostics: &[&katana_markdown_linter::rules::markdown::MarkdownDiagnostic],
    ) -> bool {
        diagnostics.iter().any(|diagnostic| {
            crate::linter_bridge::MarkdownLinterBridgeOps::has_applicable_fix(diagnostic)
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use katana_markdown_linter::rules::markdown::{
        DiagnosticFix, DiagnosticRange, DiagnosticSeverity, MarkdownDiagnostic, OfficialRuleMeta,
    };
    use std::path::PathBuf;

    fn official_meta() -> OfficialRuleMeta {
        katana_markdown_linter::rules::markdown::eval::MarkdownLinterOps::get_user_configurable_rules()
            .into_iter()
            .find(|rule| rule.id() == "MD001")
            .and_then(|rule| rule.official_meta())
            .unwrap()
    }

    fn diagnostic(start_line: usize, end_line: usize) -> MarkdownDiagnostic {
        let meta = official_meta();
        MarkdownDiagnostic {
            file: PathBuf::from("doc.md"),
            severity: DiagnosticSeverity::Warning,
            range: DiagnosticRange {
                start_line,
                start_column: 1,
                end_line,
                end_column: 10,
            },
            message: "message".to_string(),
            rule_id: meta.code.to_string(),
            official_meta: Some(meta),
            fix_info: Some(DiagnosticFix {
                start_line,
                start_column: 1,
                end_line,
                end_column: 10,
                replacement: "fixed".to_string(),
            }),
        }
    }

    #[test]
    fn action_icon_diagnostics_include_start_line_only_for_multiline_diagnostic() {
        let diagnostics = vec![diagnostic(2, 4)];

        assert_eq!(
            RowDiagnosticsRenderer::action_icon_diagnostics(&diagnostics, 2).len(),
            1
        );
        assert!(RowDiagnosticsRenderer::action_icon_diagnostics(&diagnostics, 3).is_empty());
        assert!(RowDiagnosticsRenderer::action_icon_diagnostics(&diagnostics, 4).is_empty());
    }

    #[test]
    fn action_icon_diagnostics_ignore_unofficial_diagnostics() {
        let mut diagnostic = diagnostic(2, 2);
        diagnostic.official_meta = None;
        let diagnostics = vec![diagnostic];

        assert!(RowDiagnosticsRenderer::action_icon_diagnostics(&diagnostics, 2).is_empty());
    }
}
