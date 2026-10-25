use std::collections::BTreeMap;
use std::fs;
use std::sync::atomic::AtomicBool;

use skrifa::FontRef;

use super::font_file::FontCandidate;
use super::font_metadata::face_metadata;
use super::resolver::FontFaceResolver;
use super::resolver_metadata::CandidateMetadata;
use super::resolver_selection::request_key;
use super::types::{FontFaceRequest, FontFaceResolutionDiagnostic};

const TTC_FIRST_OFFSET: usize = 12;
const OFFSET_BYTES: usize = 4;
const SFNT_DIRECTORY_BYTES: usize = 12;
const SFNT_RECORD_BYTES: usize = 16;
const TABLE_TAG_BYTES: usize = 4;
const SFNT_TABLE_COUNT_OFFSET: usize = 4;

#[test]
fn malformed_single_font_keeps_top_level_invalid_file_diagnostic() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("invalid.ttf");
    let valid = super::sfnt_fixture::embedded_font();
    let inputs = [
        vec![0; SFNT_DIRECTORY_BYTES],
        b"unsupported version header".to_vec(),
    ];
    for bytes in inputs {
        fs::write(&path, &bytes).unwrap();
        assert!(skrifa::raw::FileRef::new(&bytes).is_err());
        let candidates = [("unused".into(), path.to_string_lossy().into_owned())];
        let report =
            FontFaceResolver::resolve(&candidates, &[request_for(&valid)], &AtomicBool::new(false));
        assert!(matches!(
            report.diagnostics.as_slice(),
            [
                FontFaceResolutionDiagnostic::InvalidFontFile { .. },
                FontFaceResolutionDiagnostic::RequestedFaceUnavailable {
                    search_incomplete: true,
                    ..
                }
            ]
        ));
    }
}

#[test]
fn truncated_single_directory_keeps_full_parser_unavailable_diagnostic() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("truncated.ttf");
    let valid = super::sfnt_fixture::embedded_font();
    fs::write(&path, &valid[..SFNT_DIRECTORY_BYTES]).unwrap();
    let candidates = [("unused".into(), path.to_string_lossy().into_owned())];
    let report =
        FontFaceResolver::resolve(&candidates, &[request_for(&valid)], &AtomicBool::new(false));
    assert!(matches!(
        report.diagnostics.as_slice(),
        [FontFaceResolutionDiagnostic::RequestedFaceUnavailable {
            search_incomplete: false,
            ..
        }]
    ));
}

#[test]
fn broken_collection_face_does_not_hide_other_usable_face() {
    let fixture = super::ttc_payload_fixture::build_fixture();
    let mut bytes = fixture.bytes.clone();
    bytes[TTC_FIRST_OFFSET..TTC_FIRST_OFFSET + OFFSET_BYTES]
        .copy_from_slice(&u32::MAX.to_be_bytes());
    fs::write(&fixture.path, &bytes).unwrap();
    let request = fixture
        .requests
        .iter()
        .find(|request| fixture.source_indices[*request] == 1)
        .unwrap()
        .clone();
    let candidates = [("unused".into(), fixture.path.to_string_lossy().into_owned())];
    let report = FontFaceResolver::resolve(&candidates, &[request], &AtomicBool::new(false));
    assert_eq!(report.faces.len(), 1);
    assert_eq!(report.faces[0].face_index, 1);
    assert_eq!(
        report
            .diagnostics
            .iter()
            .filter(|error| matches!(
                error,
                FontFaceResolutionDiagnostic::InvalidFontFace { face_index: 0, .. }
            ))
            .count(),
        1
    );
}

