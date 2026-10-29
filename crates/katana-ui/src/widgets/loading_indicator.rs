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
                let bounds = ui.available_rect_before_wrap();
                Self::show_in_bounds(ui, presentation, bounds, false)
            }
        }
    }

    pub(crate) fn show_centered(
        ui: &mut egui::Ui,
        presentation: &LoadingPresentation,
    ) -> egui::Response {
        let bounds = ui.available_rect_before_wrap();
        Self::show_in_rect(ui, presentation, bounds)
    }

    pub(crate) fn show_in_rect(
        ui: &mut egui::Ui,
        presentation: &LoadingPresentation,
        bounds: egui::Rect,
    ) -> egui::Response {
        match presentation.state {
            LoadingState::Pending => Self::show_in_bounds(ui, presentation, bounds, true),
        }
    }

    fn show_in_bounds(
        ui: &mut egui::Ui,
        presentation: &LoadingPresentation,
        bounds: egui::Rect,
        center_vertically: bool,
    ) -> egui::Response {
        /* WHY: AlignCenter は全幅行の内容を左端へ置くため、実際の内容幅で中央寄せする。 */
        let content_rect = Self::content_rect(ui, presentation, bounds, center_vertically);
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            |ui| {
                let spinner = ui.spinner();
                let label = ui.label(egui::RichText::new(&presentation.label).weak());
                spinner.union(label)
            },
        )
        .inner
    }

    fn content_rect(
        ui: &egui::Ui,
        presentation: &LoadingPresentation,
        bounds: egui::Rect,
        center_vertically: bool,
    ) -> egui::Rect {
        let style = ui.style();
        let spinner_size = style.spacing.interact_size.y;
        let label_galley = ui.painter().layout_no_wrap(
            presentation.label.clone(),
            egui::TextStyle::Body.resolve(style),
            ui.visuals().weak_text_color(),
        );
        let content_width = spinner_size + style.spacing.item_spacing.x + label_galley.size().x;
        let center_y = if center_vertically {
            bounds.center().y
        } else {
            bounds.min.y + spinner_size / 2.0
        };
        egui::Rect::from_center_size(
            egui::pos2(bounds.center().x, center_y),
            egui::vec2(content_width, spinner_size),
        )
    }
}
