use skrifa::FontRef;
use write_fonts::{FontBuilder, types::Tag};

use super::{FacePayload, checked_face_size, extract_face};
use crate::font_loader::office_faces::types::MAX_FONT_PAYLOAD_BYTES;

const TAG_LENGTH: usize = 4;

#[test]
fn standalone_payload_moves_its_original_allocation_once() {
    let bytes = font_bytes(&[(b"test", b"original")]);
    let pointer = bytes.as_ptr();
    let mut payload = FacePayload::new(bytes);
    assert!(payload.take_face(1).is_err());
    let face = payload.take_face(0).expect("standalone face");
    assert_eq!(face.font.as_ptr(), pointer);
    assert_eq!(face.index, 0);
    assert!(payload.take_face(0).is_err());
}

#[test]
fn malformed_font_payload_is_rejected() {
    assert!(FacePayload::new(b"invalid".to_vec()).take_face(0).is_err());
}

#[test]
fn extracted_face_rejects_duplicate_tags_and_unreadable_table_ranges() {
    let mut bytes = font_bytes(&[(b"aaaa", b"first"), (b"bbbb", b"second")]);
    bytes[28..32].copy_from_slice(b"aaaa");
    let font = FontRef::new(&bytes).expect("duplicate directory parses");
    assert!(extract_face(&font).is_err());
    let mut bytes = font_bytes(&[(b"test", b"valid")]);
    bytes[24..28].copy_from_slice(&u32::MAX.to_be_bytes());
    let font = FontRef::new(&bytes).expect("table directory parses");
    assert!(extract_face(&font).is_err());
}

#[test]
fn extracted_face_rejects_truncated_head_and_unrepresentable_table_count() {
    let bytes = font_bytes(&[(b"head", b"short")]);
    let font = FontRef::new(&bytes).expect("short head directory parses");
    assert!(extract_face(&font).is_err());
    let mut bytes = vec![0; 12 + 4096 * 16];
    bytes[..4].copy_from_slice(&0x0001_0000_u32.to_be_bytes());
    bytes[4..6].copy_from_slice(&4096_u16.to_be_bytes());
    let font = FontRef::new(&bytes).expect("large directory parses");
    assert!(extract_face(&font).is_err());
}

#[test]
fn selected_table_budget_includes_padding_and_rejects_overflow() {
    assert_eq!(checked_face_size(12, 1), Ok(16));
    assert_eq!(
        checked_face_size(MAX_FONT_PAYLOAD_BYTES, 0),
        Ok(MAX_FONT_PAYLOAD_BYTES)
    );
    assert!(checked_face_size(MAX_FONT_PAYLOAD_BYTES, 1).is_err());
    assert!(checked_face_size(u64::MAX, 4).is_err());
    assert!(checked_face_size(0, usize::MAX).is_err());
}

#[test]
fn extracted_face_rejects_overlapping_tables_exceeding_the_payload_budget() {
    let table = vec![0; (MAX_FONT_PAYLOAD_BYTES / 2) as usize];
    let mut bytes = font_bytes(&[(b"aaaa", &table), (b"bbbb", b"small")]);
    let offset = bytes[20..24].to_vec();
    bytes[36..40].copy_from_slice(&offset);
    bytes[40..44].copy_from_slice(&(table.len() as u32).to_be_bytes());
    let font = FontRef::new(&bytes).expect("overlapping directory parses");
    assert!(extract_face(&font).is_err());
}

#[test]
fn cff_tables_and_flavor_survive_face_extraction() {
    let bytes = font_bytes(&[(b"CFF2", b"outlines"), (b"GSUB", b"shaping")]);
    let font = FontRef::new(&bytes).expect("CFF directory parses");
    let extracted = extract_face(&font).expect("all CFF tables can be copied");
    assert_eq!(extracted.get(..4), Some(&b"OTTO"[..]));
    let font = FontRef::new(&extracted).expect("standalone CFF directory parses");
    assert_eq!(
        font.table_data(Tag::new(b"GSUB"))
            .expect("shaping table")
            .as_bytes(),
        b"shaping"
    );
}

fn font_bytes(tables: &[(&[u8; TAG_LENGTH], &[u8])]) -> Vec<u8> {
    let mut builder = FontBuilder::new();
    for (tag, bytes) in tables {
        builder.add_raw(Tag::new(tag), *bytes);
    }
    builder.build()
}