#[test]
fn matching_collection_with_duplicate_payload_table_falls_back_to_next_candidate() {
    let fixture = super::ttc_payload_fixture::build_fixture();
    let request = fixture
        .requests
        .iter()
        .find(|request| fixture.source_indices[*request] == 0)
        .unwrap()
        .clone();
    let mut payload = super::face_payload::FacePayload::new(fixture.bytes.clone());
    let valid = payload.take_face(0).unwrap();
    let fallback = fixture._temporary.path().join("fallback.ttf");
    fs::write(&fallback, valid.font.as_ref()).unwrap();
    let mut damaged = fixture.bytes.clone();
    duplicate_outline_table(&mut damaged);
    fs::write(&fixture.path, damaged).unwrap();
    let candidates = [fixture.path.clone(), fallback.clone()]
        .map(|path| ("unused".into(), path.to_string_lossy().into_owned()));
    let report = FontFaceResolver::resolve(&candidates, &[request], &AtomicBool::new(false));
    assert_eq!(report.faces.len(), 1);
    assert_eq!(report.faces[0].path, fallback);
    assert!(report.diagnostics.iter().any(|error| matches!(
        error,
        FontFaceResolutionDiagnostic::InvalidFontFace { face_index: 0, .. }
    )));
}

fn duplicate_outline_table(bytes: &mut [u8]) {
    let base = u32::from_be_bytes(
        bytes[TTC_FIRST_OFFSET..TTC_FIRST_OFFSET + OFFSET_BYTES]
            .try_into()
            .unwrap(),
    ) as usize;
    let count = u16::from_be_bytes([
        bytes[base + SFNT_TABLE_COUNT_OFFSET],
        bytes[base + SFNT_TABLE_COUNT_OFFSET + 1],
    ]) as usize;
    let records = &mut bytes
        [base + SFNT_DIRECTORY_BYTES..base + SFNT_DIRECTORY_BYTES + count * SFNT_RECORD_BYTES];
    let outline_indices = records
        .chunks_exact(SFNT_RECORD_BYTES)
        .enumerate()
        .filter(|(_, record)| {
            ![b"name", b"OS/2", b"head", b"post"]
                .contains(&record[..TABLE_TAG_BYTES].try_into().unwrap())
        })
        .map(|(index, _)| index)
        .take(2)
        .collect::<Vec<_>>();
    assert_eq!(outline_indices.len(), 2);
    let original = records[outline_indices[0] * SFNT_RECORD_BYTES
        ..outline_indices[0] * SFNT_RECORD_BYTES + TABLE_TAG_BYTES]
        .to_vec();
    let start = outline_indices[1] * SFNT_RECORD_BYTES;
    records[start..start + TABLE_TAG_BYTES].copy_from_slice(&original);
}

#[cfg(unix)]
#[test]
fn metadata_and_payload_use_same_handle_when_path_is_replaced() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("font.ttf");
    let original = super::sfnt_fixture::embedded_font();
    fs::write(&path, &original).unwrap();
    let cancelled = AtomicBool::new(false);
    let mut candidate = FontCandidate::open(&path, &cancelled).unwrap();
    let request = request_for(&original);
    let requests = BTreeMap::from([(request_key(&request), request)]);
    assert!(CandidateMetadata::inspect(&mut candidate, &requests, &BTreeMap::new(), &cancelled).0);
    fs::rename(&path, temporary.path().join("previous.ttf")).unwrap();
    fs::write(&path, b"replacement is not a font").unwrap();
    assert_eq!(candidate.read_payload(&cancelled).unwrap(), original);
}

#[test]
fn cancellation_after_metadata_prevents_full_payload_read() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("font.ttf");
    let original = super::sfnt_fixture::embedded_font();
    fs::write(&path, &original).unwrap();
    let cancelled = AtomicBool::new(false);
    let mut candidate = FontCandidate::open(&path, &cancelled).unwrap();
    let request = request_for(&original);
    let requests = BTreeMap::from([(request_key(&request), request)]);
    assert!(CandidateMetadata::inspect(&mut candidate, &requests, &BTreeMap::new(), &cancelled).0);
    cancelled.store(true, std::sync::atomic::Ordering::Release);
    assert!(matches!(
        candidate.read_payload(&cancelled),
        Err(FontFaceResolutionDiagnostic::Cancelled)
    ));
}

fn request_for(bytes: &[u8]) -> FontFaceRequest {
    let metadata = face_metadata(&FontRef::new(bytes).unwrap()).unwrap();
    FontFaceRequest {
        family: metadata.family,
        bold: metadata.bold,
        italic: metadata.italic,
    }
}
