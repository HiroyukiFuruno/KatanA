use std::fs::File;
use std::io::Read;
use std::path::Path;

use katana_core::document_source::{BinaryDocumentFormat, MAX_BINARY_DOCUMENT_BYTES};
use sha2::{Digest, Sha256};

use super::super::super::cancellable_read::{CancellableReader, ReadCancellation};
use super::super::source_io::{enforce_size_limit, file_url, revision_digest};
use super::super::types::DocumentFailure;
use super::SourceRevision;

const REVISION_CHUNK_BYTES: usize = 64 * 1024;

pub(super) struct SourceRevisionProbe {
    pub(super) revision: String,
}

impl SourceRevisionProbe {
    pub(super) fn check(
        path: &Path,
        known: &SourceRevision,
        cancelled: impl Fn() -> bool,
    ) -> Result<Option<Self>, DocumentFailure> {
        let cancellation = ReadCancellation::new(cancelled);
        Self::check_cancelled(path, &cancellation)?;
        let canonical = path.canonicalize().map_err(|error| {
            DocumentFailure::intake("canonicalize", path, None, error.to_string())
        })?;
        let format = BinaryDocumentFormat::from_path(&canonical);
        let Some(format) = format else {
            return Ok(None);
        };
        let uri = file_url(&canonical, format)?;
        if uri != known.uri {
            return Ok(None);
        }
        let Some(mut file) = Self::open_regular(&canonical)? else {
            return Ok(None);
        };
        let (revision, bytes) = Self::hash(
            CancellableReader::new(&mut file, &cancellation),
            path,
            MAX_BINARY_DOCUMENT_BYTES,
        )?;
        Self::check_cancelled(path, &cancellation)?;
        Self::log(&uri, &revision, &known.revision, bytes);
        Ok(Some(Self { revision }))
    }

    fn open_regular(path: &Path) -> Result<Option<File>, DocumentFailure> {
        if !path
            .metadata()
            .map_err(|error| Self::failure(path, error))?
            .is_file()
        {
            return Ok(None);
        }
        let file = File::open(path).map_err(|error| Self::failure(path, error))?;
        if !file
            .metadata()
            .map_err(|error| Self::failure(path, error))?
            .is_file()
        {
            /* WHY: 特殊fileへの差替後に再openすると単一writerを消費して待ち続けるため、失敗を返す。 */
            return Err(Self::failure(
                path,
                "document changed from a regular file during revision check",
            ));
        }
        Ok(Some(file))
    }

    fn hash(
        reader: impl Read,
        path: &Path,
        limit: usize,
    ) -> Result<(String, usize), DocumentFailure> {
        let read_limit = limit.checked_add(1).ok_or_else(|| {
            Self::failure(
                path,
                "document size limit cannot include an overflow sentinel",
            )
        })?;
        let mut reader = reader.take(read_limit as u64);
        let mut hash = Sha256::new();
        let mut buffer = [0; REVISION_CHUNK_BYTES];
        let mut total = 0;
        loop {
            let read = match reader.read(&mut buffer) {
                Ok(read) => read,
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(Self::failure(path, error)),
            };
            if read == 0 {
                break;
            }
            total += read;
            hash.update(&buffer[..read]);
        }
        enforce_size_limit(path, BinaryDocumentFormat::from_path(path), total, limit)?;
        Ok((revision_digest(&hash.finalize()), total))
    }

    fn check_cancelled<F: Fn() -> bool>(
        path: &Path,
        cancellation: &ReadCancellation<F>,
    ) -> Result<(), DocumentFailure> {
        cancellation
            .check()
            .map_err(|error| Self::failure(path, error))
    }

    fn failure(path: &Path, error: impl std::fmt::Display) -> DocumentFailure {
        DocumentFailure::intake(
            "read",
            path,
            BinaryDocumentFormat::from_path(path),
            error.to_string(),
        )
    }

    fn log(uri: &str, revision: &str, expected: &str, bytes: usize) {
        let at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |time| time.as_micros());
        let outcome = if revision == expected {
            "unchanged"
        } else {
            "changed"
        };
        super::super::debug_log::DebugLog::write(
            "document_source_revision",
            format_args!(
                "pid={} timestamp_unix_us={at} uri={uri} revision={revision} expected_revision={expected} outcome={outcome} bytes={bytes} passes=1 payload_bytes=0",
                std::process::id()
            ),
        );
    }
}

#[cfg(test)]
#[path = "source_revision_probe_tests.rs"]
mod tests;
