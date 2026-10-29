use super::fixtures::FontFixture;
use crate::document_source::{BinaryDocumentFormat, OfficeFontSafetyError, OfficeFontSafetyPolicy};
use katana_document_viewer::OfficePreflightLimits;
use std::io::Cursor;

#[test]
fn rejects_distinct_cp437_and_utf8_entries_with_the_same_decoded_name() {
    let bytes = FontFixture::decoded_name_collision();
    let archive = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
    assert!(!archive.has_duplicate_names());
    assert_eq!(
        OfficeFontSafetyPolicy::validate(BinaryDocumentFormat::Docx, &bytes),
        Err(OfficeFontSafetyError::InvalidArchive(
            "ambiguous entry names".to_owned()
        ))
    );
}

#[test]
fn malformed_central_signature_is_rejected_before_metadata_is_exposed() {
    let mut bytes = FontFixture::package(&[("word/fontTable.xml", b"<fonts/>")]);
    let central = zip::ZipArchive::new(Cursor::new(&bytes))
        .unwrap()
        .central_directory_start();
    bytes[central as usize] = b'!';
    assert!(zip::ZipArchive::new(Cursor::new(&bytes)).is_err());
    assert!(matches!(
        OfficeFontSafetyPolicy::validate(BinaryDocumentFormat::Docx, &bytes),
        Err(OfficeFontSafetyError::InvalidArchive(_))
    ));
}

#[test]
fn pdf_and_xlsx_never_parse_empty_invalid_or_hostile_bytes() {
    for format in [BinaryDocumentFormat::Pdf, BinaryDocumentFormat::Xlsx] {
        for bytes in [b"".as_slice(), b"not a ZIP", b"<!DOCTYPE invalid>"] {
            assert_eq!(OfficeFontSafetyPolicy::validate(format, bytes), Ok(()));
        }
    }
}

#[test]
fn missing_metadata_is_normal_but_invalid_zip_is_not_hidden() {
    let bytes = FontFixture::package(&[("other.xml", b"<root/>")]);
    for format in [BinaryDocumentFormat::Docx, BinaryDocumentFormat::Pptx] {
        assert_eq!(OfficeFontSafetyPolicy::validate(format, &bytes), Ok(()));
        assert!(matches!(
            OfficeFontSafetyPolicy::validate(format, b"not a ZIP"),
            Err(OfficeFontSafetyError::InvalidArchive(_))
        ));
    }
}

#[test]
fn rejects_duplicate_metadata_parts_despite_different_payloads() {
    let first = "word/fontTable.xml";
    let alias = "word/fontTablf.xml";
    let mut bytes = FontFixture::package(&[(first, b"<unsafe/>"), (alias, b"<safe/>")]);
    for index in 0..=bytes.len() - alias.len() {
        if &bytes[index..index + alias.len()] == alias.as_bytes() {
            bytes[index..index + alias.len()].copy_from_slice(first.as_bytes());
        }
    }
    assert!(matches!(
        OfficeFontSafetyPolicy::validate(BinaryDocumentFormat::Docx, &bytes),
        Err(OfficeFontSafetyError::InvalidArchive(_))
    ));
}

#[test]
fn rejects_metadata_unicode_path_aliases_in_both_directions() {
    for format in [BinaryDocumentFormat::Docx, BinaryDocumentFormat::Pptx] {
        let part = FontFixture::part(format);
        for (raw, decoded) in [(part, "hidden.xml"), ("hidden.xml", part)] {
            let bytes = FontFixture::unicode_name(raw, decoded);
            assert!(matches!(
                OfficeFontSafetyPolicy::validate(format, &bytes),
                Err(OfficeFontSafetyError::InvalidArchive(_))
            ));
        }
    }
}

#[test]
fn rejects_the_existing_metadata_byte_limit_before_xml_state_is_built() {
    let limit = OfficePreflightLimits::strict().max_entry_uncompressed_bytes;
    let oversized = vec![b' '; limit as usize + 1];
    let bytes = FontFixture::package(&[("word/fontTable.xml", &oversized)]);
    assert_eq!(
        OfficeFontSafetyPolicy::validate(BinaryDocumentFormat::Docx, &bytes),
        Err(OfficeFontSafetyError::ResourceLimit {
            part: "word/fontTable.xml",
            limit
        })
    );
}
