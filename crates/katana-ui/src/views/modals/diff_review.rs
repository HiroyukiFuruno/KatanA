use crate::app_state::AppAction;
use crate::diff_review::{DiffReviewDecision, DiffReviewState};
use crate::views::diff_viewer::{DiffViewer, DiffViewerAction};
use eframe::egui;

const MODAL_WIDTH: f32 = 1120.0;
const MODAL_HEIGHT: f32 = 620.0;
const MODAL_INNER_MARGIN: f32 = 12.0;
const HEADER_BOTTOM_SPACING: f32 = 10.0;
const FOOTER_TOP_SPACING: f32 = 12.0;

pub(crate) struct DiffReviewModal<'a> {
    review: &'a mut DiffReviewState,
}

impl<'a> DiffReviewModal<'a> {
    pub(crate) fn new(review: &'a mut DiffReviewState) -> Self {
        Self { review }
    }

    pub(crate) fn show(mut self, ctx: &egui::Context) -> Option<AppAction> {
        self.review.current_file()?;

        let mut action = None;
        let title = crate::i18n::I18nOps::get().diff_review.title.clone();
        egui::Window::new(title)
            .id(egui::Id::new("diff_review_modal"))
            .collapsible(false)
            .resizable(false)
            .fixed_size(egui::vec2(MODAL_WIDTH, MODAL_HEIGHT))
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .frame(egui::Frame::window(&ctx.global_style()).inner_margin(MODAL_INNER_MARGIN))
            .show(ctx, |ui| {
                self.show_review_header(ui);
                ui.add_space(HEADER_BOTTOM_SPACING);
                self.show_diff_viewer(ui);
                ui.add_space(FOOTER_TOP_SPACING);
                action = self.show_footer(ui);
            });
        action
    }

    fn show_review_header(&mut self, ui: &mut egui::Ui) {
        let file_count = self.review.files.len();
        let current_number = self.review.current_index + 1;
        let counter = Self::counter_text(current_number, file_count);
        let can_previous = self.review.can_move_previous();
        let can_next = self.review.can_move_next();
        let mut move_previous = false;
        let mut move_next = false;

        crate::widgets::AlignCenter::new()
            .left(move |ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(counter).monospace().weak(),
                ))
            })
            .right(|ui| {
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .content(|ui| {
                        let messages = &crate::i18n::I18nOps::get().diff_review;
                        let next = ui
                            .add_enabled(
                                can_next,
                                crate::Icon::ChevronRight.button(ui, crate::icon::IconSize::Small),
                            )
                            .on_hover_text(&messages.next_file);
                        if next.clicked() {
                            move_next = true;
                        }
                        let previous = ui
                            .add_enabled(
                                can_previous,
                                crate::Icon::ChevronLeft.button(ui, crate::icon::IconSize::Small),
                            )
                            .on_hover_text(&messages.previous_file);
                        if previous.clicked() {
                            move_previous = true;
                        }
                    })
                    .show(ui)
            })
            .show(ui);

        if move_previous {
            self.review.move_previous();
        }
        if move_next {
            self.review.move_next();
        }
    }

    fn show_diff_viewer(&mut self, ui: &mut egui::Ui) {
        let Some(file) = self.review.current_file() else {
            return;
        };
        let display_path = self.review.current_file_display_name();
        let Some(action) = DiffViewer::new(file, self.review.mode, display_path).show(ui) else {
            return;
        };
        match action {
            DiffViewerAction::ChangeMode(mode) => {
                self.review.mode = mode;
            }
        }
    }

    fn show_footer(&self, ui: &mut egui::Ui) -> Option<AppAction> {
        let messages = &crate::i18n::I18nOps::get().diff_review;
        let is_pending = self
            .review
            .current_file()
            .is_some_and(|file| file.decision == DiffReviewDecision::Pending);
        let mut action = None;

        crate::widgets::AlignCenter::new()
            .right(|ui| {
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .content(|ui| {
                        if ui
                            .add_enabled(is_pending, egui::Button::new(&messages.apply_fix))
                            .clicked()
                        {
                            action = Some(AppAction::ConfirmCurrentDiffReviewFile);
                        }
                        if ui
                            .add_enabled(is_pending, egui::Button::new(&messages.cancel))
                            .clicked()
                        {
                            action = Some(AppAction::RejectCurrentDiffReviewFile);
                        }
                    })
                    .show(ui)
            })
            .show(ui);

        action
    }

    fn counter_text(current_number: usize, file_count: usize) -> String {
        let current = current_number.to_string();
        let total = file_count.to_string();
        crate::i18n::I18nOps::tf(
            &crate::i18n::I18nOps::get().diff_review.file_counter,
            &[("current", &current), ("total", &total)],
        )
    }
}
