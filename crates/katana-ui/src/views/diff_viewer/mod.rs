mod inline;
mod row;
mod split;
mod style;

use eframe::egui;
use katana_platform::DiffViewMode;
use style::DiffViewerPalette;

const HEADER_BOTTOM_SPACING: f32 = 8.0;
const VIEWER_BODY_MAX_HEIGHT: f32 = 380.0;

pub(crate) struct DiffViewer<'a> {
    file: &'a crate::diff_review::DiffReviewFile,
    mode: DiffViewMode,
    display_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffViewerAction {
    ChangeMode(DiffViewMode),
}

impl<'a> DiffViewer<'a> {
    pub(crate) fn new(
        file: &'a crate::diff_review::DiffReviewFile,
        mode: DiffViewMode,
        display_path: String,
    ) -> Self {
        Self {
            file,
            mode,
            display_path,
        }
    }

    pub(crate) fn show(self, ui: &mut egui::Ui) -> Option<DiffViewerAction> {
        let action = self.show_header(ui);
        ui.add_space(HEADER_BOTTOM_SPACING);
        self.show_body(ui);
        action
    }

    fn show_header(&self, ui: &mut egui::Ui) -> Option<DiffViewerAction> {
        let mut action = None;
        let path = self.display_path.clone();

        crate::widgets::AlignCenter::new()
            .left(move |ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(path).monospace().strong(),
                ))
            })
            .right(|ui| {
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .content(|ui| {
                        if self.mode_button(ui, DiffViewMode::Inline, crate::Icon::Code) {
                            action = Some(DiffViewerAction::ChangeMode(DiffViewMode::Inline));
                        }
                        if self.mode_button(ui, DiffViewMode::Split, crate::Icon::SplitVertical) {
                            action = Some(DiffViewerAction::ChangeMode(DiffViewMode::Split));
                        }
                    })
                    .show(ui)
            })
            .left(|ui| self.stats_labels(ui))
            .show(ui);

        action
    }

    fn mode_button(&self, ui: &mut egui::Ui, mode: DiffViewMode, icon: crate::Icon) -> bool {
        let messages = &crate::i18n::I18nOps::get().diff_review;
        let tooltip = if mode == DiffViewMode::Split {
            &messages.switch_to_split
        } else {
            &messages.switch_to_inline
        };
        ui.add(icon.selected_button(ui, crate::icon::IconSize::Small, self.mode == mode))
            .on_hover_text(tooltip)
            .clicked()
            && self.mode != mode
    }

    fn show_body(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt(ui.id().with("diff_viewer_scroll"))
            .max_height(VIEWER_BODY_MAX_HEIGHT)
            .show(ui, |ui| match self.mode {
                DiffViewMode::Split => split::DiffViewerSplitOps::show(ui, self.file),
                DiffViewMode::Inline => inline::DiffViewerInlineOps::show(ui, self.file),
            });
    }

    fn stats_labels(&self, ui: &mut egui::Ui) -> egui::Response {
        let palette = DiffViewerPalette::from_ui(ui);
        let added = stat_text("+", self.file.model.stats.added_count);
        let removed = stat_text("-", self.file.model.stats.removed_count);
        ui.label(
            egui::RichText::new(added)
                .monospace()
                .color(palette.added_text),
        );
        ui.label(
            egui::RichText::new(removed)
                .monospace()
                .color(palette.removed_text),
        )
    }
}

fn stat_text(prefix: &str, count: usize) -> String {
    let mut text = prefix.to_string();
    text.push_str(&count.to_string());
    text
}
