use eframe::egui;
use katana_document_viewer::{DocumentGridCommand, DocumentGridNavigation, DocumentSurfaceCommand};

use super::super::painter_tests::grid_surface;

const TEST_VIEWPORT_WIDTH: f32 = 300.0;
const TEST_VIEWPORT_HEIGHT: f32 = 200.0;
const POINTER_COORDINATE: f32 = 50.0;
const POINTER_COORDINATE_I32: i32 = 50;
const DRAG_X: f32 = 70.0;
const DRAG_Y: f32 = 60.0;
const OFFSET_DELTA: f32 = 10.0;
const EXPECTED_SCROLL_X: u32 = 25;
const EXPECTED_SCROLL_Y: u32 = 23;

fn input(events: Vec<egui::Event>) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(TEST_VIEWPORT_WIDTH, TEST_VIEWPORT_HEIGHT),
        )),
        events,
        ..Default::default()
    }
}

fn run_commands(
    context: &crate::test_ui::Context,
    events: Vec<egui::Event>,
) -> Vec<DocumentSurfaceCommand> {
    let mut commands = Vec::new();
    context.run_ui(input(events), |ui| {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
        commands = super::commands(ui, rect, &response, &grid_surface());
    });
    commands
}

fn run_focused_commands(
    context: &crate::test_ui::Context,
    events: Vec<egui::Event>,
) -> Vec<DocumentSurfaceCommand> {
    let mut commands = Vec::new();
    context.run_ui(input(events), |ui| {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
        response.request_focus();
        commands = super::commands(ui, rect, &response, &grid_surface());
    });
    commands
}

fn pointer_button(pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos: egui::pos2(POINTER_COORDINATE, POINTER_COORDINATE),
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}

#[test]
fn grid_commands_cover_selection_scroll_drag_and_navigation() {
    let context = crate::test_ui::Context::default();
    run_commands(
        &context,
        vec![egui::Event::PointerMoved(egui::pos2(
            POINTER_COORDINATE,
            POINTER_COORDINATE,
        ))],
    );
    run_commands(&context, vec![pointer_button(true)]);
    let clicked = run_commands(&context, vec![pointer_button(false)]);
    assert!(clicked.iter().any(|command| matches!(
        command,
        DocumentSurfaceCommand::Grid(DocumentGridCommand::SelectAt { x, y, .. })
            if *x == POINTER_COORDINATE_I32 && *y == POINTER_COORDINATE_I32
    )));

    let scrolled = run_commands(
        &context,
        vec![
            egui::Event::PointerMoved(egui::pos2(POINTER_COORDINATE, POINTER_COORDINATE)),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(-5.0, 7.0),
                modifiers: egui::Modifiers::NONE,
                phase: egui::TouchPhase::Move,
            },
        ],
    );
    assert!(scrolled.iter().any(|command| matches!(
        command,
        DocumentSurfaceCommand::Grid(DocumentGridCommand::ScrollTo { .. })
    )));

    run_commands(&context, vec![pointer_button(true)]);
    let dragged = run_commands(
        &context,
        vec![egui::Event::PointerMoved(egui::pos2(DRAG_X, DRAG_Y))],
    );
    assert!(dragged.iter().any(|command| matches!(
        command,
        DocumentSurfaceCommand::Grid(DocumentGridCommand::ScrollTo { .. })
    )));
    run_commands(&context, vec![pointer_button(false)]);

    let mut shift = egui::Modifiers::NONE;
    shift.shift = true;
    let navigated = run_focused_commands(
        &context,
        vec![
            egui::Event::ModifiersChanged(shift),
            egui::Event::Key {
                key: egui::Key::ArrowRight,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: shift,
            },
        ],
    );
    assert!(navigated.iter().any(|command| matches!(
        command,
        DocumentSurfaceCommand::Grid(DocumentGridCommand::Navigate {
            intent: DocumentGridNavigation::Right,
            extend: true,
        })
    )));
}

#[test]
fn grid_helpers_cover_saturation_and_every_navigation_mapping() {
    assert_eq!(super::offset(u32::MAX, OFFSET_DELTA), u32::MAX);
    assert_eq!(super::offset(2, -OFFSET_DELTA), 0);
    assert!(matches!(
        super::scroll_to(&grid_surface(), egui::vec2(-5.0, 7.0)),
        DocumentSurfaceCommand::Grid(DocumentGridCommand::ScrollTo {
            x: EXPECTED_SCROLL_X,
            y: EXPECTED_SCROLL_Y,
        })
    ));

    for (key, expected) in [
        (egui::Key::ArrowLeft, DocumentGridNavigation::Left),
        (egui::Key::ArrowRight, DocumentGridNavigation::Right),
        (egui::Key::ArrowUp, DocumentGridNavigation::Up),
        (egui::Key::ArrowDown, DocumentGridNavigation::Down),
        (egui::Key::Home, DocumentGridNavigation::Home),
        (egui::Key::End, DocumentGridNavigation::End),
        (egui::Key::PageUp, DocumentGridNavigation::PageUp),
        (egui::Key::PageDown, DocumentGridNavigation::PageDown),
    ] {
        let context = crate::test_ui::Context::default();
        let mut actual = None;
        context.run_ui(
            input(vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }]),
            |ui| actual = super::navigation_intent(ui),
        );
        assert_eq!(actual, Some(expected));
    }
}
