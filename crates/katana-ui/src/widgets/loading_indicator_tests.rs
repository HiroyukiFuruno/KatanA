use super::{LoadingIndicator, LoadingLabel, LoadingPresentation, LoadingState};

const INLINE_VIEWPORT: eframe::egui::Vec2 = eframe::egui::vec2(320.0, 600.0);
const MAX_LOADING_ROW_HEIGHT: f32 = 48.0;
const MAX_FOLLOWING_BODY_TOP: f32 = 96.0;

#[test]
fn pending_indicator_uses_content_height_before_following_body() {
    let context = eframe::egui::Context::default();
    let input = eframe::egui::RawInput {
        screen_rect: Some(eframe::egui::Rect::from_min_size(
            eframe::egui::Pos2::ZERO,
            INLINE_VIEWPORT,
        )),
        ..eframe::egui::RawInput::default()
    };
    let presentation =
        LoadingPresentation::new(LoadingState::Pending, LoadingLabel::Localized("loading"));
    let mut measured = None;

    let output = context.run_ui(input, |ui| {
        let loading = LoadingIndicator::show(ui, &presentation);
        let following_body = ui.label(&crate::i18n::I18nOps::get().preview.canvas.loading);
        measured = Some((loading.rect, following_body.rect));
    });
    output.drop_without_applying_deltas();

    let (loading_rect, following_body_rect) = measured.expect("real egui layout ran");
    assert!(
        loading_rect.height() <= MAX_LOADING_ROW_HEIGHT,
        "inline loading row consumed {} points in a {} point viewport",
        loading_rect.height(),
        INLINE_VIEWPORT.y
    );
    assert!(
        following_body_rect.min.y <= MAX_FOLLOWING_BODY_TOP,
        "following body was pushed down to y={}",
        following_body_rect.min.y
    );
}

#[test]
fn pending_indicator_is_centered_on_both_axes_in_available_surface() {
    let viewport = eframe::egui::vec2(640.0, 480.0);
    let context = eframe::egui::Context::default();
    let input = eframe::egui::RawInput {
        screen_rect: Some(eframe::egui::Rect::from_min_size(
            eframe::egui::Pos2::ZERO,
            viewport,
        )),
        ..eframe::egui::RawInput::default()
    };
    let presentation =
        LoadingPresentation::new(LoadingState::Pending, LoadingLabel::Localized("loading"));
    let mut loading_rect = None;

    let output = context.run_ui(input, |ui| {
        loading_rect = Some(LoadingIndicator::show_centered(ui, &presentation).rect);
    });
    output.drop_without_applying_deltas();

    let center = loading_rect
        .expect("real egui widget was laid out")
        .center();
    assert!((center.x - viewport.x / 2.0).abs() <= 1.0);
    assert!((center.y - viewport.y / 2.0).abs() <= 1.0);
}

#[test]
fn pending_indicator_is_centered_inside_a_bounded_surface() {
    let context = eframe::egui::Context::default();
    let bounds = eframe::egui::Rect::from_min_size(
        eframe::egui::pos2(80.0, 40.0),
        eframe::egui::vec2(400.0, 300.0),
    );
    let presentation = LoadingPresentation::new(
        LoadingState::Pending,
        LoadingLabel::RenderingKind("HTML browser"),
    );
    let mut loading_rect = None;

    let output = context.run_ui(eframe::egui::RawInput::default(), |ui| {
        loading_rect = Some(LoadingIndicator::show_in_rect(ui, &presentation, bounds).rect);
    });
    output.drop_without_applying_deltas();

    let center = loading_rect
        .expect("real egui widget was laid out")
        .center();
    assert!((center.x - bounds.center().x).abs() <= 1.0);
    assert!((center.y - bounds.center().y).abs() <= 1.0);
}

#[test]
fn rendering_label_resolves_the_document_kind() {
    let presentation =
        LoadingPresentation::new(LoadingState::Pending, LoadingLabel::RenderingKind("PNG"));
    assert_eq!(presentation.state, LoadingState::Pending);
    assert!(presentation.label.contains("PNG"));
    assert!(!presentation.label.contains("{kind}"));
}

#[test]
fn localized_loading_label_is_preserved() {
    let label = "\u{753b}\u{50cf}\u{3092}\u{8aad}\u{307f}\u{8fbc}\u{307f}\u{4e2d}\u{2026}";
    let presentation =
        LoadingPresentation::new(LoadingState::Pending, LoadingLabel::Localized(label));
    assert_eq!(presentation.label, label);
    assert_eq!(presentation.state, LoadingState::Pending);
}
