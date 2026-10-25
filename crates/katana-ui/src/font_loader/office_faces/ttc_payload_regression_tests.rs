use std::sync::atomic::AtomicBool;

use super::FontFaceResolutionDiagnostic;
use super::FontFaceResolver;
use super::ttc_payload_assertions::assert_resolution;
use super::ttc_payload_fixture::build_fixture;

const TTC_OFFSET_TABLE_OFFSET: usize = 12;
const TTC_OFFSET_END: usize = 16;
const SFNT_TABLE_COUNT_OFFSET: usize = 4;
const SFNT_TABLE_COUNT_END: usize = 6;
const SFNT_TABLE_DIRECTORY_OFFSET: usize = 12;
const SFNT_TABLE_RECORD_BYTES: usize = 16;
const SFNT_TAG_BYTES: usize = 4;
const SFNT_TABLE_LENGTH_OFFSET: usize = 12;
const SFNT_TABLE_LENGTH_END: usize = 16;

#[test]
fn multi_face_ttc_does_not_clone_the_whole_payload_for_each_selected_face() {
    let fixture = build_fixture();
    let candidates = vec![(
        "fixture-font".to_owned(),
        fixture.path.to_string_lossy().into_owned(),
    )];
    let report = FontFaceResolver::resolve(&candidates, &fixture.requests, &AtomicBool::new(false));
    assert_resolution(
        report,
        &fixture.bytes,
        &fixture.requests,
        &fixture.source_indices,
    );
}

#[test]
fn malformed_selected_ttc_table_records_an_incomplete_face_diagnostic() {
    let mut fixture = build_fixture();
    corrupt_glyph_table(&mut fixture.bytes);
    std::fs::write(&fixture.path, &fixture.bytes).expect("write malformed TTC fixture");
    let candidates = [(
        "fixture".into(),
        fixture.path.to_string_lossy().into_owned(),
    )];
    let report = FontFaceResolver::resolve(&candidates, &fixture.requests, &AtomicBool::new(false));
    assert_eq!(report.faces.len(), 1);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        FontFaceResolutionDiagnostic::InvalidFontFace { face_index: 0, .. }
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        FontFaceResolutionDiagnostic::RequestedFaceUnavailable {
            search_incomplete: true,
            ..
        }
    )));
}

fn corrupt_glyph_table(bytes: &mut [u8]) {
    let base = u32::from_be_bytes(
        bytes[TTC_OFFSET_TABLE_OFFSET..TTC_OFFSET_END]
            .try_into()
            .expect("first TTC offset"),
    ) as usize;
    let count = u16::from_be_bytes(
        bytes[base + SFNT_TABLE_COUNT_OFFSET..base + SFNT_TABLE_COUNT_END]
            .try_into()
            .expect("table count"),
    );
    let entry = (0..usize::from(count))
        .map(|index| base + SFNT_TABLE_DIRECTORY_OFFSET + index * SFNT_TABLE_RECORD_BYTES)
        .find(|entry| &bytes[*entry..*entry + SFNT_TAG_BYTES] == b"glyf")
        .expect("installed pair uses TrueType outlines");
    bytes[entry + SFNT_TABLE_LENGTH_OFFSET..entry + SFNT_TABLE_LENGTH_END]
        .copy_from_slice(&u32::MAX.to_be_bytes());
}
