use super::DiagramControllerOps;
use crate::preview_pane::ViewerState;
use eframe::egui;
use egui_kittest::Harness;
use std::sync::{Arc, Mutex};

const INITIAL_ZOOM: f32 = 1.5;
const INITIAL_PAN_X: f32 = 12.0;
const INITIAL_PAN_Y: f32 = -8.0;
const BUTTON_CENTER_FRACTION: f32 = 0.5;

#[derive(Clone, Copy)]
enum Layout {
    Below,
    Fullscreen,
}

#[test]
fn below_controls_pointer_zoom_and_reset_do_not_overlap_image_in_small_viewport() {
    assert_pointer_controls_work(Layout::Below, egui::vec2(140.0, 190.0));
}

#[test]
fn fullscreen_controls_pointer_zoom_and_reset_do_not_overlap_image_in_small_viewport() {
    assert_pointer_controls_work(Layout::Fullscreen, egui::vec2(140.0, 190.0));
}

fn assert_pointer_controls_work(layout: Layout, size: egui::Vec2) {
    let _guard = crate::test_render_env::RenderEnvLock::lock();
    let state = Arc::new(Mutex::new(ViewerState {
        zoom: INITIAL_ZOOM,
        pan: egui::vec2(INITIAL_PAN_X, INITIAL_PAN_Y),
        ..ViewerState::default()
    }));
    let rects = Arc::new(Mutex::new(None));
    let mut harness = build_harness(layout, size, Arc::clone(&state), Arc::clone(&rects));
    assert_disjoint_image_controls(&rects);
    click_zoom_buttons(&mut harness, &state, &rects);
    click_reset_button(&mut harness, &state, &rects);
}

fn build_harness(
    layout: Layout,
    size: egui::Vec2,
    state: Arc<Mutex<ViewerState>>,
    rects: Arc<Mutex<Option<(egui::Rect, egui::Rect)>>>,
) -> Harness<'static> {
    Harness::builder().with_size(size).build_ui(move |ui| {
        let mut state = state.lock().expect("viewer state lock");
        let (image, controls) = render_layout(ui, layout, &mut state);
        *rects.lock().expect("layout rectangles lock") = Some((image, controls));
    })
}

fn render_layout(
    ui: &mut egui::Ui,
    layout: Layout,
    state: &mut ViewerState,
) -> (egui::Rect, egui::Rect) {
    match layout {
        Layout::Below => {
            let image_height = (ui.available_height() - DiagramControllerOps::controls_height())
                .max(DiagramControllerOps::minimum_image_height());
            let (image, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), image_height),
                egui::Sense::hover(),
            );
            let controls = DiagramControllerOps::draw_controls_below(ui, state);
            (image, controls)
        }
        Layout::Fullscreen => {
            let (image, controls) = DiagramControllerOps::image_and_controls_rects(ui.max_rect());
            ui.allocate_rect(image, egui::Sense::hover());
            DiagramControllerOps::draw_controls(ui, state, controls);
            (image, controls)
        }
    }
}

fn click_zoom_buttons(
    harness: &mut Harness<'static>,
    state: &Mutex<ViewerState>,
    rects: &Arc<Mutex<Option<(egui::Rect, egui::Rect)>>>,
) {
    click_button(harness, rects, 2.0, 0.0);
    assert_eq!(state.lock().expect("viewer state lock").zoom, 1.75);
    click_button(harness, rects, 2.0, 2.0);
    assert_eq!(state.lock().expect("viewer state lock").zoom, 1.5);
}

fn click_reset_button(
    harness: &mut Harness<'static>,
    state: &Mutex<ViewerState>,
    rects: &Arc<Mutex<Option<(egui::Rect, egui::Rect)>>>,
) {
    click_button(harness, rects, 1.0, 1.0);
    let state = state.lock().expect("viewer state lock");
    assert_eq!(state.zoom, 1.0);
    assert_eq!(state.pan, egui::Vec2::ZERO);
}

fn assert_disjoint_image_controls(rects: &Arc<Mutex<Option<(egui::Rect, egui::Rect)>>>) {
    let (image, controls) = rects
        .lock()
        .expect("layout rectangles lock")
        .expect("image and controls rendered");
    assert_eq!(image.intersect(controls).area(), 0.0);
    assert!(controls.top() >= image.bottom());
}

fn click_button(
    harness: &mut Harness<'static>,
    rects: &Arc<Mutex<Option<(egui::Rect, egui::Rect)>>>,
    column: f32,
    row: f32,
) {
    let (_, controls) = rects
        .lock()
        .expect("layout lock")
        .expect("rendered controls");
    let extent = super::BUTTON_SIZE * super::GRID_DIM + super::GAP * (super::GRID_DIM - 1.0);
    /* WHY: 言語が並列testで変化しても、実描画したcontrolの位置へpointerを送る。 */
    let position = controls.right_bottom()
        - egui::vec2(super::MARGIN + extent, super::MARGIN + extent)
        + egui::vec2(column, row) * (super::BUTTON_SIZE + super::GAP)
        + egui::Vec2::splat(super::BUTTON_SIZE * BUTTON_CENTER_FRACTION);
    harness.event(egui::Event::PointerMoved(position));
    for pressed in [true, false] {
        harness.event(egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
        harness.step();
    }
}
