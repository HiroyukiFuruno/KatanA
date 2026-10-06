use std::fs;
use std::sync::atomic::AtomicBool;

use skrifa::{FontRef, string::StringId};

use super::resolver::FontFaceResolver;
use super::sfnt_fixture::{
    embedded_font, family_names_for_id, preferred_family_name, with_distinct_family_aliases,
};
use super::types::{FontFaceRequest, FontFaceResolutionDiagnostic};
use super::unicode_sfnt_fixture::{with_ecole_family_aliases, with_strasse_family_aliases};

#[test]
fn resolver_accepts_legacy_family_alias_from_real_font_metadata() {
    let bytes = with_distinct_family_aliases(embedded_font());
    let font = FontRef::new(&bytes).expect("modified embedded font is valid");
    let canonical = preferred_family_name(&font).expect("font has canonical family name");
    let legacy_names = family_names_for_id(&font, StringId::FAMILY_NAME);
    let alias = legacy_names
        .iter()
        .find(|name| name.contains('日'))
        .cloned()
        .expect("font has a Japanese legacy family name");
    assert!(
        legacy_names.iter().any(|name| name.starts_with('L')),
        "fixture has no same-ID localized family alias"
    );
    assert!(alias.contains('日'));
    assert!(canonical.starts_with('T'));
    let temporary = tempfile::tempdir().expect("temporary directory");
    let path = temporary.path().join("localized-alias.ttf");
    fs::write(&path, bytes).expect("write real font fixture");
    let candidates = [(
        "unrelated filename stem".into(),
        path.to_string_lossy().into_owned(),
    )];
    let requests = [
        FontFaceRequest {
            family: alias.clone(),
            bold: false,
            italic: false,
        },
        FontFaceRequest {
            family: canonical.clone(),
            bold: false,
            italic: false,
        },
    ];

    let report = FontFaceResolver::resolve(&candidates, &requests, &AtomicBool::new(false));

    assert!(
        report.diagnostics.is_empty(),
        "resolver diagnostics: {:?}",
        report
            .diagnostics
            .iter()
            .map(FontFaceResolutionDiagnostic::description)
            .collect::<Vec<_>>()
    );
    assert_eq!(report.faces.len(), 2);
    let alias_face = report
        .faces
        .iter()
        .find(|face| face.request.family == alias)
        .expect("localized alias request is preserved");
    let canonical_face = report
        .faces
        .iter()
        .find(|face| face.request.family == canonical)
        .expect("canonical request is preserved");
    assert!(std::sync::Arc::ptr_eq(
        &alias_face.payload,
        &canonical_face.payload
    ));
    assert!(report.faces.iter().all(|face| face.family == canonical));
    assert!(report.faces.iter().all(|face| !face.bold && !face.italic));
}

#[test]
fn resolver_matches_unicode_case_family_aliases_from_real_sfnt_metadata() {
    let bytes = with_ecole_family_aliases(embedded_font());
    let font = FontRef::new(&bytes).expect("modified embedded font is valid");
    let canonical = preferred_family_name(&font).expect("font has canonical family name");
    assert_eq!(canonical, "École");

    let temporary = tempfile::tempdir().expect("temporary directory");
    let path = temporary.path().join("ecole-alias.ttf");
    fs::write(&path, bytes).expect("write real font fixture");
    let candidates = [(
        "unrelated filename stem".into(),
        path.to_string_lossy().into_owned(),
    )];
    let requests = [FontFaceRequest {
        family: "école".into(),
        bold: false,
        italic: false,
    }];

    let report = FontFaceResolver::resolve(&candidates, &requests, &AtomicBool::new(false));

    assert!(
        report.diagnostics.is_empty(),
        "resolver diagnostics: {:?}",
        report
            .diagnostics
            .iter()
            .map(FontFaceResolutionDiagnostic::description)
            .collect::<Vec<_>>()
    );
    assert_eq!(report.faces.len(), 1);
    assert!(report.faces.iter().all(|face| face.family == canonical));
    assert_eq!(report.faces[0].request.family, "école");
    assert!(report.faces.iter().all(|face| !face.bold && !face.italic));
}

#[test]
fn resolver_matches_canonically_decomposed_unicode_family_aliases() {
    let bytes = with_ecole_family_aliases(embedded_font());
    let temporary = tempfile::tempdir().expect("temporary directory");
    let path = temporary.path().join("ecole-decomposed.ttf");
    fs::write(&path, bytes).expect("write real font fixture");
    let candidates = [(
        "unrelated filename stem".into(),
        path.to_string_lossy().into_owned(),
    )];
    let requests = [FontFaceRequest {
        family: "E\u{301}cole".into(),
        bold: false,
        italic: false,
    }];

    let report = FontFaceResolver::resolve(&candidates, &requests, &AtomicBool::new(false));

    assert!(report.diagnostics.is_empty());
    assert_eq!(report.faces.len(), 1);
    assert_eq!(report.faces[0].family, "École");
    assert_eq!(report.faces[0].request.family, "E\u{301}cole");
}

#[test]
fn resolver_matches_default_case_folded_sharp_s_family_aliases() {
    let bytes = with_strasse_family_aliases(embedded_font());
    let temporary = tempfile::tempdir().expect("temporary directory");
    let path = temporary.path().join("strasse.ttf");
    fs::write(&path, bytes).expect("write real font fixture");
    let candidates = [(
        "unrelated filename stem".into(),
        path.to_string_lossy().into_owned(),
    )];
    let requests = [FontFaceRequest {
        family: "STRASSE".into(),
        bold: false,
        italic: false,
    }];

    let report = FontFaceResolver::resolve(&candidates, &requests, &AtomicBool::new(false));

    assert!(report.diagnostics.is_empty());
    assert_eq!(report.faces.len(), 1);
    assert_eq!(report.faces[0].family, "Straße");
}
