use std::io::Cursor;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use skrifa::raw::FileRef;

use super::super::metadata_reader::FontMetadataReader;

const HEADER_BYTES: usize = 12;
const U32_BYTES: usize = 4;
const EXTENDED_FACE_COUNT: u32 = 2;

#[test]
fn extended_collection_offsets_match_full_parser_for_both_versions() {
    for major in [1u16, 2] {
        let (bytes, expected) = extended_collection(major);
        let cancelled = AtomicBool::new(false);
        let mut input = Cursor::new(&bytes);
        let mut reader = FontMetadataReader::new(
            &mut input,
            Path::new("extended.ttc"),
            bytes.len() as u64,
            &cancelled,
        );
        assert_eq!(reader.offsets().unwrap(), expected);
        let full = FileRef::new(&bytes).unwrap();
        assert_eq!(
            full.fonts().filter(Result::is_ok).count(),
            EXTENDED_FACE_COUNT as usize
        );
        for (index, offset) in expected.into_iter().enumerate() {
            let compact = reader.face(offset, index).unwrap();
            assert!(skrifa::FontRef::new(&compact).is_ok());
        }
    }
}

#[test]
fn missing_or_out_of_bounds_extended_offsets_are_rejected() {
    for major in [1u16, 2] {
        let (bytes, offsets) = extended_collection(major);
        let count_offset = offsets[1] as usize - U32_BYTES * (EXTENDED_FACE_COUNT as usize + 1);
        let truncated = &bytes[..count_offset];
        assert_invalid_collection(truncated);
        let mut excessive = bytes.clone();
        excessive[count_offset..count_offset + U32_BYTES].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_invalid_collection(&excessive);
    }
}

fn assert_invalid_collection(bytes: &[u8]) {
    let cancelled = AtomicBool::new(false);
    let mut input = Cursor::new(bytes);
    let mut reader = FontMetadataReader::new(
        &mut input,
        Path::new("invalid.ttc"),
        bytes.len() as u64,
        &cancelled,
    );
    assert!(reader.offsets().is_err());
    assert!(FileRef::new(bytes).is_err());
}

fn extended_collection(major: u16) -> (Vec<u8>, Vec<u64>) {
    let dsig_bytes = if major == 2 { HEADER_BYTES } else { 0 };
    let first = HEADER_BYTES
        + U32_BYTES
        + dsig_bytes
        + U32_BYTES
        + U32_BYTES * EXTENDED_FACE_COUNT as usize;
    let offsets = vec![(first + HEADER_BYTES) as u64, first as u64];
    let mut bytes = b"ttcf".to_vec();
    bytes.extend_from_slice(&major.to_be_bytes());
    bytes.extend_from_slice(&1u16.to_be_bytes());
    bytes.extend_from_slice(&1u32.to_be_bytes());
    bytes.extend_from_slice(&(first as u32).to_be_bytes());
    bytes.resize(bytes.len() + dsig_bytes, 0);
    bytes.extend_from_slice(&EXTENDED_FACE_COUNT.to_be_bytes());
    for offset in &offsets {
        bytes.extend_from_slice(&(*offset as u32).to_be_bytes());
    }
    for _ in 0..EXTENDED_FACE_COUNT {
        bytes.extend_from_slice(&[0, 1, 0, 0]);
        bytes.resize(bytes.len() + HEADER_BYTES - U32_BYTES, 0);
    }
    (bytes, offsets)
}
