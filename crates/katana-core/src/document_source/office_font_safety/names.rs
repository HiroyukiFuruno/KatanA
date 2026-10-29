use super::{FontParts, OfficeFontSafetyError};
use std::collections::HashSet;
use std::io::Cursor;
use zip::ZipArchive;

const CENTRAL_HEADER_BYTES: usize = 46;
const CENTRAL_HEADER_SIGNATURE: &[u8] = b"PK\x01\x02";
const NAME_LENGTH_LOW_OFFSET: usize = 28;
const NAME_LENGTH_HIGH_OFFSET: usize = 29;

pub(super) struct ArchiveNames;

impl ArchiveNames {
    pub(super) fn has_metadata(
        bytes: &[u8],
        archive: &ZipArchive<Cursor<&[u8]>>,
        parts: &FontParts,
    ) -> Result<bool, OfficeFontSafetyError> {
        let metadata = archive.metadata();
        let mut decoded = HashSet::with_capacity(archive.len());
        let mut found = false;
        for index in 0..archive.len() {
            let entry = metadata
                .entry(index)
                .map_err(OfficeFontSafetyError::archive)?;
            let name = entry
                .name()
                .map_err(OfficeFontSafetyError::archive)?
                .into_owned();
            let raw = Self::raw_name(bytes, entry.central_header_start())?;
            Self::validate_metadata_name(raw, &name, parts)?;
            found |= parts.contains(name.as_bytes());
            if !decoded.insert(name) {
                return Err(OfficeFontSafetyError::archive("ambiguous entry names"));
            }
        }
        Ok(found)
    }

    fn validate_metadata_name(
        raw: &[u8],
        decoded: &str,
        parts: &FontParts,
    ) -> Result<(), OfficeFontSafetyError> {
        if (parts.contains(raw) || parts.contains(decoded.as_bytes())) && raw != decoded.as_bytes()
        {
            return Err(OfficeFontSafetyError::archive(
                "ambiguous font metadata entry name",
            ));
        }
        Ok(())
    }

    fn raw_name(bytes: &[u8], offset: u64) -> Result<&[u8], OfficeFontSafetyError> {
        /* WHY: ZIP9はUnicodePath適用後のname_rawを返すため、検証済CDFH位置から元の名前だけを読む。 */
        let start = usize::try_from(offset).map_err(OfficeFontSafetyError::archive)?;
        let header = bytes
            .get(start..start.saturating_add(CENTRAL_HEADER_BYTES))
            .ok_or_else(|| OfficeFontSafetyError::archive("truncated central header"))?;
        if &header[..CENTRAL_HEADER_SIGNATURE.len()] != CENTRAL_HEADER_SIGNATURE {
            return Err(OfficeFontSafetyError::archive("invalid central header"));
        }
        let length = usize::from(u16::from_le_bytes([
            header[NAME_LENGTH_LOW_OFFSET],
            header[NAME_LENGTH_HIGH_OFFSET],
        ]));
        let begin = start.saturating_add(CENTRAL_HEADER_BYTES);
        bytes
            .get(begin..begin.saturating_add(length))
            .ok_or_else(|| OfficeFontSafetyError::archive("truncated entry name"))
    }
}
