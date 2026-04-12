#[cfg(test)]
mod tests {
    use crate::app_state::{AppAction, SearchState, WorkspaceState};
    use crate::views::panels::explorer::header::ExplorerHeader;
    use eframe::egui::{self, Rect, pos2};

    fn test_context() -> egui::Context {
        egui::Context::default()
    }

    fn test_input(size: egui::Vec2) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), size)),
            ..Default::default()
        }
    }

    #[test]
    fn explorer_filter_input_width_is_stable_and_does_not_expand_infinitely() {
        let ctx = test_context();
        let mut ws = WorkspaceState::default();
        /* WHY: Just providing empty workspace representation is enough to trigger `ws.data.is_some()` */
        ws.data = Some(katana_core::workspace::Workspace::new("/", vec![]));

        let mut search = SearchState::default();
        search.filter_enabled = true; /* WHY: Show filter input! */

        let mut action = AppAction::None;

        let mut panel_width_frame1 = 0.0;
        let mut panel_width_frame2 = 0.0;
        let mut panel_width_frame3 = 0.0;

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            let resp = egui::SidePanel::left("explorer_test_panel")
                .resizable(true)
                .show(ctx, |ui| {
                    ExplorerHeader::new(&mut ws, &mut search, &mut action).show(ui);
                });
            panel_width_frame1 = resp.response.rect.width();
        });

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            let resp = egui::SidePanel::left("explorer_test_panel")
                .resizable(true)
                .show(ctx, |ui| {
                    ExplorerHeader::new(&mut ws, &mut search, &mut action).show(ui);
                });
            panel_width_frame2 = resp.response.rect.width();
        });

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            let resp = egui::SidePanel::left("explorer_test_panel")
                .resizable(true)
                .show(ctx, |ui| {
                    ExplorerHeader::new(&mut ws, &mut search, &mut action).show(ui);
                });
            panel_width_frame3 = resp.response.rect.width();
        });

        assert!(
            (panel_width_frame2 - panel_width_frame1).abs() < 1.0,
            "Explorer panel width expanded between frame 1 and 2! Width 1: {}, Width 2: {}",
            panel_width_frame1,
            panel_width_frame2
        );
        assert!(
            (panel_width_frame3 - panel_width_frame2).abs() < 1.0,
            "Explorer panel width expanded between frame 2 and 3! Width 2: {}, Width 3: {}",
            panel_width_frame2,
            panel_width_frame3
        );
    }
}
