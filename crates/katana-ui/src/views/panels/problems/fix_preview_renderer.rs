use eframe::egui;

const TOOLTIP_MAX_WIDTH: f32 = 400.0;
const TOOLTIP_LABEL_GAP: f32 = 4.0;
const MAX_DIFF_ROWS: usize = 10;

pub(crate) struct FixPreviewRendererOps;

impl FixPreviewRendererOps {
    pub(crate) fn show(
        ui: &mut egui::Ui,
        fix: &katana_markdown_linter::rules::markdown::DiagnosticFix,
        content: Option<&str>,
    ) {
        ui.set_max_width(TOOLTIP_MAX_WIDTH);
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(&crate::i18n::I18nOps::get().linter.fix_preview).strong());
            ui.add_space(TOOLTIP_LABEL_GAP);
            Self::show_body(ui, fix, content);
        });
    }

    fn show_body(
        ui: &mut egui::Ui,
        fix: &katana_markdown_linter::rules::markdown::DiagnosticFix,
        content: Option<&str>,
    ) {
        let Some(content) = content else {
            ui.weak(
                &crate::i18n::I18nOps::get()
                    .linter
                    .fix_preview_missing_content,
            );
            return;
        };

        let lines: Vec<&str> = content.lines().collect();
        if fix.start_line == 0 || fix.start_line > lines.len() {
            return;
        }

        let start = fix.start_line - 1;
        let end = fix.end_line.min(lines.len());
        let original = lines[start..end].join("\n");

        ui.group(|ui| {
            Self::show_removed_lines(ui, &original);
            ui.separator();
            Self::show_added_lines(ui, &fix.replacement);
        });
    }

    fn show_removed_lines(ui: &mut egui::Ui, original: &str) {
        let color = Self::removed_color(ui);
        for line in original.lines().take(MAX_DIFF_ROWS) {
            let text = crate::i18n::I18nOps::tf(
                &crate::i18n::I18nOps::get().linter.fix_preview_removed_line,
                &[("line", line)],
            );
            ui.label(egui::RichText::new(text).color(color));
        }
        Self::show_overflow(ui, original.lines().count());
    }

    fn show_added_lines(ui: &mut egui::Ui, replacement: &str) {
        let color = Self::added_color(ui);
        for line in replacement.lines().take(MAX_DIFF_ROWS) {
            let text = crate::i18n::I18nOps::tf(
                &crate::i18n::I18nOps::get().linter.fix_preview_added_line,
                &[("line", line)],
            );
            ui.label(egui::RichText::new(text).color(color));
        }
        Self::show_overflow(ui, replacement.lines().count());
    }

    fn show_overflow(ui: &mut egui::Ui, line_count: usize) {
        if line_count > MAX_DIFF_ROWS {
            ui.label(
                egui::RichText::new(&crate::i18n::I18nOps::get().linter.fix_preview_more).weak(),
            );
        }
    }

    fn removed_color(ui: &egui::Ui) -> egui::Color32 {
        Self::theme_colors(ui)
            .map(|it| crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(it.system.error_text))
            .unwrap_or_else(|| ui.visuals().error_fg_color)
    }

    fn added_color(ui: &egui::Ui) -> egui::Color32 {
        Self::theme_colors(ui)
            .map(|it| crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(it.system.success_text))
            .unwrap_or_else(|| ui.visuals().hyperlink_color)
    }

    fn theme_colors(ui: &egui::Ui) -> Option<katana_platform::theme::ThemeColors> {
        ui.data(|data| {
            data.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                "katana_theme_colors",
            ))
        })
    }
}
