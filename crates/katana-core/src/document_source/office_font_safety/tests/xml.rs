use super::fixtures::FontFixture;
use crate::document_source::{BinaryDocumentFormat, OfficeFontSafetyError, OfficeFontSafetyPolicy};

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
