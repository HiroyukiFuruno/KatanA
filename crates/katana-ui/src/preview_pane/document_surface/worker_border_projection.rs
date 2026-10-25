use eframe::egui;
use katana_document_viewer::DocumentFrame;
use std::sync::mpsc::Sender;

use super::painter_grid_borders::{BorderPreparationError, PreparedGridBorders};
use super::types::DocumentFailure;

pub(super) fn project(
    frame: &DocumentFrame,
    generation: u64,
) -> Result<PreparedGridBorders, BorderPreparationError> {
    let started = std::time::Instant::now();
    let cache = super::painter_grid_borders::prepare(&frame.surface)?;
    super::debug_log::DebugLog::write(
        "document_grid_border_projection",
        format_args!(
            "generation={} cells={} elapsed_us={} owned_bytes={}",
            generation,
            cache.cells.len(),
            started.elapsed().as_micros(),
            cache.cells.capacity()
                * std::mem::size_of::<super::painter_grid_borders::PreparedCellBorders>()
        ),
    );
    Ok(cache)
}

pub(super) fn log_frame(generation: u64, frame: &DocumentFrame) {
    tracing::debug!(
        generation,
        format = super::worker_support::format_extension(frame.format),
        active_index = frame.state.active_index,
        item_count = frame.state.item_count,
        surface = ?frame.surface.kind(),
        "produced document frame"
    );
}

pub(super) fn send_failure(
    generation: u64,
    failure: DocumentFailure,
    events: &Sender<super::worker::DocumentWorkerEvent>,
    repaint: &egui::Context,
) {
    let _ = events.send(super::worker::DocumentWorkerEvent::Failure {
        generation,
        failure,
    });
    repaint.request_repaint_after(std::time::Duration::ZERO);
}
