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

pub(crate) struct DiffReviewModal<'a> {
    review: &'a mut DiffReviewState,
}

impl<'a> DiffReviewModal<'a> {
    pub(crate) fn new(review: &'a mut DiffReviewState) -> Self {
        Self { review }
    }

    pub(crate) fn show(self, ctx: &egui::Context) -> Option<AppAction> {
        let Some(current_file) = self.review.current_file().cloned() else {
            return None;
        };

        let display_path = self.review.current_file_display_name();
        let mut mode = self.review.mode;
        let is_fullscreen = self.review.is_fullscreen;
        let mut request_previous = false;
        let mut request_next = false;
        let messages = crate::i18n::I18nOps::get().diff_review.clone();
        let counter = Self::counter_text(self.review.current_index + 1, self.review.files.len());
        let can_previous = self.review.can_move_previous();
        let can_next = self.review.can_move_next();
        let is_pending = current_file.decision == DiffReviewDecision::Pending;
        let modal_size = Self::modal_size(ctx, is_fullscreen);
        let mut should_fullscreen = is_fullscreen;

        let action = crate::widgets::Modal::new("diff_review_modal", &messages.title)
            .fixed_size(modal_size)
            .frame(egui::Frame::window(&ctx.global_style()).inner_margin(MODAL_INNER_MARGIN))
            .window_controls(crate::widgets::ModalWindowControls {
                is_fullscreen: is_fullscreen,
                show_fullscreen: true,
                close_tooltip: &messages.reject_all,
                enter_fullscreen_tooltip: &messages.enter_fullscreen,
                exit_fullscreen_tooltip: &messages.exit_fullscreen,
            })
            .show_with_controls(
                ctx,
                |ui| {
                    let (previous_clicked, next_clicked) =
                        Self::show_review_header(ui, &counter, can_previous, can_next, &messages);
                    request_previous = previous_clicked;
                    request_next = next_clicked;
                    None
                },
                |ui| {
                    if let Some(DiffViewerAction::ChangeMode(next_mode)) =
                        Self::show_diff_viewer(ui, &current_file, mode, display_path.clone())
                    {
                        mode = next_mode;
                    }
                    ui.add_space(FOOTER_TOP_SPACING);
                },
                |ui| Self::show_footer(ui, is_pending, &messages),
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

    fn show_review_header(
        ui: &mut egui::Ui,
        counter: &str,
        can_previous: bool,
        can_next: bool,
        messages: &crate::i18n::DiffReviewMessages,
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
                });
                ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
            })
            .show(ui);

        (move_previous, move_next)
    }

    fn show_diff_viewer(
        ui: &mut egui::Ui,
        file: &DiffReviewFile,
        mode: DiffViewMode,
        display_path: String,
    ) -> Option<DiffViewerAction> {
        DiffViewer::new(file, mode, display_path).show(ui)
    }

    fn show_footer(
        ui: &mut egui::Ui,
        is_pending: bool,
        messages: &crate::i18n::DiffReviewMessages,
    ) -> Option<AppAction> {
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

    fn modal_size(ctx: &egui::Context, is_fullscreen: bool) -> egui::Vec2 {
        if !is_fullscreen {
            return egui::vec2(MODAL_WIDTH, MODAL_HEIGHT);
        }

        let available_size = ctx.content_rect().size();
        egui::vec2(
            (available_size.x - FULLSCREEN_INSET * 2.0).max(1.0),
            (available_size.y - FULLSCREEN_INSET * 2.0).max(1.0),
        )
    }
}
