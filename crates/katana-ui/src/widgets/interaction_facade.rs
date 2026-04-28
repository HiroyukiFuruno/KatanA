use eframe::egui;

const HOVER_BLOCKER_RECTS_ID: &str = "interaction_facade_hover_blocker_rects";
const HOVER_BLOCKED_ID: &str = "interaction_facade_hover_blocked";

/// Facade used to block interaction with underlying UI elements
/// without changing their visual opacity.
pub struct InteractionFacade;

impl InteractionFacade {
    pub fn begin_frame(ctx: &egui::Context) {
        let pointer_pos = ctx.input(|it| it.pointer.hover_pos());
        let blocker_rects = ctx.data(|it| {
            it.get_temp::<Vec<egui::Rect>>(egui::Id::new(HOVER_BLOCKER_RECTS_ID))
                .unwrap_or_default()
        });
        let is_blocked = pointer_pos.is_some_and(|pos| {
            blocker_rects
                .iter()
                .any(|blocker_rect| blocker_rect.contains(pos))
        });

        ctx.data_mut(|it| {
            it.insert_temp(egui::Id::new(HOVER_BLOCKED_ID), is_blocked);
            it.insert_temp(
                egui::Id::new(HOVER_BLOCKER_RECTS_ID),
                Vec::<egui::Rect>::new(),
            );
        });
    }

    pub fn is_hover_blocked(ctx: &egui::Context) -> bool {
        ctx.data(|it| {
            it.get_temp::<bool>(egui::Id::new(HOVER_BLOCKED_ID))
                .unwrap_or(false)
        })
    }

    /// Creates a scope that disables interaction (clicks/hovers) on all contents within it
    /// if `is_blocked` is true, while preserving their original visual opacity (`disabled_alpha` = 1.0)
    /// so it doesn't look grayed out.
    pub fn scope<R>(
        ui: &mut egui::Ui,
        is_blocked: bool,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        let is_blocked = is_blocked || Self::is_hover_blocked(ui.ctx());
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

    pub fn consume_rect(ui: &mut egui::Ui, id: &'static str, rect: egui::Rect) {
        Self::register_hover_blocker(ui.ctx(), rect);
        let _response = ui.interact(rect, egui::Id::new(id), egui::Sense::click_and_drag());
    }

    pub fn register_hover_blocker(ctx: &egui::Context, rect: egui::Rect) {
        let pointer_inside =
            ctx.input(|it| it.pointer.hover_pos().is_some_and(|pos| rect.contains(pos)));
        let was_blocked = Self::is_hover_blocked(ctx);

        ctx.data_mut(|it| {
            let mut blocker_rects = it
                .get_temp::<Vec<egui::Rect>>(egui::Id::new(HOVER_BLOCKER_RECTS_ID))
                .unwrap_or_default();
            blocker_rects.push(rect);
            it.insert_temp(egui::Id::new(HOVER_BLOCKER_RECTS_ID), blocker_rects);
            if pointer_inside {
                it.insert_temp(egui::Id::new(HOVER_BLOCKED_ID), true);
            }
        });

        if pointer_inside && !was_blocked {
            ctx.request_repaint();
        }
    }
}
