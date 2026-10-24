use std::path::Path;

use super::source::DocumentSurfaceSource;
use super::types::{DocumentFailure, DocumentSurface};

#[path = "source_revision_probe.rs"]
mod probe;
use probe::SourceRevisionProbe;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SourceRevision {
    pub(super) uri: String,
    pub(super) revision: String,
    generation: u64,
}

#[derive(Debug)]
pub(super) enum IntakeOutcome {
    Loaded(DocumentSurfaceSource),
    Unchanged(SourceRevision),
}

impl SourceRevision {
    pub(super) fn from_surface(surface: &DocumentSurface) -> Self {
        Self::from_source(surface.source(), surface.generation)
    }

    pub(super) fn from_source(source: &DocumentSurfaceSource, generation: u64) -> Self {
        Self {
            uri: source.uri.clone(),
            revision: source.revision.clone(),
            generation,
        }
    }

    pub(super) fn matches(&self, surface: Option<&DocumentSurface>) -> bool {
        surface.is_some_and(|surface| {
            self.generation == surface.generation
                && self.uri == surface.source().uri
                && self.revision == surface.source().revision
        })
    }
}

pub(super) struct SourceRevisionOps;

#[cfg(test)]
#[path = "source_revision_tests.rs"]
mod tests;

impl SourceRevisionOps {
    pub(super) fn load(
        path: &Path,
        known: Option<&SourceRevision>,
        cancelled: impl Fn() -> bool,
    ) -> Result<IntakeOutcome, DocumentFailure> {
        let _intake = super::worker_memory::DOCUMENT_WORKER_MEMORY.retain_intake();
        let checked = known
            .map(|known| SourceRevisionProbe::check(path, known, &cancelled))
            .transpose()?
            .flatten();
        if let (Some(known), Some(checked)) = (known, checked.as_ref())
            && known.revision == checked.revision
        {
            /* WHY: 更新確認だけの再読込では、文書サイズのVecを再確保せず既存sessionを維持する。 */
            return Ok(IntakeOutcome::Unchanged(known.clone()));
        }
        DocumentSurfaceSource::local_with_cancellation(path, &cancelled).map(IntakeOutcome::Loaded)
    }
}
