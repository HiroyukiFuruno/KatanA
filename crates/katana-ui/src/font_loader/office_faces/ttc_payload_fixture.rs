use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use skrifa::MetadataProvider;
use skrifa::raw::FileRef;
use tempfile::TempDir;

use super::FontFaceRequest;
use super::font_metadata::face_metadata;

const TTC_TAG_BYTES: usize = 4;
const TTC_VERSION_OFFSET: usize = 4;
const TTC_FACE_COUNT_OFFSET: usize = 8;
const TTC_HEADER_BYTES: usize = 12;
const TTC_OFFSET_BYTES: usize = 4;
const TTC_ALIGNMENT_BYTES: usize = 4;
const TTC_VERSION_1_0: u32 = 65_536;
const SFNT_TABLE_COUNT_OFFSET: usize = 4;
const SFNT_TABLE_COUNT_BYTES: usize = 2;
const SFNT_TABLE_DIRECTORY_OFFSET: usize = 12;
const SFNT_TABLE_RECORD_BYTES: usize = 16;
const SFNT_TABLE_OFFSET_FIELD: usize = 8;
const SFNT_U32_BYTES: usize = 4;

pub(super) struct TtcFixture {
    pub(super) _temporary: TempDir,
    pub(super) path: PathBuf,
    pub(super) bytes: Vec<u8>,
    pub(super) requests: Vec<FontFaceRequest>,
    pub(super) source_indices: BTreeMap<FontFaceRequest, u32>,
}

pub(super) fn build_fixture() -> TtcFixture {
    let (family, regular_bytes, bold_bytes) = installed_pair_bytes();
    let bytes = build_ttc(&[regular_bytes, bold_bytes]);
    let temporary = tempfile::tempdir().expect("temporary TTC directory");
    let path = temporary.path().join("installed-pair.ttc");
    fs::write(&path, &bytes).expect("write TTC fixture");
    let file = FileRef::new(&bytes).expect("constructed TTC fixture is valid");
    let selected = select_faces(file);
    assert_fixture_faces(&selected, &family);
    let requests = selected
        .iter()
        .map(|(request, _)| request.clone())
        .collect();
    let source_indices = selected.into_iter().collect();
    TtcFixture {
        _temporary: temporary,
        path,
        bytes,
        requests,
        source_indices,
    }
}

fn installed_pair_bytes() -> (String, Vec<u8>, Vec<u8>) {
    let (family, regular, bold) = super::installed_regular_bold_pair();
    (
        family,
        fs::read(regular).expect("read installed regular font"),
        fs::read(bold).expect("read installed bold font"),
    )
}

fn select_faces(file: FileRef<'_>) -> Vec<(FontFaceRequest, u32)> {
    let mut selected = Vec::new();
    for (index, parsed) in file.fonts().enumerate() {
        let Ok(font) = parsed else { continue };
        let Some(metadata) = face_metadata(&font) else {
            continue;
        };
        let Some(glyph) = font.charmap().map('A') else {
            continue;
        };
        if font.outline_glyphs().get(glyph).is_none() {
            continue;
        }
        selected.push((
            FontFaceRequest {
                family: metadata.family,
                bold: metadata.bold,
                italic: metadata.italic,
            },
            u32::try_from(index).expect("TTC face index fits u32"),
        ));
    }
    selected.dedup_by(|left, right| left.0 == right.0);
    selected
}

fn assert_fixture_faces(selected: &[(FontFaceRequest, u32)], family: &str) {
    assert_eq!(selected.len(), 2, "constructed TTC must expose two faces");
    assert!(selected.iter().any(|(request, index)| {
        request.family == family && !request.bold && !request.italic && *index == 0
    }));
}

fn build_ttc(fonts: &[Vec<u8>]) -> Vec<u8> {
    let mut output = vec![0; TTC_HEADER_BYTES + fonts.len() * TTC_OFFSET_BYTES];
    output[..TTC_TAG_BYTES].copy_from_slice(b"ttcf");
    output[TTC_VERSION_OFFSET..TTC_FACE_COUNT_OFFSET]
        .copy_from_slice(&TTC_VERSION_1_0.to_be_bytes());
    output[TTC_FACE_COUNT_OFFSET..TTC_HEADER_BYTES]
        .copy_from_slice(&(fonts.len() as u32).to_be_bytes());
    for (index, font) in fonts.iter().enumerate() {
        while output.len() % TTC_ALIGNMENT_BYTES != 0 {
            output.push(0);
        }
        let base = output.len();
        let offset = TTC_HEADER_BYTES + index * TTC_OFFSET_BYTES;
        output[offset..offset + TTC_OFFSET_BYTES].copy_from_slice(&(base as u32).to_be_bytes());
        output.extend_from_slice(font);
        relocate_tables(&mut output, base, font);
    }
    output
}

fn relocate_tables(output: &mut [u8], base: usize, font: &[u8]) {
    let tables = u16::from_be_bytes(
        font[SFNT_TABLE_COUNT_OFFSET..SFNT_TABLE_COUNT_OFFSET + SFNT_TABLE_COUNT_BYTES]
            .try_into()
            .expect("sfnt table count is two bytes"),
    ) as usize;
    for table in 0..tables {
        let offset = base
            + SFNT_TABLE_DIRECTORY_OFFSET
            + table * SFNT_TABLE_RECORD_BYTES
            + SFNT_TABLE_OFFSET_FIELD;
        let old = u32::from_be_bytes(
            output[offset..offset + SFNT_U32_BYTES]
                .try_into()
                .expect("sfnt table offset is four bytes"),
        );
        output[offset..offset + SFNT_U32_BYTES].copy_from_slice(&(old + base as u32).to_be_bytes());
    }
}
