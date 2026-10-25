use eframe::egui;

/// Canonical wrapper around `ui.menu_button()`.
///
/// `ui.menu_button` conditionally shows a frame only on hover, which normally
/// triggers the `conditional_frame` lint. This widget is the **only sanctioned
/// call-site** so the linter excludes this file and callers stay clean.
pub struct MenuButtonOps;

impl MenuButtonOps {
    pub fn show<R>(
        ui: &mut egui::Ui,
        label: impl Into<egui::WidgetText>,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<Option<R>> {
        ui.menu_button(label, add_contents)
    }

    pub fn show_unframed<'a, R>(
        ui: &mut egui::Ui,
        label: impl egui::IntoAtoms<'a>,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<Option<R>> {
        Self::show_unframed_with_close_behavior(
            ui,
            label,
            egui::PopupCloseBehavior::CloseOnClick,
            add_contents,
        )
    }

    pub fn show_unframed_interactive<'a, R>(
        ui: &mut egui::Ui,
        label: impl egui::IntoAtoms<'a>,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<Option<R>> {
        Self::show_unframed_with_close_behavior(
            ui,
            label,
            egui::PopupCloseBehavior::CloseOnClickOutside,
            add_contents,
        )
    }

    fn show_unframed_with_close_behavior<'a, R>(
        ui: &mut egui::Ui,
        label: impl egui::IntoAtoms<'a>,
        close_behavior: egui::PopupCloseBehavior,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<Option<R>> {
        let button = egui::Button::new(label).small().frame(false);
        let config = egui::containers::menu::MenuConfig::new().close_behavior(close_behavior);
        let (response, inner) = egui::containers::menu::MenuButton::from_button(button)
            .config(config)
            .ui(ui, add_contents);
        egui::InnerResponse::new(inner.map(|r| r.inner), response)
    }
}
