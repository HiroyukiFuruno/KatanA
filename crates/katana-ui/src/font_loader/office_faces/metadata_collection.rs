use std::io::{Read, Seek};

use super::metadata_reader::{FontMetadataReader, SFNT_HEADER_BYTES};
use super::types::FontFaceResolutionDiagnostic;

const TTC_TAG_BYTES: usize = 4;
const TTC_MAJOR_OFFSET: usize = 4;
const TTC_MINOR_OFFSET: usize = 6;
const TTC_COUNT_OFFSET: usize = 8;
const U32_BYTES: usize = 4;
const TTC_DSIG_BYTES: u64 = 12;
const TTC_VERSION_TWO: u16 = 2;

pub(super) struct CollectionDirectory;

impl CollectionDirectory {
    pub(super) fn read<R: Read + Seek>(
        reader: &mut FontMetadataReader<'_, R>,
    ) -> Result<(Vec<u64>, bool), FontFaceResolutionDiagnostic> {
        let header = reader.read_at(0, SFNT_HEADER_BYTES)?;
        if &header[..TTC_TAG_BYTES] != b"ttcf" {
            return Ok((vec![0], false));
        }
        let count = Self::u32_at(&header, TTC_COUNT_OFFSET);
        let major = u16::from_be_bytes([header[TTC_MAJOR_OFFSET], header[TTC_MAJOR_OFFSET + 1]]);
        let minor = u16::from_be_bytes([header[TTC_MINOR_OFFSET], header[TTC_MINOR_OFFSET + 1]]);
        let offsets = if matches!(major, 1 | TTC_VERSION_TWO) && minor >= 1 {
            Self::extended(reader, count, major)?
        } else {
            Self::offsets(reader, SFNT_HEADER_BYTES as u64, count)?
        };
        Ok((offsets, true))
    }

    fn extended<R: Read + Seek>(
        reader: &mut FontMetadataReader<'_, R>,
        count: u32,
        major: u16,
    ) -> Result<Vec<u64>, FontFaceResolutionDiagnostic> {
        let legacy_bytes = u64::from(count) * U32_BYTES as u64;
        let dsig = if major == TTC_VERSION_TWO {
            TTC_DSIG_BYTES
        } else {
            0
        };
        let count_offset = SFNT_HEADER_BYTES as u64 + legacy_bytes + dsig;
        let bytes = reader.read_at(count_offset, U32_BYTES)?;
        Self::offsets(
            reader,
            count_offset + U32_BYTES as u64,
            Self::u32_at(&bytes, 0),
        )
    }

    fn offsets<R: Read + Seek>(
        reader: &mut FontMetadataReader<'_, R>,
        offset: u64,
        count: u32,
    ) -> Result<Vec<u64>, FontFaceResolutionDiagnostic> {
        let length = usize::try_from(count)
            .ok()
            .and_then(|count| count.checked_mul(U32_BYTES))
            .ok_or_else(|| reader.invalid())?;
        let bytes = reader.read_at(offset, length)?;
        Ok(bytes
            .as_chunks::<U32_BYTES>()
            .0
            .iter()
            .map(|value| u64::from(Self::u32_at(value, 0)))
            .collect())
    }

    fn u32_at(bytes: &[u8], offset: usize) -> u32 {
        u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + U32_BYTES - 1],
        ])
    }
}

#[cfg(test)]
#[path = "metadata_collection_tests.rs"]
mod tests;
