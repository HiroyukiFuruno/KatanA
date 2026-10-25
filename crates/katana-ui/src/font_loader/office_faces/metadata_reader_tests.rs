use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};

use skrifa::{FontRef, MetadataProvider, string::StringId};

use super::super::font_metadata::{face_metadata, matches_request};
use super::super::sfnt_fixture;
use super::super::types::FontFaceResolutionDiagnostic;
use super::FontMetadataReader;

const LARGE_PAYLOAD_BYTES: u64 = 16 * 1024 * 1024;
const MAX_METADATA_READ_BYTES: usize = 64 * 1024;

struct ObservedFile<'a> {
    file: File,
    read_bytes: usize,
    cancel_after_read: Option<&'a AtomicBool>,
}

impl Read for ObservedFile<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        let count = self.file.read(bytes)?;
        self.read_bytes += count;
        if let Some(cancelled) = self.cancel_after_read {
            cancelled.store(true, Ordering::Release);
        }
        Ok(count)
    }
}

impl Seek for ObservedFile<'_> {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        self.file.seek(position)
    }
}

#[test]
fn metadata_scan_does_not_read_large_unselected_payload() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("large.ttf");
    fs::write(&path, sfnt_fixture::embedded_font()).unwrap();
    let mut input = ObservedFile {
        file: File::open(&path).unwrap(),
        read_bytes: 0,
        cancel_after_read: None,
    };
    File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(LARGE_PAYLOAD_BYTES)
        .unwrap();
    let cancelled = AtomicBool::new(false);
    let bytes = FontMetadataReader::new(&mut input, &path, LARGE_PAYLOAD_BYTES, &cancelled)
        .face(0, 0)
        .unwrap();
    let metadata = face_metadata(&FontRef::new(&bytes).unwrap()).unwrap();
    assert!(!matches_request(&metadata, "Absent Family", false, false));
    assert!(input.read_bytes < MAX_METADATA_READ_BYTES);
}

#[test]
fn compact_metadata_preserves_localized_aliases_attributes_and_classification() {
    let original = sfnt_fixture::with_distinct_family_aliases(sfnt_fixture::embedded_font());
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("aliases.ttf");
    fs::write(&path, &original).unwrap();
    let mut file = File::open(&path).unwrap();
    let cancelled = AtomicBool::new(false);
    let bytes = FontMetadataReader::new(&mut file, &path, original.len() as u64, &cancelled)
        .face(0, 0)
        .unwrap();
    let original_face = FontRef::new(&original).unwrap();
    let compact_face = FontRef::new(&bytes).unwrap();
    for id in [StringId::FAMILY_NAME, StringId::TYPOGRAPHIC_FAMILY_NAME] {
        let names = |face: &FontRef<'_>| {
            face.localized_strings(id)
                .map(|name| name.to_string())
                .collect::<Vec<_>>()
        };
        assert_eq!(names(&original_face), names(&compact_face));
    }
    let left = face_metadata(&original_face).unwrap();
    let right = face_metadata(&compact_face).unwrap();
    assert_eq!(
        (
            left.family,
            left.weight,
            left.bold,
            left.italic,
            left.monospaced
        ),
        (
            right.family,
            right.weight,
            right.bold,
            right.italic,
            right.monospaced
        )
    );
}

#[test]
fn malformed_directory_and_collection_counts_are_bounded() {
    for bytes in [
        b"ttcf\0\x01\0\0\xff\xff\xff\xff".to_vec(),
        b"\0\x01\0\0\xff\xff\0\0\0\0\0\0".to_vec(),
    ] {
        let temporary = tempfile::tempdir().unwrap();
        let path = temporary.path().join("malformed.ttf");
        fs::write(&path, &bytes).unwrap();
        let mut file = File::open(&path).unwrap();
        let cancelled = AtomicBool::new(false);
        let mut reader = FontMetadataReader::new(&mut file, &path, bytes.len() as u64, &cancelled);
        let offsets = reader.offsets();
        if offsets.is_ok() {
            let compact = reader.face(0, 0).unwrap();
            assert!(face_metadata(&FontRef::new(&compact).unwrap()).is_none());
        }
        assert!(reader.read_at(u64::MAX, 1).is_err());
    }
}

#[test]
fn metadata_read_observes_cancellation_during_io() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("font.ttf");
    let bytes = sfnt_fixture::embedded_font();
    fs::write(&path, &bytes).unwrap();
    let cancelled = AtomicBool::new(false);
    let mut input = ObservedFile {
        file: File::open(&path).unwrap(),
        read_bytes: 0,
        cancel_after_read: Some(&cancelled),
    };
    let result =
        FontMetadataReader::new(&mut input, &path, bytes.len() as u64, &cancelled).face(0, 0);
    assert!(matches!(
        result,
        Err(FontFaceResolutionDiagnostic::Cancelled)
    ));
    assert_eq!(input.read_bytes, super::SFNT_HEADER_BYTES);
}

#[test]
fn truncated_directory_preserves_full_parser_empty_table_behavior() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("truncated.ttf");
    let bytes = sfnt_fixture::embedded_font();
    fs::write(&path, &bytes[..super::SFNT_HEADER_BYTES]).unwrap();
    let mut file = File::open(&path).unwrap();
    let cancelled = AtomicBool::new(false);
    let result = FontMetadataReader::new(
        &mut file,
        &path,
        super::SFNT_HEADER_BYTES as u64,
        &cancelled,
    )
    .face(0, 0);
    let compact = result.unwrap();
    assert!(face_metadata(&FontRef::new(&compact).unwrap()).is_none());
    assert!(
        FontRef::new(&bytes[..super::SFNT_HEADER_BYTES])
            .unwrap()
            .table_directory()
            .table_records()
            .is_empty()
    );
}
