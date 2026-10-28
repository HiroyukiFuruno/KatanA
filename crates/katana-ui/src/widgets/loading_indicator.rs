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
                ui.horizontal_centered(|ui| {
                    ui.spinner();
                    ui.label(egui::RichText::new(&presentation.label).weak());
                })
                .response
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LoadingLabel, LoadingPresentation, LoadingState};

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
