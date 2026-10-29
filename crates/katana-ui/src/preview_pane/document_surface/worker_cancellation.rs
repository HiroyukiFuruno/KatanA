use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug, Default)]
pub(super) struct DocumentWorkerCancellation(Arc<AtomicBool>);

impl DocumentWorkerCancellation {
    pub(super) fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub(super) fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::DocumentWorkerCancellation;

    #[test]
    fn cancellation_is_shared_only_with_the_owning_worker() {
        let owner = DocumentWorkerCancellation::default();
        let worker = owner.clone();
        let next_document = DocumentWorkerCancellation::default();
        owner.cancel();
        owner.cancel();
        assert!(worker.is_cancelled());
        assert!(!next_document.is_cancelled());
    }

    #[test]
    fn closing_surface_cancels_its_worker() {
        let (surface, _, _) = super::super::worker_tests::idle_surface();
        let worker = surface.cancellation.clone();
        drop(surface);
        assert!(worker.is_cancelled());
    }

    #[test]
    fn cancelled_worker_does_not_attempt_invalid_pdf_open() {
        assert!(run_invalid_pdf(false));
        assert!(!run_invalid_pdf(true));
    }

    #[test]
    fn cancelled_open_keeps_source_bytes_unconsumed() {
        let mut source = super::super::source::DocumentSurfaceSource::remote(
            "https://example.test/cancelled.pdf".to_owned(),
            Some("application/pdf"),
            b"%PDF-1.7\nnot opened".to_vec(),
        )
        .expect("source metadata");
        let original_bytes = source.byte_len();
        let cancellation = DocumentWorkerCancellation::default();
        cancellation.cancel();
        assert!(
            super::super::worker_session::open_session(&mut source, &cancellation)
                .expect("cancelled without an error")
                .is_none()
        );
        assert_eq!(source.byte_len(), original_bytes);
    }

    fn run_invalid_pdf(cancelled: bool) -> bool {
        let source = super::super::source::DocumentSurfaceSource::remote(
            "https://example.test/invalid.pdf".to_owned(),
            Some("application/pdf"),
            b"%PDF-1.7\ninvalid document".to_vec(),
        )
        .expect("source metadata");
        let (_commands, input) = std::sync::mpsc::channel();
        let (events, output) = std::sync::mpsc::channel();
        let cancellation = DocumentWorkerCancellation::default();
        if cancelled {
            cancellation.cancel();
        }
        super::super::worker::run(
            0,
            source,
            input,
            events,
            eframe::egui::Context::default(),
            cancellation,
        );
        output.try_iter().next().is_some()
    }
}
