use std::collections::BTreeMap;
use std::sync::Arc;

use skrifa::MetadataProvider;

use super::FontFaceRequest;
use super::FontFamilyIdentity;
use super::font_metadata::{face_metadata, matches_request};
use super::types::{FontFaceResolution, ResolvedFontFace};
const GLYPH_BOUNDS_COMPONENTS: usize = 4;

const SFNT_TAG_BYTES: usize = 4;
const SFNT_TABLE_COUNT_OFFSET: usize = 4;
const SFNT_TABLE_COUNT_BYTES: usize = 2;
const SFNT_TABLE_DIRECTORY_OFFSET: usize = 12;
const SFNT_TABLE_RECORD_BYTES: usize = 16;
const SFNT_TABLE_OFFSET_FIELD: usize = 8;
const SFNT_TABLE_LENGTH_FIELD: usize = 12;
const TTC_OFFSET_TABLE_OFFSET: usize = 12;
const TTC_OFFSET_BYTES: usize = 4;
const TTC_FACE_COUNT_OFFSET: usize = 8;
const HEAD_CHECKSUM_OFFSET: usize = 8;
const HEAD_CHECKSUM_END: usize = 12;

pub(super) fn assert_resolution(
    report: FontFaceResolution,
    bytes: &[u8],
    requests: &[FontFaceRequest],
    source_indices: &BTreeMap<FontFaceRequest, u32>,
) {
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    assert_eq!(report.faces.len(), requests.len());
    let expected = expected_indices(requests, source_indices);
    for resolved in &report.faces {
        assert_face(resolved, &expected, bytes);
    }
    assert_retained_and_released(report, bytes);
}

fn expected_indices(
    requests: &[FontFaceRequest],
    source_indices: &BTreeMap<FontFaceRequest, u32>,
) -> BTreeMap<(String, bool, bool), u32> {
    requests
        .iter()
        .map(|request| (request_identity(request), source_indices[request]))
        .collect()
}

fn request_identity(request: &FontFaceRequest) -> (String, bool, bool) {
    (
        FontFamilyIdentity::key(&request.family),
        request.bold,
        request.italic,
    )
}

fn assert_retained_and_released(report: FontFaceResolution, bytes: &[u8]) {
    let retained: usize = report
        .faces
        .iter()
        .map(|face| face.payload.font.len())
        .sum();
    assert!(retained < bytes.len() * report.faces.len());
    eprintln!(
        "TTC payload: source={} faces={} old_retained={} extracted_retained={retained}",
        bytes.len(),
        report.faces.len(),
        bytes.len() * report.faces.len()
    );
    let weak_payloads: Vec<_> = report
        .faces
        .iter()
        .map(|face| Arc::downgrade(&face.payload))
        .collect();
    drop(report);
    assert!(
        weak_payloads
            .iter()
            .all(|payload| payload.upgrade().is_none())
    );
}

fn assert_face(
    resolved: &ResolvedFontFace,
    expected: &BTreeMap<(String, bool, bool), u32>,
    source: &[u8],
) {
    let key = request_identity(&resolved.request);
    let source_index = expected[&key];
    assert_eq!(resolved.face_index, source_index);
    assert_eq!(resolved.payload.index, 0);
    assert_ne!(
        resolved.payload.font.as_ref().get(..SFNT_TAG_BYTES),
        Some(&b"ttcf"[..])
    );
    let extracted = skrifa::FontRef::from_index(resolved.payload.font.as_ref(), 0)
        .expect("resolved payload contains selected face at index zero");
    assert_tables(source, source_index, resolved.payload.font.as_ref());
    assert_glyphs(source, source_index, &extracted);
    let metadata = face_metadata(&extracted).expect("selected face metadata");
    assert!(matches_request(
        &metadata,
        &resolved.request.family,
        resolved.request.bold,
        resolved.request.italic,
    ));
}

