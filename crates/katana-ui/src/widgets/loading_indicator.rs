use eframe::egui;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum LoadingState {
    Pending,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum LoadingLabel<'a> {
    RenderingKind(&'a str),
    Localized(&'a str),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LoadingPresentation {
    pub(crate) state: LoadingState,
    pub(crate) label: String,
}

impl LoadingPresentation {
    pub(crate) fn new(state: LoadingState, label: LoadingLabel<'_>) -> Self {
        let label = match label {
            LoadingLabel::RenderingKind(kind) => crate::i18n::I18nOps::tf(
                &crate::i18n::I18nOps::get().preview.rendering,
                &[("kind", kind)],
            ),
            LoadingLabel::Localized(label) => label.to_owned(),
        };
        Self { state, label }
    }
}

pub(crate) struct LoadingIndicator;

impl LoadingIndicator {
    pub(crate) fn show(ui: &mut egui::Ui, presentation: &LoadingPresentation) -> egui::Response {
        match presentation.state {
            LoadingState::Pending => {
                /* WHY: 本文フローの高さを保ち、全面表示の中央寄せは呼出側レイアウトに委ねる。 */
                crate::widgets::AlignCenter::new()
                    .spacing(ui.spacing().item_spacing.x)
                    .shrink_to_fit(true)
                    .left(|ui| ui.spinner())
                    .left(|ui| ui.label(egui::RichText::new(&presentation.label).weak()))
                    .show(ui)
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
            let following_body = ui.label("following body");
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
}
