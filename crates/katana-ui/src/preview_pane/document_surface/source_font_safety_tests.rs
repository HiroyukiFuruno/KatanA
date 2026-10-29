use super::DocumentSurfaceSource;
use fixtures::{docx_with_font, office_zip, pptx_with_font};
use katana_core::document_source::BinaryDocumentFormat;
use katana_document_viewer::ViewerSource;
use std::path::Path;

#[path = "source_font_safety_fixtures.rs"]
mod fixtures;

#[derive(Clone, Copy)]
enum SourceKind {
    Local,
    Remote,
}

fn build_source(
    kind: SourceKind,
    directory: &Path,
    format: BinaryDocumentFormat,
    bytes: Vec<u8>,
) -> DocumentSurfaceSource {
    match kind {
        SourceKind::Local => {
            let path = directory.join(format!("fixture.{}", format.extension()));
            std::fs::write(&path, bytes).expect("write local ZIP fixture");
            DocumentSurfaceSource::local(&path).expect("local document source")
        }
        SourceKind::Remote => DocumentSurfaceSource::remote(
            format!("https://example.test/fixture.{}", format.extension()),
            Some(format.mime()),
            bytes,
        )
        .expect("remote document source"),
    }
}
fn assert_rejected(source: &mut DocumentSurfaceSource) {
    let error = source
        .take_viewer_source()
        .err()
        .expect("unsafe embedded font name must be rejected");
    assert_eq!(error.operation, "validate_embedded_fonts");
}
fn assert_accepted(source: &mut DocumentSurfaceSource, format: BinaryDocumentFormat) {
    let viewer_source = source
        .take_viewer_source()
        .expect("safe document source must reach viewer handoff");
    match format {
        BinaryDocumentFormat::Pdf => assert!(matches!(viewer_source, ViewerSource::Pdf(_))),
        _ => assert!(matches!(viewer_source, ViewerSource::Office(_))),
    }
}

#[test]
fn local_and_remote_office_sources_reject_traversal_font_names() {
    let directory = tempfile::tempdir().expect("temporary fixture directory");
    for (format, bytes) in [
        (BinaryDocumentFormat::Docx, docx_with_font("../outside")),
        (BinaryDocumentFormat::Pptx, pptx_with_font("../outside")),
    ] {
        for kind in [SourceKind::Local, SourceKind::Remote] {
            let mut source = build_source(kind, directory.path(), format, bytes.clone());
            assert_rejected(&mut source);
        }
    }
}

#[test]
fn local_and_remote_office_sources_accept_unicode_font_names() {
    let directory = tempfile::tempdir().expect("temporary fixture directory");
    for kind in [SourceKind::Local, SourceKind::Remote] {
        let docx = docx_with_font("\u{6e38}\u{30b4}\u{30b7}\u{30c3}\u{30af}");
        let mut source = build_source(kind, directory.path(), BinaryDocumentFormat::Docx, docx);
        assert_accepted(&mut source, BinaryDocumentFormat::Docx);

        let pptx = pptx_with_font("\u{6e38}\u{30b4}\u{30b7}\u{30c3}\u{30af}");
        let mut source = build_source(kind, directory.path(), BinaryDocumentFormat::Pptx, pptx);
        assert_accepted(&mut source, BinaryDocumentFormat::Pptx);
    }
}

#[test]
fn pdf_and_xlsx_do_not_parse_office_font_xml() {
    let directory = tempfile::tempdir().expect("temporary fixture directory");
    let markup = r#"<w:font w:name="../outside"><w:embedRegular r:id="rId1"/></w:font>"#;
    for kind in [SourceKind::Local, SourceKind::Remote] {
        let pdf = format!("%PDF-1.7\n{markup}").into_bytes();
        let mut source = build_source(kind, directory.path(), BinaryDocumentFormat::Pdf, pdf);
        assert_accepted(&mut source, BinaryDocumentFormat::Pdf);

        let xlsx = office_zip(&[("xl/styles.xml", markup)]);
        let mut source = build_source(kind, directory.path(), BinaryDocumentFormat::Xlsx, xlsx);
        assert_accepted(&mut source, BinaryDocumentFormat::Xlsx);
    }
}
