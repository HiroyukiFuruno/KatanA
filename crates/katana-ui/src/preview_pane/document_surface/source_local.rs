use std::path::Path;

use katana_core::document_source::{BinaryDocumentFormat, MAX_BINARY_DOCUMENT_BYTES};

use super::super::super::cancellable_read::{CancellableReader, ReadCancellation};
use super::super::debug_log::DebugLog;
use super::super::source_io::{file_url, opened_file_capacity, read_limited, revision};
use super::super::types::DocumentFailure;
use super::super::worker_memory::DOCUMENT_WORKER_MEMORY;
use super::DocumentSurfaceSource;

pub(super) struct SourceLocalOps;

impl SourceLocalOps {
    pub(super) fn load<F: Fn() -> bool>(
        path: &Path,
        cancellation: &ReadCancellation<F>,
    ) -> Result<DocumentSurfaceSource, DocumentFailure> {
        check(cancellation, path, None)?;
        let intake_memory = std::sync::Arc::new(DOCUMENT_WORKER_MEMORY.retain_intake());
        let started_at = std::time::Instant::now();
        let canonical = path.canonicalize().map_err(|error| {
            DocumentFailure::intake("canonicalize", path, None, error.to_string())
        })?;
        check(cancellation, &canonical, None)?;
        let canonical_ms = started_at.elapsed().as_millis();
        let format = BinaryDocumentFormat::from_path(&canonical).ok_or_else(|| {
            DocumentFailure::intake(
                "classify",
                &canonical,
                None,
                "document extension is not supported",
            )
        })?;
        check(cancellation, &canonical, Some(format))?;
        let bytes = Self::read(&canonical, Some(format), cancellation)?;
        let read_ms = started_at.elapsed().as_millis();
        check(cancellation, &canonical, Some(format))?;
        BinaryDocumentFormat::detect(&canonical, Some(format.mime()), &bytes).map_err(|error| {
            DocumentFailure::intake("validate", &canonical, Some(format), error.to_string())
        })?;
        check(cancellation, &canonical, Some(format))?;
        let validate_ms = started_at.elapsed().as_millis();
        let uri = file_url(&canonical, format)?;
        check(cancellation, &canonical, Some(format))?;
        let revision = revision(&bytes);
        check(cancellation, &canonical, Some(format))?;
        let source = DocumentSurfaceSource {
            uri,
            format,
            mime: format.mime().to_owned(),
            revision,
            bytes,
            _intake_memory: Some(intake_memory),
        };
        DebugLog::write(
            "document_source_intake",
            format_args!(
                "kind=local format={} bytes={} elapsed_ms={} canonical_ms={} read_ms={} validate_ms={} uri={}",
                source.format.extension(),
                source.bytes.len(),
                started_at.elapsed().as_millis(),
                canonical_ms,
                read_ms.saturating_sub(canonical_ms),
                validate_ms.saturating_sub(read_ms),
                source.uri
            ),
        );
        Ok(source)
    }

    fn read<F: Fn() -> bool>(
        canonical: &Path,
        format: Option<BinaryDocumentFormat>,
        cancellation: &ReadCancellation<F>,
    ) -> Result<Vec<u8>, DocumentFailure> {
        let started_at = std::time::Instant::now();
        check(cancellation, canonical, format)?;
        let file = std::fs::File::open(canonical).map_err(|error| {
            DocumentFailure::intake("read", canonical, format, error.to_string())
        })?;
        let opened_at = std::time::Instant::now();
        check(cancellation, canonical, format)?;
        let capacity = opened_file_capacity(
            file.metadata(),
            canonical,
            format,
            MAX_BINARY_DOCUMENT_BYTES.saturating_add(1),
        )?;
        check(cancellation, canonical, format)?;
        let result = read_limited(
            CancellableReader::new(file, cancellation),
            canonical,
            format,
            MAX_BINARY_DOCUMENT_BYTES,
            capacity,
        );
        DebugLog::write(
            "document_file_read",
            format_args!(
                "open_ms={} read_ms={} success={} path={}",
                opened_at.duration_since(started_at).as_millis(),
                opened_at.elapsed().as_millis(),
                result.is_ok(),
                canonical.display()
            ),
        );
        let bytes = result?;
        check(cancellation, canonical, format)?;
        Ok(bytes)
    }
}

fn check<F: Fn() -> bool>(
    cancellation: &ReadCancellation<F>,
    path: &Path,
    format: Option<BinaryDocumentFormat>,
) -> Result<(), DocumentFailure> {
    cancellation
        .check()
        .map_err(|error| DocumentFailure::intake("read", path, format, error.to_string()))
}
