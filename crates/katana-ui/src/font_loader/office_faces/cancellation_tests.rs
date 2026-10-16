use std::sync::atomic::AtomicBool;

use super::resolver::FontFaceResolver;
use super::types::{FontFaceRequest, FontFaceResolutionDiagnostic};

#[test]
fn already_cancelled_does_not_open_candidate_and_returns_typed_diagnostic() {
    let temporary = tempfile::tempdir().expect("temporary directory");
    let path = temporary.path().join("font.ttf");
    let requests = [FontFaceRequest {
        family: "Family".into(),
        bold: false,
        italic: false,
    }];
    let candidates = vec![("ignored".into(), path.to_string_lossy().into_owned())];

    let report = FontFaceResolver::resolve(&candidates, &requests, &AtomicBool::new(true));

    assert!(report.faces.is_empty());
    assert!(
        report
            .diagnostics
            .iter()
            .any(|item| matches!(item, FontFaceResolutionDiagnostic::Cancelled))
    );
    assert!(
        !report
            .diagnostics
            .iter()
            .any(|item| matches!(item, FontFaceResolutionDiagnostic::FileUnreadable { .. }))
    );
}

#[test]
fn empty_requests_return_before_io_even_if_cancelled() {
    let candidates = vec![("missing".into(), "/missing/candidate.ttf".into())];
    let report = FontFaceResolver::resolve(&candidates, &[], &AtomicBool::new(true));
    assert!(report.faces.is_empty());
    assert!(report.diagnostics.is_empty());
}
