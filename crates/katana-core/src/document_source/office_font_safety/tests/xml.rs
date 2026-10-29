use super::super::xml::FontXml;
use super::fixtures::FontFixture;
use crate::document_source::{BinaryDocumentFormat, OfficeFontSafetyError, OfficeFontSafetyPolicy};
use std::io::Cursor;

#[test]
fn parser_rejects_namespace_aliases_of_the_same_attribute_before_emitting_a_start() {
    let xml = br#"<r xmlns:a="urn:same" xmlns:b="urn:same" a:name="Safe" b:name="../unsafe"/>"#;
    let bytes = FontFixture::with_metadata(BinaryDocumentFormat::Docx, xml);
    assert!(matches!(
        OfficeFontSafetyPolicy::validate(BinaryDocumentFormat::Docx, &bytes),
        Err(OfficeFontSafetyError::InvalidMetadata { reason, .. }) if reason.contains("redefined")
    ));
}

#[test]
fn zip_xml_stream_accepts_the_exact_byte_limit_and_rejects_one_byte_more() {
    let part = FontFixture::part(BinaryDocumentFormat::Docx);
    let xml = b"<fonts/> ";
    let bytes = FontFixture::with_metadata(BinaryDocumentFormat::Docx, xml);
    for limit in [xml.len() as u64, xml.len() as u64 - 1] {
        let mut archive = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
        let entry = archive.by_name(part).unwrap();
        let result = FontXml::visit(entry, part, limit, |_| Ok(()));
        let expected = if limit == xml.len() as u64 {
            Ok(())
        } else {
            Err(OfficeFontSafetyError::InvalidMetadata {
                part,
                reason: "XML byte limit exceeded".to_owned(),
            })
        };
        assert_eq!(result, expected);
    }
}

#[test]
fn supports_both_utf16_orders_and_rejects_encoded_unsafe_names() {
    for format in [BinaryDocumentFormat::Docx, BinaryDocumentFormat::Pptx] {
        for name in [
            "\u{65e5}\u{672c}\u{8a9e} Safe",
            "../outside",
            "&#47;outside",
            "&#92;outside",
            "C&#58;outside",
        ] {
            assert_utf16_name(format, name);
        }
    }
}

fn assert_utf16_name(format: BinaryDocumentFormat, name: &str) {
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-16\"?>{}",
        FontFixture::metadata(format, &FontFixture::face(format, name))
    );
    for little_endian in [true, false] {
        let bytes = FontFixture::with_metadata(format, &FontFixture::utf16(&xml, little_endian));
        let result = OfficeFontSafetyPolicy::validate(format, &bytes);
        assert_eq!(
            result.is_ok(),
            name == "\u{65e5}\u{672c}\u{8a9e} Safe",
            "{format:?}, LE={little_endian}: {result:?}"
        );
    }
}

#[test]
fn rejects_dtd_entities_and_malformed_or_ambiguous_xml() {
    for xml in [
        "<!DOCTYPE r [<!ENTITY path '../outside'>]><r>&path;</r>",
        "<!DOCTYPE r SYSTEM 'file:///not-read'><r/>",
        "<r>&undefined;</r>",
        "<r><unclosed></r>",
        "<r duplicate='a' duplicate='b'/>",
        "<?xml version='1.0' encoding='not-an-encoding'?><r/>",
    ] {
        let bytes = FontFixture::with_metadata(BinaryDocumentFormat::Docx, xml.as_bytes());
        assert!(
            matches!(
                OfficeFontSafetyPolicy::validate(BinaryDocumentFormat::Docx, &bytes),
                Err(OfficeFontSafetyError::InvalidMetadata { .. })
            ),
            "{xml}"
        );
    }
}

#[test]
fn wide_and_deep_unrelated_xml_does_not_require_a_dom_or_new_depth_limit() {
    let deep = format!("{}<none/>{}", "<n>".repeat(2_000), "</n>".repeat(2_000));
    let wide = format!("<root>{}</root>", "<none/>".repeat(20_000));
    for xml in [deep, wide] {
        let bytes = FontFixture::with_metadata(BinaryDocumentFormat::Docx, xml.as_bytes());
        assert_eq!(
            OfficeFontSafetyPolicy::validate(BinaryDocumentFormat::Docx, &bytes),
            Ok(())
        );
    }
}
