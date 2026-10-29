use crate::document_source::{BinaryDocumentFormat, OfficeFontSafetyError, OfficeFontSafetyPolicy};
use std::io::{Cursor, Write};
use zip::write::{FullFileOptions, SimpleFileOptions};

const KEY: &str = "{7B19B49C-2336-4F82-AAD2-5D2BAE389560}";
const LOCAL_HEADER_BYTES: usize = 30;
const CENTRAL_HEADER_BYTES: usize = 46;
const CP437_E_ACUTE: u8 = 0x82;

pub(super) struct FontFixture;

impl FontFixture {
    pub(super) fn package(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            archive
                .start_file(*name, SimpleFileOptions::default())
                .unwrap();
            archive.write_all(bytes).unwrap();
        }
        archive.finish().unwrap().into_inner()
    }

    pub(super) fn part(format: BinaryDocumentFormat) -> &'static str {
        if format == BinaryDocumentFormat::Docx {
            "word/fontTable.xml"
        } else {
            "ppt/presentation.xml"
        }
    }

    pub(super) fn metadata(format: BinaryDocumentFormat, body: &str) -> String {
        match format {
            BinaryDocumentFormat::Docx => format!(
                r#"<w:fonts xmlns:w="urn:word" xmlns:r="urn:rels" xmlns:alias="urn:alias">{body}</w:fonts>"#
            ),
            _ => format!(
                r#"<p:presentation xmlns:p="urn:ppt" xmlns:r="urn:rels" xmlns:alias="urn:alias"><p:embeddedFontLst>{body}</p:embeddedFontLst></p:presentation>"#
            ),
        }
    }

    pub(super) fn face(format: BinaryDocumentFormat, name: &str) -> String {
        match format {
            BinaryDocumentFormat::Docx => format!(
                r#"<w:font w:name="{name}"><w:embedRegular r:id="face" w:fontKey="{KEY}"/></w:font>"#
            ),
            _ => format!(
                r#"<p:embeddedFont><p:font typeface="{name}"/><p:regular r:id="face"/></p:embeddedFont>"#
            ),
        }
    }

    pub(super) fn with_metadata(format: BinaryDocumentFormat, xml: &[u8]) -> Vec<u8> {
        Self::package(&[(Self::part(format), xml)])
    }

    pub(super) fn validate_body(
        format: BinaryDocumentFormat,
        body: &str,
    ) -> Result<(), OfficeFontSafetyError> {
        let xml = Self::metadata(format, body);
        OfficeFontSafetyPolicy::validate(format, &Self::with_metadata(format, xml.as_bytes()))
    }

    pub(super) fn validate_face(
        format: BinaryDocumentFormat,
        name: &str,
    ) -> Result<(), OfficeFontSafetyError> {
        Self::validate_body(format, &Self::face(format, name))
    }

    pub(super) fn utf16(xml: &str, little_endian: bool) -> Vec<u8> {
        let mut encoded = if little_endian {
            vec![0xff, 0xfe]
        } else {
            vec![0xfe, 0xff]
        };
        for unit in xml.encode_utf16() {
            encoded.extend(if little_endian {
                unit.to_le_bytes()
            } else {
                unit.to_be_bytes()
            });
        }
        encoded
    }

    pub(super) fn unicode_name(raw: &str, decoded: &str) -> Vec<u8> {
        let mut data = vec![1];
        data.extend(Self::crc32(raw.as_bytes()).to_le_bytes());
        data.extend(decoded.as_bytes());
        let mut options = FullFileOptions::default();
        options.add_extra_field(0x7075, data, false).unwrap();
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer.start_file(raw, options).unwrap();
        writer.write_all(b"<fonts/>").unwrap();
        writer.finish().unwrap().into_inner()
    }

    pub(super) fn decoded_name_collision() -> Vec<u8> {
        let mut bytes = Self::package(&[
            ("word/fontTable.xml", b"<fonts/>"),
            ("x", b"legacy"),
            ("\u{e9}", b"unicode"),
        ]);
        let (local, central) = {
            let mut archive = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
            let entry = archive.by_name("x").unwrap();
            (
                entry.header_start() as usize,
                entry.central_header_start() as usize,
            )
        };
        bytes[local + LOCAL_HEADER_BYTES] = CP437_E_ACUTE;
        bytes[central + CENTRAL_HEADER_BYTES] = CP437_E_ACUTE;
        bytes
    }

    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = u32::MAX;
        for byte in bytes {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xedb8_8320 & 0_u32.wrapping_sub(crc & 1));
            }
        }
        !crc
    }
}
