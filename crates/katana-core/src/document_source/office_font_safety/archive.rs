use super::names::ArchiveNames;
use super::scan::FontScan;
use super::xml::FontXml;
use super::{FontParts, OfficeFontSafetyError};
use katana_document_viewer::OfficePreflightLimits;
use std::io::Cursor;
use zip::ZipArchive;

pub(super) struct FontArchive;

impl FontArchive {
    pub(super) fn validate(bytes: &[u8], parts: &FontParts) -> Result<(), OfficeFontSafetyError> {
        /* WHY: 既存KDVの上限を再利用し、この防御だけの通常文書制限を増やさない。 */
        let limits = OfficePreflightLimits::strict();
        OfficeFontSafetyError::ensure_size("package", bytes.len() as u64, limits.max_source_bytes)?;
        let mut archive =
            ZipArchive::new(Cursor::new(bytes)).map_err(OfficeFontSafetyError::archive)?;
        if archive.len() > limits.max_entries || archive.has_duplicate_names() {
            return Err(OfficeFontSafetyError::archive(
                "duplicate entries or existing KDV entry limit exceeded",
            ));
        }
        if !ArchiveNames::has_metadata(bytes, &archive, parts)? {
            return Ok(());
        }
        let entry = archive
            .by_name(parts.metadata)
            .map_err(OfficeFontSafetyError::archive)?;
        OfficeFontSafetyError::ensure_size(
            parts.metadata,
            entry.size(),
            limits.max_entry_uncompressed_bytes,
        )?;
        let mut scan = FontScan::new(parts);
        FontXml::visit(
            entry,
            parts.metadata,
            limits.max_entry_uncompressed_bytes,
            |event| scan.visit(event),
        )
    }
}
