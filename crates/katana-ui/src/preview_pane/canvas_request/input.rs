use std::io::Read;
use std::path::{Path, PathBuf};

use katana_core::document_source::MAX_BINARY_DOCUMENT_BYTES;

use super::super::cancellable_read::{CancellableReader, ReadCancellation};
use super::CanvasRequestError;

pub(super) enum CanvasInput {
    Text(String),
    File(PathBuf),
}

impl From<String> for CanvasInput {
    fn from(source: String) -> Self {
        Self::Text(source)
    }
}

impl CanvasInput {
    pub(super) fn load(self, cancelled: impl Fn() -> bool) -> Result<String, CanvasRequestError> {
        if cancelled() {
            return Err(CanvasRequestError::Cancelled);
        }
        let source = match self {
            Self::Text(source) => source,
            Self::File(path) => CanvasFileInput::load(&path, &cancelled)?,
        };
        if cancelled() {
            return Err(CanvasRequestError::Cancelled);
        }
        Ok(source)
    }
}

struct CanvasFileInput;

impl CanvasFileInput {
    fn load(path: &Path, cancelled: impl Fn() -> bool) -> Result<String, CanvasRequestError> {
        let cancellation = ReadCancellation::new(cancelled);
        Self::check(&cancellation)?;
        let metadata = Self::io_result(std::fs::metadata(path), &cancellation)?;
        /* WHY: FIFO・端末などを開く前に除外し、固定ワーカーを無期限に占有させない。 */
        Self::validate_metadata(&metadata)?;
        Self::check(&cancellation)?;
        let file = Self::io_result(std::fs::File::open(path), &cancellation)?;
        let metadata = Self::io_result(file.metadata(), &cancellation)?;
        Self::validate_metadata(&metadata)?;
        Self::check(&cancellation)?;
        let mut bytes = Vec::new();
        let read_limit = MAX_BINARY_DOCUMENT_BYTES.saturating_add(1) as u64;
        let read = CancellableReader::new(file, &cancellation)
            .take(read_limit)
            .read_to_end(&mut bytes);
        Self::io_result(read, &cancellation)?;
        if bytes.len() > MAX_BINARY_DOCUMENT_BYTES {
            return Err(CanvasRequestError::SizeLimit);
        }
        String::from_utf8(bytes).map_err(|_| CanvasRequestError::InvalidUtf8)
    }

    fn validate_metadata(metadata: &std::fs::Metadata) -> Result<(), CanvasRequestError> {
        if !metadata.is_file() {
            return Err(CanvasRequestError::NotRegularFile);
        }
        if metadata.len() > MAX_BINARY_DOCUMENT_BYTES as u64 {
            return Err(CanvasRequestError::SizeLimit);
        }
        Ok(())
    }

    fn check<F: Fn() -> bool>(
        cancellation: &ReadCancellation<F>,
    ) -> Result<(), CanvasRequestError> {
        cancellation
            .check()
            .map_err(|_| CanvasRequestError::Cancelled)
    }

    fn io_result<T, F: Fn() -> bool>(
        result: std::io::Result<T>,
        cancellation: &ReadCancellation<F>,
    ) -> Result<T, CanvasRequestError> {
        Self::check(cancellation)?;
        result.map_err(|error| CanvasRequestError::FileRead(error.kind()))
    }
}
