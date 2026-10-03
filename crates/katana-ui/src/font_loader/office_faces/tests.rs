use std::fs;
use std::sync::atomic::AtomicBool;

use egui::FontDefinitions;
use skrifa::MetadataProvider;

use super::resolver::FontFaceResolver;
use super::types::{FontFaceRequest, FontFaceResolutionDiagnostic, MAX_FONT_PAYLOAD_BYTES};

#[test]
fn resolves_embedded_font_by_name_table_not_filename_stem() {
    let bytes = embedded_regular_font();
    let original = skrifa::FontRef::new(&bytes).expect("embedded font is valid");
    let family = family_name(&original);
    let weight = original.attributes().weight.value().round() as u16;
    let temporary = tempfile::tempdir().expect("temporary directory");
    let path = temporary.path().join("Misleading-Bold-Font.ttf");
    fs::write(&path, &bytes).expect("write real embedded font");

    let report = resolve(&path, &[request(&family, false, false)]);

    assert!(report.diagnostics.is_empty());
    assert_eq!(report.faces.len(), 1);
    let resolved = &report.faces[0];
    assert_eq!(resolved.family, family);
    assert_eq!(resolved.weight, weight);
    assert!(!resolved.bold);
    assert!(!resolved.italic);
    assert!(!resolved.monospaced);
    assert_eq!(resolved.payload.font.as_ref(), bytes.as_slice());
}

#[test]
fn deduplicates_case_insensitive_family_requests() {
    let bytes = embedded_regular_font();
    let family = family_name(&skrifa::FontRef::new(&bytes).expect("embedded font"));
    let temporary = tempfile::tempdir().expect("temporary directory");
    let path = temporary.path().join("font.ttf");
    fs::write(&path, bytes).expect("write font");
    let candidates = vec![("font".into(), path.to_string_lossy().into_owned())];
    let requests = [
        request(&family, false, false),
        request(&family.to_lowercase(), false, false),
    ];

    let report = FontFaceResolver::resolve(&candidates, &requests, &AtomicBool::new(false));

    assert_eq!(report.faces.len(), 1);
    assert!(report.diagnostics.is_empty());
}

#[test]
fn invalid_oversized_and_nonregular_paths_are_diagnostic() {
    let temporary = tempfile::tempdir().expect("temporary directory");
    let candidates = invalid_candidates(temporary.path());
    let report = FontFaceResolver::resolve(
        &candidates,
        &[request("Absent Family", false, false)],
        &AtomicBool::new(false),
    );
    assert_invalid_diagnostics(&report);
}

fn invalid_candidates(root: &std::path::Path) -> Vec<(String, String)> {
    let invalid = root.join("invalid.ttf");
    let oversized = root.join("oversized.ttf");
    fs::write(&invalid, b"not a font").expect("write invalid font");
    fs::File::create(&oversized)
        .expect("create sparse oversized font")
        .set_len(MAX_FONT_PAYLOAD_BYTES + 1)
        .expect("set sparse file length");
    [invalid, oversized, root.to_owned()]
        .into_iter()
        .map(|path| ("ignored stem".into(), path.to_string_lossy().into_owned()))
        .collect::<Vec<_>>()
}

fn assert_invalid_diagnostics(report: &super::types::FontFaceResolution) {
    assert!(report.faces.is_empty());
    assert!(
        report
            .diagnostics
            .iter()
            .any(|item| matches!(item, FontFaceResolutionDiagnostic::InvalidFontFile { .. }))
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|item| matches!(item, FontFaceResolutionDiagnostic::FileTooLarge { .. }))
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|item| matches!(item, FontFaceResolutionDiagnostic::NonRegularFile { .. }))
    );
    assert!(report.diagnostics.iter().any(|item| matches!(
        item,
        FontFaceResolutionDiagnostic::RequestedFaceUnavailable {
            search_incomplete: true,
            ..
        }
    )));
}

#[test]
fn missing_face_is_not_fabricated() {
    let report = FontFaceResolver::resolve(
        &[],
        &[request("Absent Office Family", true, true)],
        &AtomicBool::new(false),
    );

    assert!(report.faces.is_empty());
    assert!(matches!(
        report.diagnostics.as_slice(),
        [FontFaceResolutionDiagnostic::RequestedFaceUnavailable {
            search_incomplete: false,
            ..
        }]
    ));
}

fn resolve(
    path: &std::path::Path,
    requests: &[FontFaceRequest],
) -> super::types::FontFaceResolution {
    let candidates = vec![("ignored stem".into(), path.to_string_lossy().into_owned())];
    FontFaceResolver::resolve(&candidates, requests, &AtomicBool::new(false))
}

fn request(family: &str, bold: bool, italic: bool) -> FontFaceRequest {
    FontFaceRequest {
        family: family.into(),
        bold,
        italic,
    }
}

fn embedded_regular_font() -> Vec<u8> {
    FontDefinitions::default().font_data["Ubuntu-Light"]
        .font
        .to_vec()
}

fn family_name(font: &skrifa::FontRef<'_>) -> String {
    use skrifa::string::StringId;
    [StringId::TYPOGRAPHIC_FAMILY_NAME, StringId::FAMILY_NAME]
        .into_iter()
        .find_map(|id| {
            font.localized_strings(id)
                .english_or_first()
                .map(|record| record.to_string())
        })
        .expect("embedded family name")
}
