use crate::app_state::AppAction;
use crate::state::{AutofixState, DiffLineKind, DiffPreviewLine};
use eframe::egui;

const DIFF_MODAL_WIDTH: f32 = 760.0;
const DIFF_MODAL_HEIGHT: f32 = 520.0;
const LINE_NUMBER_WIDTH: f32 = 46.0;

pub(crate) struct AutofixDiffModal<'a> {
    autofix: &'a AutofixState,
    pending_action: &'a mut AppAction,
}

impl<'a> AutofixDiffModal<'a> {
    pub(crate) fn new(autofix: &'a AutofixState, pending_action: &'a mut AppAction) -> Self {
        Self {
            autofix,
            pending_action,
        }
    }

    pub(crate) fn show(self, ctx: &egui::Context) {
        let Some(candidate) = self.autofix.candidate.as_ref() else {
            return;
        };
        let messages = &crate::i18n::I18nOps::get().linter;
        egui::Window::new(&messages.autofix_diff_title)
            .collapsible(false)
            .resizable(true)
            .default_width(DIFF_MODAL_WIDTH)
            .default_height(DIFF_MODAL_HEIGHT)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(candidate.path.display().to_string());
                ui.separator();
                self.render_diff(ui);
                ui.separator();
                self.render_actions(ui);
            });
    }

    fn render_diff(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt("autofix_diff_preview")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let Some(candidate) = self.autofix.candidate.as_ref() else {
                    return;
                };
                for line in &candidate.diff.lines {
                    Self::render_line(ui, line);
                }
            });
    }

    fn render_line(ui: &mut egui::Ui, line: &DiffPreviewLine) {
        crate::widgets::AlignCenter::new()
            .content(|ui| {
                ui.set_min_height(ui.text_style_height(&egui::TextStyle::Monospace));
                ui.add_sized(
                    [LINE_NUMBER_WIDTH, 0.0],
                    egui::Label::new(
                        egui::RichText::new(Self::line_number(line))
                            .monospace()
                            .weak(),
                    ),
                );
                ui.label(Self::prefix(line, ui));
                ui.label(Self::text(line, ui));
            })
            .show(ui);
    }

    fn line_number(line: &DiffPreviewLine) -> String {
        match (line.old_line, line.new_line) {
            (Some(old), Some(new)) => format!("{old}/{new}"),
            (Some(old), None) => old.to_string(),
            (None, Some(new)) => new.to_string(),
            (None, None) => String::new(),
        }
    }

    fn prefix(line: &DiffPreviewLine, ui: &egui::Ui) -> egui::RichText {
        let text = match line.kind {
            DiffLineKind::Unchanged => " ",
            DiffLineKind::Added => "+",
            DiffLineKind::Removed => "-",
        };
        egui::RichText::new(text)
            .monospace()
            .color(Self::line_color(line.kind, ui))
    }

    fn text(line: &DiffPreviewLine, ui: &egui::Ui) -> egui::RichText {
        let text = if line.text.is_empty() {
            " "
        } else {
            &line.text
        };
        let color = match line.kind {
            DiffLineKind::Unchanged => ui.visuals().text_color(),
            _ => Self::line_color(line.kind, ui),
        };
        egui::RichText::new(text).monospace().color(color)
    }

    fn line_color(kind: DiffLineKind, ui: &egui::Ui) -> egui::Color32 {
        let theme_colors = ui.data(|data| {
            data.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                "katana_theme_colors",
            ))
        });
        match kind {
            DiffLineKind::Unchanged => ui.visuals().weak_text_color(),
            DiffLineKind::Added => theme_colors
                .as_ref()
                .map(|colors| {
                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(colors.system.success_text)
                })
                .unwrap_or_else(|| ui.visuals().text_color()),
            DiffLineKind::Removed => theme_colors
                .as_ref()
                .map(|colors| {
                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(colors.system.error_text)
                })
                .unwrap_or_else(|| ui.visuals().error_fg_color),
        }
    }

    fn render_actions(self, ui: &mut egui::Ui) {
        let messages = &crate::i18n::I18nOps::get().linter;
        crate::widgets::AlignCenter::new()
            .content(|ui| {
                if ui.button(&messages.autofix_cancel).clicked() {
                    *self.pending_action = AppAction::CancelAutofixCandidate;
                }
                if ui.button(&messages.autofix_apply).clicked() {
                    *self.pending_action = AppAction::ApplyAutofixCandidate;
                }
            })
            .show(ui);
    }
}
