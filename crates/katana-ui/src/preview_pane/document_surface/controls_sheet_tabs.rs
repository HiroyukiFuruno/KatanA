use eframe::egui;
use katana_document_viewer::{DocumentViewerCommand, ViewerDocumentFormat};

#[path = "controls_sheet_tabs_logic.rs"]
mod controls_sheet_tabs_logic;

use super::types::DocumentSurface;
use super::worker::DocumentWorkerCommand;
use controls_sheet_tabs_logic::SheetTabLabelOps;

/* WHY: The horizontal scrollbar and sheet buttons need separate vertical space.
 * A 32px rail lets the scrollbar paint over long labels at the viewport edge. */
const SHEET_TAB_RAIL_HEIGHT: f32 = 44.0;

pub(super) const fn sheet_tab_rail_height(format: ViewerDocumentFormat) -> f32 {
    match format {
        ViewerDocumentFormat::Xlsx => SHEET_TAB_RAIL_HEIGHT,
        _ => 0.0,
    }
}

pub(super) fn show_sheet_tabs(
    surface: &mut DocumentSurface,
    ui: &mut egui::Ui,
    frame: &katana_document_viewer::DocumentFrame,
) {
    if frame.format != ViewerDocumentFormat::Xlsx {
        return;
    }
    egui::ScrollArea::horizontal()
        .id_salt(("document-sheet-tabs", surface.generation))
        .max_height(SHEET_TAB_RAIL_HEIGHT)
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                for (index, label) in
                    SheetTabLabelOps::labels(frame.surface.item_labels(), frame.state.item_count)
                {
                    let selected = index == frame.state.active_index;
                    let response = ui.add(
                        egui::Button::selectable(selected, label).frame_when_inactive(true),
                    );
                    if ui.input(|input| input.pointer.any_pressed() || input.pointer.any_released()) {
                        crate::debug_log::DebugLog::write(
                            "sheet_tab_pointer",
                            format_args!(
                                "index={index} enabled={} rect={:?} clip={:?} pointer={:?} hovered={} contains_pointer={} clicked={} id={:?} dragged_id={:?} down_on={} hit={:?}",
                                response.enabled(), response.rect, ui.clip_rect(),
                                ui.input(|input| input.pointer.hover_pos()), response.hovered(),
                                response.contains_pointer(), response.clicked(),
                                response.id, ui.ctx().dragged_id(), response.is_pointer_button_down_on(),
                                ui.ctx().interaction_snapshot(|state| (state.clicked, state.hovered.clone())),
                            ),
                        );
                    }
                    if response.clicked() {
                        surface.queue(DocumentWorkerCommand::Viewer(
                            DocumentViewerCommand::JumpTo(index),
                        ));
                    }
                }
            });
        });
}

#[cfg(test)]
mod tests {
    use egui_kittest::Harness;
    use egui_kittest::kittest::{NodeT, Queryable};
    use katana_document_viewer::{
        DocumentSession, DocumentSessionConfig, DocumentViewerCommand, DocumentViewport,
        OfficeWorkerConfig,
    };
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    #[test]
    fn xlsx_sheet_tab_stays_in_view_and_dispatches_jump_from_its_real_hit_target() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../scripts/screenshot/fixtures/v0-22-38-multi-format/representative.xlsx");
        let mut source = super::super::source::DocumentSurfaceSource::local(&fixture)
            .expect("representative xlsx source");
        let viewer_source = source.take_viewer_source().expect("viewer source");
        let worker = std::env::current_exe()
            .expect("test executable")
            .parent()
            .and_then(std::path::Path::parent)
            .expect("test profile directory")
            .join(if cfg!(windows) {
                "kdv-office-worker.exe"
            } else {
                "kdv-office-worker"
            });
        let config = DocumentSessionConfig::new(DocumentViewport::new(800, 240))
            .office_worker(OfficeWorkerConfig::new(worker));
        let mut session = DocumentSession::open(viewer_source, config).expect("xlsx session");
        let frame = session.frame().expect("xlsx frame");
        let click_frame = frame.clone();
        let border_cache =
            super::super::painter_grid_borders::prepare(&frame.surface).expect("xlsx border cache");
        let first_label = frame
            .surface
            .item_labels()
            .first()
            .cloned()
            .expect("first sheet label");

        let (command_tx, command_rx) = std::sync::mpsc::sync_channel(1);
        let (_event_tx, event_rx) = std::sync::mpsc::channel();
        let surface = Arc::new(Mutex::new(super::super::types::DocumentSurface {
            generation: 91,
            source: source.descriptor(),
            command_tx: Some(command_tx),
            event_rx,
            frame: Some(frame),
            border_cache,
            filter_ui: Default::default(),
            fonts: Default::default(),
            failure: None,
            painter: Default::default(),
            loading: false,
            command_in_flight: false,
            pending_commands: Default::default(),
            viewport: None,
            started_at: std::time::Instant::now(),
        }));
        let rendered_surface = Arc::clone(&surface);
        let mut harness = Harness::builder()
            .with_size(eframe::egui::vec2(800.0, 240.0))
            .build_ui(move |ui| {
                rendered_surface.lock().expect("surface lock").show(ui);
            });

        harness.step();
        let resize = command_rx.try_recv().expect("initial viewport resize");
        assert!(matches!(
            resize,
            super::super::worker::DocumentWorkerCommand::Surface(
                katana_document_viewer::DocumentSurfaceCommand::Resize(_)
            )
        ));
        let sheet_tab = harness.get_by_label(&first_label);
        let bounds = sheet_tab
            .accesskit_node()
            .raw_bounds()
            .expect("sheet tab bounds");
        assert!(
            bounds.y0 >= 180.0,
            "sheet tab must remain on the bottom rail"
        );
        assert!(
            bounds.y1 <= 240.0,
            "sheet tab must remain inside the viewport"
        );
        drop(harness);

        let (click_tx, click_rx) = std::sync::mpsc::sync_channel(1);
        let (_click_event_tx, click_event_rx) = std::sync::mpsc::channel();
        let click_surface = Arc::new(Mutex::new(super::super::types::DocumentSurface {
            generation: 92,
            source: source.descriptor(),
            command_tx: Some(click_tx),
            event_rx: click_event_rx,
            frame: None,
            border_cache: Default::default(),
            filter_ui: Default::default(),
            fonts: Default::default(),
            failure: None,
            painter: Default::default(),
            loading: false,
            command_in_flight: false,
            pending_commands: Default::default(),
            viewport: None,
            started_at: std::time::Instant::now(),
        }));
        let rendered_click_surface = Arc::clone(&click_surface);
        let mut click_harness = Harness::builder()
            .with_size(eframe::egui::vec2(800.0, 80.0))
            .build_ui(move |ui| {
                super::show_sheet_tabs(
                    &mut rendered_click_surface.lock().expect("surface lock"),
                    ui,
                    &click_frame,
                );
            });
        click_harness.get_by_label(&first_label).click();
        click_harness.step();

        assert_eq!(
            click_rx.try_recv().expect("sheet jump command"),
            super::super::worker::DocumentWorkerCommand::Viewer(DocumentViewerCommand::JumpTo(0))
        );
    }
}
