use std::collections::BTreeMap;
use std::fs;
use std::sync::atomic::AtomicBool;

use super::super::font_file::FontCandidate;
use super::super::font_metadata::face_metadata;
use super::super::resolver_metadata::CandidateMetadata;
use super::super::resolver_selection::request_key;
use super::super::types::{FontFaceRequest, FontFaceResolution, FontFaceResolutionDiagnostic};

#[test]
fn full_payload_metadata_is_revalidated_after_same_file_update() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("font.ttf");
    let original = super::super::sfnt_fixture::embedded_font();
    fs::write(&path, &original).unwrap();
    let cancelled = AtomicBool::new(false);
    let mut candidate = FontCandidate::open(&path, &cancelled).unwrap();
    let requests = requests_for(&original);
    assert!(CandidateMetadata::inspect(&mut candidate, &requests, &BTreeMap::new(), &cancelled).0);
    let replacement = super::super::sfnt_fixture::with_distinct_family_aliases(original);
    fs::write(&path, replacement).unwrap();
    let payload = candidate.read_payload(&cancelled).unwrap();
    let mut selected = BTreeMap::new();
    super::inspect_payload(
        &path,
        payload,
        &requests,
        &mut selected,
        &mut FontFaceResolution::default(),
        &mut false,
        &cancelled,
    );
    assert!(selected.is_empty());
}

#[test]
fn corrupted_payload_after_metadata_does_not_block_next_valid_candidate() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("font.ttf");
    let next = temporary.path().join("fallback.ttf");
    let original = super::super::sfnt_fixture::embedded_font();
    fs::write(&path, &original).unwrap();
    fs::write(&next, &original).unwrap();
    let requests = requests_for(&original);
    let cancelled = AtomicBool::new(false);
    let mut candidate = FontCandidate::open(&path, &cancelled).unwrap();
    assert!(CandidateMetadata::inspect(&mut candidate, &requests, &BTreeMap::new(), &cancelled).0);
    fs::write(&path, b"corrupted after metadata").unwrap();
    let payload = candidate.read_payload(&cancelled).unwrap();
    let mut selected = BTreeMap::new();
    let mut result = FontFaceResolution::default();
    let mut incomplete = false;
    super::inspect_payload(
        &path,
        payload,
        &requests,
        &mut selected,
        &mut result,
        &mut incomplete,
        &cancelled,
    );
    super::inspect_candidate(
        &next,
        &requests,
        &mut selected,
        &mut result,
        &mut incomplete,
        &cancelled,
    );
    assert_eq!(selected.len(), 1);
    assert!(incomplete);
    assert!(matches!(
        result.diagnostics.as_slice(),
        [FontFaceResolutionDiagnostic::InvalidFontFile { .. }]
    ));
}

fn requests_for(bytes: &[u8]) -> BTreeMap<super::RequestKey, FontFaceRequest> {
    let font = skrifa::FontRef::new(bytes).unwrap();
    let metadata = face_metadata(&font).unwrap();
    let request = FontFaceRequest {
        family: metadata.family,
        bold: metadata.bold,
        italic: metadata.italic,
    };
    BTreeMap::from([(request_key(&request), request)])
}
