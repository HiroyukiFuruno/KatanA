use std::collections::BTreeSet;

use skrifa::FontRef;
use write_fonts::FontBuilder;

use super::types::MAX_FONT_PAYLOAD_BYTES;

const SFNT_HEADER_LENGTH: u64 = 12;
const TABLE_RECORD_LENGTH: usize = 16;
const TABLE_ALIGNMENT: u64 = 4;
const HEAD_CHECKSUM_END: usize = 12;

pub(super) struct FacePayload {
    bytes: Option<Vec<u8>>,
}

impl FacePayload {
    pub(super) fn new(bytes: Vec<u8>) -> Self {
        Self { bytes: Some(bytes) }
    }

    pub(super) fn take_face(&mut self, index: u32) -> Result<egui::FontData, ()> {
        let bytes = self.bytes.as_ref().ok_or(())?;
        let font = FontRef::from_index(bytes, index).map_err(|_| ())?;
        if font.ttc_index().is_none() {
            return self.bytes.take().map(egui::FontData::from_owned).ok_or(());
        }
        let standalone = extract_face(&font)?;
        Ok(egui::FontData::from_owned(standalone))
    }
}

fn extract_face(font: &FontRef<'_>) -> Result<Vec<u8>, ()> {
    let records = font.table_directory().table_records();
    if records.len() > usize::from(u16::MAX) / TABLE_RECORD_LENGTH {
        return Err(());
    }
    let mut size = SFNT_HEADER_LENGTH + records.len() as u64 * TABLE_RECORD_LENGTH as u64;
    let mut seen = BTreeSet::new();
    let mut builder = FontBuilder::new();
    for record in records {
        let tag = record.tag();
        if !seen.insert(tag) {
            return Err(());
        }
        let data = font.table_data(tag).ok_or(())?;
        let bytes = data.as_bytes();
        size = checked_face_size(size, bytes.len())?;
        if tag == skrifa::raw::types::Tag::new(b"head") && bytes.len() < HEAD_CHECKSUM_END {
            return Err(());
        }
        builder.add_raw(tag, bytes);
    }
    Ok(builder.build())
}

fn checked_face_size(current: u64, table_length: usize) -> Result<u64, ()> {
    let length = u64::try_from(table_length).map_err(|_| ())?;
    let padded = length.checked_add(TABLE_ALIGNMENT - 1).ok_or(())? & !(TABLE_ALIGNMENT - 1);
    let size = current.checked_add(padded).ok_or(())?;
    (size <= MAX_FONT_PAYLOAD_BYTES).then_some(size).ok_or(())
}

#[cfg(test)]
mod tests;
