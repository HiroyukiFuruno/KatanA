use super::render_support::PendingDocumentCommands;
use super::source::DocumentSurfaceSource;
use super::types::{DocumentFailure, DocumentFailureLayer, DocumentSurface};
use super::worker;
use eframe::egui;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);

impl DocumentSurface {
    pub(crate) fn start(source: DocumentSurfaceSource, ctx: &egui::Context) -> Self {
        let started_at = std::time::Instant::now();
        let generation = NEXT_GENERATION.fetch_add(1, Ordering::Relaxed);
        let (command_tx, command_rx) = std::sync::mpsc::sync_channel(1);
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let display_source = source.descriptor();
        super::debug_log::DebugLog::write(
            "document_worker_handoff",
            format_args!(
                "generation={} format={} bytes={} uri={}",
                generation,
                source.format.extension(),
                source.byte_len(),
                source.uri
            ),
        );
        let cancellation = super::worker_cancellation::DocumentWorkerCancellation::default();
        let worker_cancellation = cancellation.clone();
        let worker_source = source;
        let repaint = ctx.clone();
        let spawn = super::DocumentWorkerLifecycle::spawn(
            format!("katana-document-{generation}"),
            move || {
                worker::run(
                    generation,
                    worker_source,
                    command_rx,
                    event_tx,
                    repaint,
                    worker_cancellation,
                );
            },
        );
        let failure = match spawn {
            Ok(_worker) => None,
            Err(error) => {
                let failure = DocumentFailure::new(
                    DocumentFailureLayer::KatanaHost,
                    "start worker",
                    display_source.uri.clone(),
                    Some(display_source.format),
                    error.to_string(),
                );
                failure.log();
                Some(failure)
            }
        };
        let started = failure.is_none();
        Self {
            generation,
            source: display_source,
            cancellation,
            command_tx: Some(command_tx),
            event_rx,
            frame: None,
            border_cache: Default::default(),
            filter_ui: Default::default(),
            fonts: Default::default(),
            failure,
            painter: Default::default(),
            loading: started,
            command_in_flight: started,
            pending_commands: PendingDocumentCommands::default(),
            viewport: None,
            started_at,
        }
    }
}
