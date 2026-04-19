use eframe::egui;

/// Facade used to block interaction with underlying UI elements
/// without changing their visual opacity.
pub struct InteractionFacade;

impl InteractionFacade {
    /// Creates a scope that disables interaction (clicks/hovers) on all contents within it
    /// if `is_blocked` is true, while preserving their original visual opacity (`disabled_alpha` = 1.0)
    /// so it doesn't look grayed out.
    pub fn scope<R>(
        ui: &mut egui::Ui,
        is_blocked: bool,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        let prev_enabled = ui.is_enabled();
        let prev_alpha = ui.visuals().disabled_alpha;

        if is_blocked {
            ui.visuals_mut().disabled_alpha = 1.0;
        }
        ui.set_enabled(!is_blocked);

        let res = add_contents(ui);

        ui.set_enabled(prev_enabled);
        ui.visuals_mut().disabled_alpha = prev_alpha;
        res
    }
}
