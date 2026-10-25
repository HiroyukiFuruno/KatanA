use super::super::source_revision::IntakeOutcome;
use super::{LocalDocumentIntake, PreviewPane, request};

impl PreviewPane {
    pub(super) fn finish_document_intake(
        &mut self,
        pending: LocalDocumentIntake,
        result: request::IntakeResult,
    ) {
        self.is_loading = false;
        match result {
            Ok(IntakeOutcome::Loaded(source)) => {
                self.full_render_document_source(source, pending.force)
            }
            Ok(IntakeOutcome::Unchanged(known)) => {
                if pending.force || !known.matches(self.document_surface.as_ref()) {
                    self.full_render_document_path(&pending.path, pending.force);
                    return;
                }
            }
            Err(error) => self.full_render_document_failure(error),
        }
        if pending.refresh_requested {
            self.full_render_document_path(&pending.path, false);
        }
    }
}
