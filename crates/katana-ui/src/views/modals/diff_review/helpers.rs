use crate::i18n::DiffReviewMessages;
use eframe::egui;

pub(crate) struct DiffReviewUi;

impl DiffReviewUi {
    pub(crate) fn show_review_header(
        ui: &mut egui::Ui,
        counter: &str,
        can_previous: bool,
        can_next: bool,
        messages: &DiffReviewMessages,
    ) -> (bool, bool) {
        let mut move_previous = false;
        let mut move_next = false;

        crate::widgets::AlignCenter::new()
            .left(|ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(counter.to_owned()).monospace().weak(),
                ));
                ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
            })
            .right(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    /* WHY: Add `next` first so that in a right-to-left layout the
                    next button appears at the far right and previous to its
                    left (expected UX). */
                    let next = ui
                        .add_enabled(
                            can_next,
                            crate::Icon::ChevronRight.button(ui, crate::icon::IconSize::Small),
                        )
                        .on_hover_text(&messages.next_file)
                        .on_disabled_hover_text(&messages.next_file);
                    if next.clicked() {
                        move_next = true;
                    }

                    let previous = ui
                        .add_enabled(
                            can_previous,
                            crate::Icon::ChevronLeft.button(ui, crate::icon::IconSize::Small),
                        )
                        .on_hover_text(&messages.previous_file)
                        .on_disabled_hover_text(&messages.previous_file);
                    if previous.clicked() {
                        move_previous = true;
                    }
                });
                ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
            })
            .show(ui);

        (move_previous, move_next)
    }

    pub(crate) fn show_footer(
        ui: &mut egui::Ui,
        is_pending: bool,
        messages: &DiffReviewMessages,
    ) -> Option<crate::app_state::AppAction> {
        let mut action = None;

        crate::widgets::AlignCenter::new()
            .right(|ui| {
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .content(|ui| {
                        if ui
                            .add_enabled(is_pending, egui::Button::new(&messages.cancel))
                            .clicked()
                        {
                            action = Some(crate::app_state::AppAction::RejectCurrentDiffReviewFile);
                        }
                        if ui
                            .add_enabled(is_pending, egui::Button::new(&messages.apply_fix))
                            .clicked()
                        {
                            action =
                                Some(crate::app_state::AppAction::ConfirmCurrentDiffReviewFile);
                        }
                    })
                    .show(ui)
            })
            .show(ui);

        action
    }

    pub(crate) fn counter_text(current_number: usize, file_count: usize) -> String {
        let current = current_number.to_string();
        let total = file_count.to_string();
        crate::i18n::I18nOps::tf(
            &crate::i18n::I18nOps::get().diff_review.file_counter,
            &[("current", &current), ("total", &total)],
        )
    }

    pub(crate) fn modal_size(ctx: &egui::Context, is_fullscreen: bool) -> egui::Vec2 {
        if !is_fullscreen {
            return egui::vec2(super::MODAL_WIDTH, super::MODAL_HEIGHT);
        }

        let available_size = ctx.content_rect().size();
        egui::vec2(
            (available_size.x - super::FULLSCREEN_INSET * 2.0).max(1.0),
            (available_size.y - super::FULLSCREEN_INSET * 2.0).max(1.0),
        )
    }
}
