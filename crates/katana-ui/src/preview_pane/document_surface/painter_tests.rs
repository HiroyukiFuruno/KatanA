use std::path::PathBuf;

use eframe::egui;
use katana_document_viewer::{
    DocumentFrame, DocumentGridCell, DocumentGridCellAppearance, DocumentGridCoordinate,
    DocumentGridSurfaceFrame, DocumentGridViewport, DocumentRect, DocumentSession,
    DocumentSessionConfig, DocumentSurfaceCommand, DocumentSurfaceKind, DocumentViewport,
    OfficeWorkerConfig,
};

use super::painter::{DocumentFramePainter, paint_document_frame};
use super::source::DocumentSurfaceSource;

const VIEWPORT_WIDTH: u32 = 960;
const VIEWPORT_HEIGHT: u32 = 640;
const CELL_OFFSET: i32 = 10;
const CELL_WIDTH: u32 = 120;
const CELL_HEIGHT: u32 = 32;
const GRID_SCROLL_X: u32 = 20;
const GRID_SCROLL_Y: u32 = 30;
const TEST_SURFACE_ID: u64 = 41;
const FALLBACK_SURFACE_ID: u64 = 42;

fn input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(VIEWPORT_WIDTH as f32, VIEWPORT_HEIGHT as f32),
        )),
        ..Default::default()
    }
}

pub(super) fn grid_cell() -> DocumentGridCell {
    DocumentGridCell {
        coordinate: DocumentGridCoordinate { row: 0, column: 0 },
        bounds: DocumentRect {
            x: CELL_OFFSET,
            y: CELL_OFFSET,
            width: CELL_WIDTH,
            height: CELL_HEIGHT,
        },
        clipped_bounds: DocumentRect {
            x: CELL_OFFSET,
            y: CELL_OFFSET,
            width: CELL_WIDTH,
            height: CELL_HEIGHT,
        },
        text: "42".to_owned(),
        appearance: DocumentGridCellAppearance::default(),
        row_span: 1,
        column_span: 1,
        selected: false,
        active: false,
        frozen_row: false,
        frozen_column: false,
        accessibility_row_index: 1,
        accessibility_column_index: 1,
    }
}

pub(super) fn grid_surface() -> DocumentGridSurfaceFrame {
    DocumentGridSurfaceFrame {
        row_count: 1,
        column_count: 1,
        total_width: CELL_WIDTH,
        total_height: CELL_HEIGHT,
        viewport: DocumentGridViewport {
            width: VIEWPORT_WIDTH,
            height: VIEWPORT_HEIGHT,
            scroll_x: GRID_SCROLL_X,
            scroll_y: GRID_SCROLL_Y,
        },
        active_cell: Some(DocumentGridCoordinate { row: 0, column: 0 }),
        show_grid_lines: true,
        cells: vec![grid_cell()],
    }
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/screenshot/fixtures/v0-22-38-multi-format")
        .join(name)
}

fn office_worker() -> PathBuf {
    if let Some(path) = std::env::var_os("KATANA_KDV_OFFICE_WORKER") {
        return PathBuf::from(path);
    }
    let executable = std::env::current_exe().expect("test executable");
    executable
        .parent()
        .and_then(std::path::Path::parent)
        .expect("cargo target profile directory")
        .join(if cfg!(windows) {
            "kdv-office-worker.exe"
        } else {
            "kdv-office-worker"
        })
}

fn frame(name: &str) -> DocumentFrame {
    let mut source = DocumentSurfaceSource::local(&fixture(name)).expect("document fixture");
    let viewer_source = source.take_viewer_source().expect("KDV viewer source");
    let viewport = DocumentViewport::new(VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
    let mut config = DocumentSessionConfig::new(viewport);
    if name.ends_with(".xlsx") {
        let worker = office_worker();
        assert!(worker.is_file(), "office worker is missing: {worker:?}");
        config = config.office_worker(OfficeWorkerConfig::new(worker));
    }
    let mut session = DocumentSession::open(viewer_source, config).expect("document session");
    session.frame().expect("document frame")
}

fn paint(frame: &DocumentFrame, painter: &mut DocumentFramePainter) -> Vec<DocumentSurfaceCommand> {
    let context = crate::test_ui::Context::default();
    let mut commands = Vec::new();
    let output = context.run_ui(input(), |ui| {
        commands = paint_document_frame(painter, ui, &frame.surface, TEST_SURFACE_ID);
    });
    assert!(!output.shapes.is_empty());
    commands
}

#[test]
fn page_frame_is_projected_without_owning_pdf_layout() {
    let frame = frame("representative.pdf");
    assert_eq!(frame.surface.kind(), DocumentSurfaceKind::Page);
    let fingerprint = frame
        .surface
        .page()
        .expect("page surface")
        .fingerprint
        .clone();
    let mut painter = DocumentFramePainter::default();

    let first = paint(&frame, &mut painter);
    let first_texture = painter.texture.as_ref().expect("page texture").id();
    let second = paint(&frame, &mut painter);
    let context = crate::test_ui::Context::default();
    let mut fallback = Vec::new();
    context.run_ui(input(), |ui| {
        fallback = super::painter_grid::paint(ui, &frame.surface);
    });

    assert!(matches!(
        first.as_slice(),
        [DocumentSurfaceCommand::Resize(_)]
    ));
    assert_eq!(first, second);
    assert_eq!(
        painter.texture_fingerprint.as_deref(),
        Some(fingerprint.as_str())
    );
    assert_eq!(
        painter.texture.as_ref().expect("reused texture").id(),
        first_texture
    );
    assert!(matches!(
        fallback.as_slice(),
        [DocumentSurfaceCommand::Resize(_)]
    ));
    assert!(format!("{painter:?}").contains(&fingerprint));
}

#[test]
fn grid_frame_is_projected_without_owning_spreadsheet_layout_or_hit_test() {
    if !office_worker().is_file() {
        return;
    }
    let frame = frame("representative.xlsx");
    assert_eq!(frame.surface.kind(), DocumentSurfaceKind::Grid);
    assert!(!frame.surface.grid().expect("grid surface").cells.is_empty());

    let commands = paint(&frame, &mut DocumentFramePainter::default());
    let context = crate::test_ui::Context::default();
    let mut fallback = Vec::new();
    context.run_ui(input(), |ui| {
        fallback = super::painter_page::paint(
            &mut DocumentFramePainter::default(),
            ui,
            &frame.surface,
            FALLBACK_SURFACE_ID,
        );
    });

    assert!(matches!(
        commands.first(),
        Some(DocumentSurfaceCommand::Resize(_))
    ));
    assert!(matches!(
        fallback.as_slice(),
        [DocumentSurfaceCommand::Resize(_)]
    ));
}
