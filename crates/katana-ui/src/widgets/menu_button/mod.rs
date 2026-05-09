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
        let button = egui::Button::new(label).small().frame(false);
        let (response, inner) =
            egui::containers::menu::MenuButton::from_button(button).ui(ui, add_contents);
        egui::InnerResponse::new(inner.map(|r| r.inner), response)
    }
}
