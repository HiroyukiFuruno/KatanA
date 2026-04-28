mod helpers;
use helpers::*;

use crate::app_state::AppAction;
use crate::diff_review::{DiffReviewDecision, DiffReviewFile, DiffReviewState};
use crate::views::diff_viewer::{DiffViewer, DiffViewerAction};
use eframe::egui;
use katana_platform::DiffViewMode;

const MODAL_WIDTH: f32 = 1120.0;
const MODAL_HEIGHT: f32 = 620.0;
const FULLSCREEN_INSET: f32 = 8.0;
const MODAL_INNER_MARGIN: f32 = 12.0;
const FOOTER_TOP_SPACING: f32 = 12.0;
const MIN_DIFF_VIEWER_HEIGHT: f32 = 200.0;
const MODAL_INNER_MARGIN_I: i8 = 12;

pub(crate) struct DiffReviewModal<'a> {
    review: &'a mut DiffReviewState,
}

impl<'a> DiffReviewModal<'a> {
    pub(crate) fn new(review: &'a mut DiffReviewState) -> Self {
        Self { review }
    }

    pub(crate) fn show(self, ctx: &egui::Context) -> Option<AppAction> {
        let current_file = self.review.current_file().cloned()?;

        let display_path = self.review.current_file_display_name();
        let mut mode = self.review.mode;
        let is_fullscreen = self.review.is_fullscreen;
        let mut request_previous = false;
        let mut request_next = false;
        let messages = crate::i18n::I18nOps::get().diff_review.clone();
        let counter =
            DiffReviewUi::counter_text(self.review.current_index + 1, self.review.files.len());
        let can_previous = self.review.can_move_previous();
        let can_next = self.review.can_move_next();
        let is_pending = current_file.decision == DiffReviewDecision::Pending;
        let modal_size = DiffReviewUi::modal_size(ctx, is_fullscreen);
        let mut should_fullscreen = is_fullscreen;

        let action = crate::widgets::Modal::new("diff_review_modal", &messages.title)
            .fixed_size(modal_size)
            .frame(egui::Frame::window(&ctx.global_style()).inner_margin(MODAL_INNER_MARGIN))
            .window_controls(crate::widgets::ModalWindowControls {
                is_fullscreen,
                show_fullscreen: true,
                close_tooltip: &messages.reject_all,
                enter_fullscreen_tooltip: &messages.enter_fullscreen,
                exit_fullscreen_tooltip: &messages.exit_fullscreen,
            })
            .show_with_controls(
                ctx,
                |ui| {
                    let (previous_clicked, next_clicked) = DiffReviewUi::show_review_header(
                        ui,
                        &counter,
                        can_previous,
                        can_next,
                        &messages,
                    );
                    request_previous = previous_clicked;
                    request_next = next_clicked;
                    None
                },
                |ui| {
                    if let Some(DiffViewerAction::ChangeMode(next_mode)) =
                        Self::show_diff_viewer(ui, &current_file, mode, Some(display_path.clone()))
                    {
                        mode = next_mode;
                    }
                    ui.add_space(FOOTER_TOP_SPACING);
                },
                |ui| DiffReviewUi::show_footer(ui, is_pending, &messages),
                |button| match button {
                    crate::widgets::ModalWindowButton::Close => {
                        Some(AppAction::RejectAllDiffReviewFiles)
                    }
                    crate::widgets::ModalWindowButton::Fullscreen => {
                        should_fullscreen = !should_fullscreen;
                        None
                    }
                },
            );

        if request_previous {
            self.review.move_previous();
        }
        if request_next {
            self.review.move_next();
        }
        self.review.mode = mode;
        self.review.is_fullscreen = should_fullscreen;
        action
    }

    pub(crate) fn show_in_tab(self, ui: &mut egui::Ui) -> Option<AppAction> {
        let current_file = self.review.current_file().cloned()?;

        let display_path = self.review.current_file_display_name();
        let mut mode = self.review.mode;
        let mut request_previous = false;
        let mut request_next = false;
        let messages = crate::i18n::I18nOps::get().diff_review.clone();
        let counter =
            DiffReviewUi::counter_text(self.review.current_index + 1, self.review.files.len());
        let can_previous = self.review.can_move_previous();
        let can_next = self.review.can_move_next();
        let is_pending = current_file.decision == DiffReviewDecision::Pending;
        let mut action = None;

        egui::Frame::none().inner_margin(egui::Margin {
            left: MODAL_INNER_MARGIN_I,
            right: MODAL_INNER_MARGIN_I,
            top: MODAL_INNER_MARGIN_I,
            bottom: MODAL_INNER_MARGIN_I,
        })
        .show(ui, |ui| {
            let (previous_clicked, next_clicked) =
                DiffReviewUi::show_review_header(ui, &counter, can_previous, can_next, &messages);
            request_previous = previous_clicked;
            request_next = next_clicked;

            /* WHY: Reserve footer height up-front so the diff viewer does not push buttons off-screen. */
            let footer_h = ui.spacing().interact_size.y
                + ui.spacing().item_spacing.y
                + FOOTER_TOP_SPACING;
            let diff_h = (ui.available_height() - footer_h).max(MIN_DIFF_VIEWER_HEIGHT);

            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), diff_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    if let Some(DiffViewerAction::ChangeMode(next_mode)) =
                        Self::show_diff_viewer(ui, &current_file, mode, Some(display_path.clone()))
                    {
                        mode = next_mode;
                    }
                },
            );

            ui.add_space(FOOTER_TOP_SPACING);
            if let Some(footer_action) = DiffReviewUi::show_footer(ui, is_pending, &messages) {
                action = Some(footer_action);
            }
        });

        if request_previous {
            self.review.move_previous();
        }
        if request_next {
            self.review.move_next();
        }
        self.review.mode = mode;
        action
    }

    fn show_diff_viewer(
        ui: &mut egui::Ui,
        file: &DiffReviewFile,
        mode: DiffViewMode,
        display_path: Option<String>,
    ) -> Option<DiffViewerAction> {
        DiffViewer::new(file, mode, display_path).show(ui)
    }
}
