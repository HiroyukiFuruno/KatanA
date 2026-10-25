use super::{CanvasWorker, REVISION_CHUNK_BYTES};
use sha2::{Digest, Sha256};
use std::cell::Cell;

const SOURCE_CHUNKS: usize = 3;
const CANCEL_INSIDE_JSON_CHECK: usize = 16;
const ARRAY_ELEMENTS: usize = 256;

#[test]
fn chunked_revision_matches_original_sha256() {
    for source in [
        String::new(),
        "\u{3042}".repeat(REVISION_CHUNK_BYTES * SOURCE_CHUNKS),
    ] {
        let original: [u8; 32] = Sha256::digest(source.as_bytes()).into();
        assert_eq!(CanvasWorker::revision(&source, || false), Some(original));
    }
}

#[test]
fn revision_cancellation_stops_before_remaining_chunks() {
    let source = "x".repeat(REVISION_CHUNK_BYTES * SOURCE_CHUNKS);
    let checks = Cell::new(0);
    assert_eq!(
        CanvasWorker::revision(&source, || {
            let count = checks.get() + 1;
            checks.set(count);
            count == SOURCE_CHUNKS
        }),
        None
    );
    assert_eq!(checks.get(), SOURCE_CHUNKS);
}

#[test]
fn production_worker_parser_cancels_inside_unknown_json_values() {
    let values = std::iter::repeat_n("0", ARRAY_ELEMENTS)
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(r#"{{"extension":[{values}],"nodes":INVALID}}"#);
    let checks = Cell::new(0);
    let parsed = CanvasWorker::parse_source(&source, || {
        checks.set(checks.get() + 1);
        checks.get() == CANCEL_INSIDE_JSON_CHECK
    });
    assert!(parsed.is_none());
    assert_eq!(checks.get(), CANCEL_INSIDE_JSON_CHECK);
}

#[test]
fn production_worker_parser_keeps_invalid_input_revision() {
    let parsed = CanvasWorker::parse_source("INVALID", || false).expect("uncancelled result");
    assert!(parsed.result.is_err());
    let original: [u8; 32] = Sha256::digest(b"INVALID").into();
    assert_eq!(parsed.revision, original);
}
