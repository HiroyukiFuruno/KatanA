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
}
