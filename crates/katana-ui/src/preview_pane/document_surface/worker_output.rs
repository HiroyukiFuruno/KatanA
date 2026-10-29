use super::super::{
    painter_grid_borders::PreparedGridBorders, types::DocumentFailureLayer, worker_support::failure,
};
use super::{DocumentSurfaceSource, DocumentWorkerEvent};
use katana_document_viewer::{
    DocumentFrame, DocumentSession, DocumentSessionEvent, SpreadsheetFilterEvent,
};

pub(super) struct WorkerOutput {
    pub(super) events: std::sync::mpsc::Sender<DocumentWorkerEvent>,
    pub(super) repaint: eframe::egui::Context,
    pub(super) cancellation: super::super::worker_cancellation::DocumentWorkerCancellation,
}

impl WorkerOutput {
    pub(super) fn send_frame(
        &self,
        generation: u64,
        session: &mut DocumentSession,
        session_event: DocumentSessionEvent,
        filter_event: Option<SpreadsheetFilterEvent>,
        source: &DocumentSurfaceSource,
    ) -> bool {
        if self.cancellation.is_cancelled() {
            return false;
        }
        let Some(frame) = self.read_frame(generation, session, source) else {
            return false;
        };
        if self.cancellation.is_cancelled() {
            return false;
        }
        let Some(border_cache) = self.border_cache(generation, &frame, source) else {
            return false;
        };
        if self.cancellation.is_cancelled() {
            return false;
        }
        self.publish(
            generation,
            session,
            frame,
            border_cache,
            session_event,
            filter_event,
        )
    }

    fn read_frame(
        &self,
        generation: u64,
        session: &mut DocumentSession,
        source: &DocumentSurfaceSource,
    ) -> Option<DocumentFrame> {
        match session.frame() {
            Ok(frame) => Some(frame),
            Err(error) => {
                let failure = failure(source, "frame", DocumentFailureLayer::KdvSurface, error);
                super::super::worker_border_projection::send_failure(
                    generation,
                    failure,
                    &self.events,
                    &self.repaint,
                );
                None
            }
        }
    }

    fn border_cache(
        &self,
        generation: u64,
        frame: &DocumentFrame,
        source: &DocumentSurfaceSource,
    ) -> Option<PreparedGridBorders> {
        match super::super::worker_border_projection::project(frame, generation) {
            Ok(cache) => Some(cache),
            Err(error) => {
                let failure = failure(
                    source,
                    "frame border projection",
                    DocumentFailureLayer::KatanaHost,
                    error,
                );
                super::super::worker_border_projection::send_failure(
                    generation,
                    failure,
                    &self.events,
                    &self.repaint,
                );
                None
            }
        }
    }

    fn publish(
        &self,
        generation: u64,
        session: &mut DocumentSession,
        frame: DocumentFrame,
        border_cache: PreparedGridBorders,
        session_event: DocumentSessionEvent,
        filter_event: Option<SpreadsheetFilterEvent>,
    ) -> bool {
        let spreadsheet_metadata = session.spreadsheet_frame_metadata();
        let font_requests =
            super::super::font_requests::DocumentFontRequestsOps::project(&frame.surface);
        super::super::worker_border_projection::log_frame(generation, &frame);
        if self
            .events
            .send(DocumentWorkerEvent::Frame {
                generation,
                frame: Box::new(frame),
                border_cache,
                font_requests,
                session_event,
                spreadsheet_metadata,
                filter_event,
            })
            .is_err()
        {
            return false;
        }
        self.repaint
            .request_repaint_after(std::time::Duration::ZERO);
        true
    }
}
