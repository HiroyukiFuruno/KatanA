use std::io::{Read, Seek};

use skrifa::{FontRef, Tag};
use write_fonts::FontBuilder;

use super::metadata_reader::{FontMetadataReader, SFNT_HEADER_BYTES, SFNT_TABLE_BYTES};
use super::types::{FontFaceResolutionDiagnostic, MAX_FONT_PAYLOAD_BYTES};

const METADATA_TAGS: [Tag; 4] = [
    Tag::new(b"name"),
    Tag::new(b"OS/2"),
    Tag::new(b"head"),
    Tag::new(b"post"),
];
const TABLE_ALIGNMENT: usize = 4;

pub(super) struct MetadataTables;

impl MetadataTables {
    pub(super) fn read<R: Read + Seek>(
        reader: &mut FontMetadataReader<'_, R>,
        directory: &[u8],
    ) -> Result<Vec<u8>, FontFaceResolutionDiagnostic> {
        let face = FontRef::new(directory).map_err(|_| reader.invalid())?;
        let mut builder = FontBuilder::new();
        let mut size = SFNT_HEADER_BYTES + METADATA_TAGS.len() * SFNT_TABLE_BYTES;
        for tag in METADATA_TAGS {
            Self::add_table(reader, &face, tag, &mut builder, &mut size)?;
        }
        Ok(builder.build())
    }

    fn add_table<R: Read + Seek>(
        reader: &mut FontMetadataReader<'_, R>,
        face: &FontRef<'_>,
        tag: Tag,
        builder: &mut FontBuilder<'_>,
        size: &mut usize,
    ) -> Result<(), FontFaceResolutionDiagnostic> {
        let Some(record) = face
            .table_directory()
            .table_records()
            .iter()
            .find(|record| record.tag() == tag)
        else {
            return Ok(());
        };
        let offset = u64::from(record.offset());
        let length = u64::from(record.length());
        if !reader.range_exists(offset, length) {
            return Ok(());
        }
        let padded = (length + TABLE_ALIGNMENT as u64 - 1) & !(TABLE_ALIGNMENT as u64 - 1);
        *size = size
            .checked_add(padded as usize)
            .filter(|total| *total as u64 <= MAX_FONT_PAYLOAD_BYTES)
            .ok_or_else(|| reader.invalid())?;
        let bytes = reader.read_at(offset, length as usize)?;
        builder.add_raw(tag, bytes);
        Ok(())
    }
}
