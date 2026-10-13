use eframe::egui;
use egui_kittest::{Harness, kittest::Queryable};
use std::{cell::RefCell, rc::Rc};

use super::render_tests::CanvasTest;

#[test]
fn scene_pointer_pinch_zoom_changes_view_without_changing_document() {
    let mut surface = CanvasTest::surface();
    let context = egui::Context::default();
    CanvasTest::frame(&mut surface, &context, vec![]);
    let pointer = egui::pos2(400.0, 300.0);
    let move_event = egui::Event::PointerMoved(pointer);
    CanvasTest::frame(&mut surface, &context, vec![move_event]);
    let before = surface.view_rect();
    CanvasTest::frame(&mut surface, &context, vec![egui::Event::Zoom(2.0)]);
    assert!(surface.view_rect().width() < before.width());
    assert_eq!(surface.layout.rects.len(), 6);
}

#[test]
fn scene_drag_pans_and_reset_fits_the_original_graph() {
    let mut surface = CanvasTest::surface();
    let context = egui::Context::default();
    CanvasTest::frame(&mut surface, &context, vec![]);
    let pointer = egui::pos2(400.0, 300.0);
    let move_event = egui::Event::PointerMoved(pointer);
    CanvasTest::frame(&mut surface, &context, vec![move_event]);
    let before = surface.view_rect();
    CanvasTest::frame(
        &mut surface,
        &context,
        vec![CanvasTest::pointer_button(pointer, true)],
    );
    CanvasTest::frame(
        &mut surface,
        &context,
        vec![egui::Event::PointerMoved(pointer + egui::vec2(80.0, 30.0))],
    );
    assert_ne!(surface.view_rect().center(), before.center());
    CanvasTest::frame(
        &mut surface,
        &context,
        vec![CanvasTest::pointer_button(pointer, false)],
    );
    surface.reset_view();
    CanvasTest::frame(&mut surface, &context, vec![]);
    assert_eq!(surface.view_rect(), before);
}

#[test]
fn actual_fit_button_restores_graph_center_after_pan() {
    let surface = Rc::new(RefCell::new(CanvasTest::surface()));
    let shared = Rc::clone(&surface);
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 600.0))
        .build_ui(move |ui| {
            shared.borrow_mut().show(ui, &CanvasTest::labels());
        });
    let expected_center = surface.borrow().view_rect().center();
    {
        let mut surface = surface.borrow_mut();
        surface.view_rect = surface.view_rect.translate(egui::vec2(500.0, 300.0));
    }
    harness.step();
    harness.get_by_label("全体表示").click();
    harness.step();
    assert_eq!(surface.borrow().view_rect().center(), expected_center);
}
