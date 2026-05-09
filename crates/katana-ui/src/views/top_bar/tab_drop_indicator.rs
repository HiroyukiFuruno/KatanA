use crate::shell::{TAB_DROP_ANIMATION_TIME, TAB_DROP_INDICATOR_WIDTH};
use crate::views::top_bar::types::TopBarOps;
use eframe::egui;

pub(crate) struct TabDropIndicator<'a> {
    pub tab_rects: &'a [(usize, egui::Rect)],
    pub ghost_info: Option<(egui::Rect, egui::Rangef)>,
    pub id_salt: &'static str,
}

impl<'a> TabDropIndicator<'a> {
    pub(crate) fn render(self, ui: &mut egui::Ui) {
        let Some((ghost_rect, y_range)) = self.ghost_info else {
            return;
        };
        let drop_points = TopBarOps::compute_drop_points(self.tab_rects);
        let best_x = drop_points
            .iter()
            .min_by(|(_, a), (_, b)| {
                let da = (ghost_rect.center().x - a).abs();
                let db = (ghost_rect.center().x - b).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(_, x)| *x);
        let Some(target_x) = best_x else { return };
        let animated_x = ui.ctx().animate_value_with_time(
            egui::Id::new(self.id_salt),
            target_x,
            TAB_DROP_ANIMATION_TIME,
        );
        ui.painter().vline(
            animated_x,
            y_range,
            egui::Stroke::new(TAB_DROP_INDICATOR_WIDTH, ui.visuals().selection.bg_fill),
        );
    }
}
