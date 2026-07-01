use crate::views::panels::toc::TocPanel;
use eframe::egui;

#[test]
fn active_leaf_item_requests_scroll_into_view() {
    let ctx = egui::Context::default();
    let mut scroll_offset = 0.0;

    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(240.0, 180.0),
        )),
        ..Default::default()
    };

    let _ = ctx.run_ui(input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let output = egui::ScrollArea::vertical()
                .animated(false)
                .max_height(120.0)
                .show(ui, |ui| {
                    for index in 0..20 {
                        let response = ui.allocate_response(
                            egui::vec2(ui.available_width(), 24.0),
                            egui::Sense::hover(),
                        );
                        TocPanel::scroll_active_item_into_view(&response, index == 15, true);
                    }
                });
            scroll_offset = output.state.offset.y;
        });
    });

    assert!(scroll_offset > 0.0);
}

#[test]
fn active_parent_item_requests_scroll_into_view() {
    let ctx = egui::Context::default();
    let mut scroll_offset = 0.0;

    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(240.0, 180.0),
        )),
        ..Default::default()
    };

    let _ = ctx.run_ui(input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let output = egui::ScrollArea::vertical()
                .animated(false)
                .max_height(120.0)
                .show(ui, |ui| {
                    for index in 0..20 {
                        let response = crate::widgets::Accordion::new(
                            ("toc_test_parent", index),
                            format!("Heading {index}"),
                            |_| {},
                        )
                        .default_open(false)
                        .show(ui);
                        TocPanel::scroll_active_item_into_view(
                            &response.response,
                            index == 15,
                            true,
                        );
                    }
                });
            scroll_offset = output.state.offset.y;
        });
    });

    assert!(scroll_offset > 0.0);
}

#[test]
fn active_item_keeps_toc_scroll_when_auto_scroll_is_disabled() {
    let ctx = egui::Context::default();
    let mut scroll_offset = 0.0;

    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(240.0, 180.0),
        )),
        ..Default::default()
    };

    let _ = ctx.run_ui(input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let output = egui::ScrollArea::vertical()
                .animated(false)
                .max_height(120.0)
                .show(ui, |ui| {
                    for index in 0..20 {
                        let response = ui.allocate_response(
                            egui::vec2(ui.available_width(), 24.0),
                            egui::Sense::hover(),
                        );
                        TocPanel::scroll_active_item_into_view(&response, index == 15, false);
                    }
                });
            scroll_offset = output.state.offset.y;
        });
    });

    assert_eq!(scroll_offset, 0.0);
}
