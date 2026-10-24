use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use super::metadata_collection::CollectionDirectory;
use super::metadata_tables::MetadataTables;
use super::types::{FONT_READ_CHUNK_BYTES, FontFaceResolutionDiagnostic, MAX_FONT_PAYLOAD_BYTES};

pub(super) const SFNT_HEADER_BYTES: usize = 12;
pub(super) const SFNT_TABLE_BYTES: usize = 16;
const SFNT_TABLE_COUNT_OFFSET: usize = 4;

pub(super) struct FontMetadataReader<'a, R> {
    input: &'a mut R,
    path: &'a Path,
    size: u64,
    cancelled: &'a AtomicBool,
    collection: bool,
}

impl<'a, R: Read + Seek> FontMetadataReader<'a, R> {
    pub(super) fn new(
        input: &'a mut R,
        path: &'a Path,
        size: u64,
        cancelled: &'a AtomicBool,
    ) -> Self {
        Self {
            input,
            path,
            size,
            cancelled,
            collection: false,
        }
    }

    pub(super) fn offsets(&mut self) -> Result<Vec<u64>, FontFaceResolutionDiagnostic> {
        let (offsets, collection) = CollectionDirectory::read(self)?;
        self.collection = collection;
        Ok(offsets)
    }

    pub(super) fn face(
        &mut self,
        offset: u64,
        index: usize,
    ) -> Result<Vec<u8>, FontFaceResolutionDiagnostic> {
        self.read_face(offset).map_err(|error| match error {
            FontFaceResolutionDiagnostic::InvalidFontFile { .. } if self.collection => {
                FontFaceResolutionDiagnostic::InvalidFontFace {
                    path: self.path.to_owned(),
                    face_index: index,
                }
            }
            other => other,
        })
    }

    fn read_face(&mut self, offset: u64) -> Result<Vec<u8>, FontFaceResolutionDiagnostic> {
        let header = self.read_at(offset, SFNT_HEADER_BYTES)?;
        let count = u16::from_be_bytes([
            header[SFNT_TABLE_COUNT_OFFSET],
            header[SFNT_TABLE_COUNT_OFFSET + 1],
        ]) as usize;
        let length = count
            .checked_mul(SFNT_TABLE_BYTES)
            .and_then(|bytes| bytes.checked_add(SFNT_HEADER_BYTES))
            .ok_or_else(|| self.invalid())?;
        let available = self.size.saturating_sub(offset).min(length as u64) as usize;
        let directory = self.read_at(offset, available)?;
        MetadataTables::read(self, &directory)
    }

    pub(super) fn read_at(
        &mut self,
        offset: u64,
        length: usize,
    ) -> Result<Vec<u8>, FontFaceResolutionDiagnostic> {
        self.check_cancelled()?;
        if offset
            .checked_add(length as u64)
            .is_none_or(|end| end > self.size)
            || length as u64 > MAX_FONT_PAYLOAD_BYTES
        {
            return Err(self.invalid());
        }
        self.input
            .seek(SeekFrom::Start(offset))
            .map_err(|error| self.unreadable(error))?;
        let mut bytes = vec![0; length];
        for chunk in bytes.chunks_mut(FONT_READ_CHUNK_BYTES) {
            self.check_cancelled()?;
            self.input
                .read_exact(chunk)
                .map_err(|error| self.unreadable(error))?;
        }
        self.check_cancelled()?;
        Ok(bytes)
    }

    pub(super) fn range_exists(&self, offset: u64, length: u64) -> bool {
        offset != 0
            && offset
                .checked_add(length)
                .is_some_and(|end| end <= self.size)
    }

    pub(super) fn invalid(&self) -> FontFaceResolutionDiagnostic {
        FontFaceResolutionDiagnostic::InvalidFontFile {
            path: self.path.to_owned(),
        }
    }

    fn unreadable(&self, error: std::io::Error) -> FontFaceResolutionDiagnostic {
        FontFaceResolutionDiagnostic::FileUnreadable {
            path: self.path.to_owned(),
            kind: error.kind(),
        }
    }

    fn check_cancelled(&self) -> Result<(), FontFaceResolutionDiagnostic> {
        if self.cancelled.load(Ordering::Acquire) {
            Err(FontFaceResolutionDiagnostic::Cancelled)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "metadata_reader_tests.rs"]
mod tests;