fn assert_glyphs(source: &[u8], index: u32, extracted: &skrifa::FontRef<'_>) {
    let source = skrifa::FontRef::from_index(source, index).expect("source face index");
    assert_eq!(glyph_metrics(extracted), glyph_metrics(&source));
}

fn glyph_metrics(font: &skrifa::FontRef<'_>) -> (f32, [f32; GLYPH_BOUNDS_COMPONENTS]) {
    use skrifa::outline::{DrawSettings, pen::ControlBoundsPen};
    use skrifa::prelude::{LocationRef, Size};
    let glyph = font.charmap().map('A').expect("font maps A");
    let advance = font
        .glyph_metrics(Size::unscaled(), LocationRef::default())
        .advance_width(glyph)
        .expect("glyph advance");
    let outline = font.outline_glyphs().get(glyph).expect("glyph outline");
    let mut pen = ControlBoundsPen::new();
    outline
        .draw(
            DrawSettings::unhinted(Size::unscaled(), LocationRef::default()),
            &mut pen,
        )
        .expect("draw glyph outline");
    let bounds = pen.bounding_box().expect("glyph bounds");
    (
        advance,
        [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
    )
}

fn assert_tables(source: &[u8], source_index: u32, extracted: &[u8]) {
    let source_tables = sfnt_tables(source, sfnt_offsets(source)[source_index as usize]);
    let extracted_tables = sfnt_tables(extracted, 0);
    assert_eq!(source_tables.len(), extracted_tables.len());
    for (tag, mut expected) in source_tables {
        let actual = extracted_tables.get(&tag).expect("selected table retained");
        if &tag == b"head" {
            expected[HEAD_CHECKSUM_OFFSET..HEAD_CHECKSUM_END].fill(0);
        }
        let mut actual = actual.clone();
        if &tag == b"head" {
            actual[HEAD_CHECKSUM_OFFSET..HEAD_CHECKSUM_END].fill(0);
        }
        assert_eq!(actual, expected, "table {:?} changed", tag);
    }
}

fn sfnt_offsets(bytes: &[u8]) -> Vec<usize> {
    let count = u32::from_be_bytes(
        bytes[TTC_FACE_COUNT_OFFSET..HEAD_CHECKSUM_END]
            .try_into()
            .expect("TTC count is four bytes"),
    );
    (0..count)
        .map(|index| {
            let offset = TTC_OFFSET_TABLE_OFFSET + index as usize * TTC_OFFSET_BYTES;
            u32::from_be_bytes(
                bytes[offset..offset + TTC_OFFSET_BYTES]
                    .try_into()
                    .expect("TTC offset is four bytes"),
            ) as usize
        })
        .collect()
}

fn sfnt_tables(bytes: &[u8], base: usize) -> BTreeMap<[u8; SFNT_TAG_BYTES], Vec<u8>> {
    let count = u16::from_be_bytes(
        bytes[base + SFNT_TABLE_COUNT_OFFSET
            ..base + SFNT_TABLE_COUNT_OFFSET + SFNT_TABLE_COUNT_BYTES]
            .try_into()
            .expect("table count is two bytes"),
    );
    (0..count)
        .map(|index| {
            let entry =
                base + SFNT_TABLE_DIRECTORY_OFFSET + index as usize * SFNT_TABLE_RECORD_BYTES;
            let tag = bytes[entry..entry + SFNT_TAG_BYTES]
                .try_into()
                .expect("table tag is four bytes");
            let offset = u32::from_be_bytes(
                bytes[entry + SFNT_TABLE_OFFSET_FIELD..entry + SFNT_TABLE_LENGTH_FIELD]
                    .try_into()
                    .expect("table offset is four bytes"),
            ) as usize;
            let length = u32::from_be_bytes(
                bytes[entry + SFNT_TABLE_LENGTH_FIELD..entry + SFNT_TABLE_RECORD_BYTES]
                    .try_into()
                    .expect("table length is four bytes"),
            ) as usize;
            (tag, bytes[offset..offset + length].to_vec())
        })
        .collect()
}
