use super::fixtures::FontFixture;
use crate::document_source::{BinaryDocumentFormat, OfficeFontSafetyError};

#[test]
fn pptx_order_and_multiple_names_cannot_clear_an_unsafe_candidate() {
    for body in [
        r#"<p:embeddedFont><p:font typeface="../bad"/><p:regular/><p:font typeface="Safe"/></p:embeddedFont>"#,
        r#"<p:embeddedFont><p:bold/><p:font typeface="../bad"/></p:embeddedFont>"#,
        r#"<p:embeddedFont><p:font typeface="Safe" alias:typeface="../bad"/><p:regular/></p:embeddedFont>"#,
    ] {
        assert!(matches!(
            FontFixture::validate_body(BinaryDocumentFormat::Pptx, body),
            Err(OfficeFontSafetyError::UnsafeFontName { .. })
        ));
    }
}

#[test]
fn nested_wrappers_fail_closed_instead_of_resetting_outer_state() {
    for (format, body) in [
        (
            BinaryDocumentFormat::Docx,
            r#"<w:font w:name="../bad"><w:embedRegular/><w:font w:name="Safe"/></w:font>"#,
        ),
        (
            BinaryDocumentFormat::Pptx,
            r#"<p:embeddedFont><p:font typeface="../bad"/><p:regular/><p:embeddedFont/></p:embeddedFont>"#,
        ),
    ] {
        assert!(matches!(
            FontFixture::validate_body(format, body),
            Err(OfficeFontSafetyError::InvalidMetadata { .. })
        ));
    }
}

#[test]
fn completed_unembedded_fonts_do_not_poison_later_normal_faces() {
    let body = format!(
        r#"<w:font w:name="../unused"/>{}"#,
        FontFixture::face(BinaryDocumentFormat::Docx, "Normal")
    );
    assert_eq!(
        FontFixture::validate_body(BinaryDocumentFormat::Docx, &body),
        Ok(())
    );
    let body = format!(
        r#"<p:embeddedFont><p:font typeface="../unused"/></p:embeddedFont>{}"#,
        FontFixture::face(BinaryDocumentFormat::Pptx, "Normal")
    );
    assert_eq!(
        FontFixture::validate_body(BinaryDocumentFormat::Pptx, &body),
        Ok(())
    );
}
