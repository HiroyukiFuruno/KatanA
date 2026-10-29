use super::fixtures::FontFixture;
use crate::document_source::{BinaryDocumentFormat, OfficeFontSafetyError};

#[test]
fn rejects_separators_drive_prefixes_ads_and_numeric_references() {
    for format in [BinaryDocumentFormat::Docx, BinaryDocumentFormat::Pptx] {
        for name in [
            "../outside",
            "/absolute",
            r"..\outside",
            r"C:\outside",
            "font:stream",
            "&#x2f;outside",
            "&#47;outside",
            "&#92;outside",
            "C&#58;outside",
        ] {
            assert!(
                matches!(
                    FontFixture::validate_face(format, name),
                    Err(OfficeFontSafetyError::UnsafeFontName { .. })
                ),
                "{format:?}: {name}"
            );
        }
    }
}

#[test]
fn preserves_unicode_spaces_hyphens_and_entities_in_multiple_faces() {
    for format in [BinaryDocumentFormat::Docx, BinaryDocumentFormat::Pptx] {
        let body = [
            "\u{6e38}\u{30b4}\u{30b7}\u{30c3}\u{30af} \u{65e5}\u{672c}\u{8a9e}",
            "Noto-Sans",
            "A&amp;B",
            "Font &quot;Regular&quot;",
            "Font&apos;s",
            "&#x65e5;\u{672c}\u{8a9e}",
            "Font.With.Dots",
        ]
        .map(|name| FontFixture::face(format, name))
        .join("");
        assert_eq!(FontFixture::validate_body(format, &body), Ok(()));
    }
}

#[test]
fn ignores_unembedded_font_list_names() {
    let docx = r#"<w:font w:name="../not-embedded"/>"#;
    let pptx = r#"<p:font typeface="../not-embedded"/><p:embeddedFont><p:font typeface="../no-face"/></p:embeddedFont>"#;
    for (format, body) in [
        (BinaryDocumentFormat::Docx, docx),
        (BinaryDocumentFormat::Pptx, pptx),
    ] {
        assert_eq!(FontFixture::validate_body(format, body), Ok(()));
    }
}

#[test]
fn unsafe_alias_names_cannot_be_overwritten_by_a_safe_namespace_value() {
    let body =
        r#"<alias:font alias:name="../outside" w:name="Safe"><w:embedRegular/></alias:font>"#;
    assert!(matches!(
        FontFixture::validate_body(BinaryDocumentFormat::Docx, body),
        Err(OfficeFontSafetyError::UnsafeFontName { .. })
    ));
}

#[test]
fn every_known_face_marker_is_checked_without_assuming_rid_or_guid_resolution() {
    for marker in [
        "embedRegular",
        "embedBold",
        "embedItalic",
        "embedBoldItalic",
    ] {
        let body = format!(r#"<w:font w:name="../unsafe"><w:{marker}/></w:font>"#);
        assert!(matches!(
            FontFixture::validate_body(BinaryDocumentFormat::Docx, &body),
            Err(OfficeFontSafetyError::UnsafeFontName { .. })
        ));
    }
    for marker in ["regular", "bold", "italic", "boldItalic"] {
        let body = format!(
            r#"<p:embeddedFont><p:font typeface="../unsafe"/><p:{marker}/></p:embeddedFont>"#
        );
        assert!(matches!(
            FontFixture::validate_body(BinaryDocumentFormat::Pptx, &body),
            Err(OfficeFontSafetyError::UnsafeFontName { .. })
        ));
    }
}
