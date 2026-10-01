use std::fs;
use std::sync::atomic::AtomicBool;

use egui::FontDefinitions;
use skrifa::MetadataProvider;

use super::resolver::FontFaceResolver;
use super::types::{BOLD_WEIGHT, FontFaceRequest, REGULAR_WEIGHT};

const SEMIBOLD_WEIGHT: u16 = 600;
const OS2_BOLD_SELECTION: u16 = 1 << 5;
const OS2_ITALIC_SELECTION: u16 = 1;
const OS2_WEIGHT_OFFSET: usize = 4;
const OS2_SELECTION_OFFSET: usize = 62;
const OS2_FIELD_BYTES: usize = 2;

#[test]
fn metadata_only_fixture_prefers_exact_bold_weight_over_semibold() {
    let bytes = embedded_font();
    let family = family_name(&skrifa::FontRef::new(&bytes).expect("embedded font"));
    let temporary = tempfile::tempdir().expect("temporary directory");
    let candidates = write_metadata_variants(temporary.path(), &bytes);
    let report = FontFaceResolver::resolve(
        &candidates,
        &[request(&family, true, false)],
        &AtomicBool::new(false),
    );

    assert_eq!(report.faces.len(), 1);
    let selected = &report.faces[0];
    assert!(selected.bold);
    assert_eq!(selected.weight, BOLD_WEIGHT);
    assert!(selected.path.ends_with("bold.ttf"));
}

#[test]
fn metadata_only_fixture_keeps_selected_weight_and_italic_classification() {
    let bytes = embedded_font();
    let family = family_name(&skrifa::FontRef::new(&bytes).expect("embedded font"));
    let temporary = tempfile::tempdir().expect("temporary directory");
    let candidates = write_metadata_variants(temporary.path(), &bytes);
    let report = FontFaceResolver::resolve(
        &candidates,
        &[request(&family, false, true)],
        &AtomicBool::new(false),
    );

    assert_eq!(report.faces.len(), 1);
    let face = &report.faces[0];
    assert_eq!(face.weight, REGULAR_WEIGHT);
    assert!(!face.bold);
    assert!(face.italic);
    assert!(face.path.ends_with("italic.ttf"));
}

#[test]
fn metadata_only_nonexact_bold_keeps_its_actual_weight() {
    let bytes = embedded_font();
    let family = family_name(&skrifa::FontRef::new(&bytes).expect("embedded font"));
    let temporary = tempfile::tempdir().expect("temporary directory");
    let candidates = write_metadata_variants(temporary.path(), &bytes)
        .into_iter()
        .filter(|(_, path)| path.ends_with("semibold.ttf"))
        .collect::<Vec<_>>();
    let report = FontFaceResolver::resolve(
        &candidates,
        &[request(&family, true, false)],
        &AtomicBool::new(false),
    );

    assert_eq!(report.faces.len(), 1);
    assert!(report.faces[0].bold);
    assert_eq!(report.faces[0].weight, SEMIBOLD_WEIGHT);
}

fn write_metadata_variants(root: &std::path::Path, bytes: &[u8]) -> Vec<(String, String)> {
    [
        ("regular.ttf", REGULAR_WEIGHT, 0),
        ("semibold.ttf", SEMIBOLD_WEIGHT, OS2_BOLD_SELECTION),
        ("bold.ttf", BOLD_WEIGHT, OS2_BOLD_SELECTION),
        ("italic.ttf", REGULAR_WEIGHT, OS2_ITALIC_SELECTION),
    ]
    .into_iter()
    .map(|(filename, weight, selection)| {
        let path = root.join(filename);
        fs::write(&path, with_os2_style(bytes, weight, selection)).expect("write metadata fixture");
        (
            "unrelated filename stem".into(),
            path.to_string_lossy().into_owned(),
        )
    })
    .collect()
}

fn request(family: &str, bold: bool, italic: bool) -> FontFaceRequest {
    FontFaceRequest {
        family: family.into(),
        bold,
        italic,
    }
}

fn embedded_font() -> Vec<u8> {
    FontDefinitions::default().font_data["Ubuntu-Light"]
        .font
        .to_vec()
}

fn family_name(font: &skrifa::FontRef<'_>) -> String {
    use skrifa::string::StringId;
    [StringId::TYPOGRAPHIC_FAMILY_NAME, StringId::FAMILY_NAME]
        .into_iter()
        .find_map(|id| {
            font.localized_strings(id)
                .english_or_first()
                .map(|record| record.to_string())
        })
        .expect("embedded family name")
}

fn with_os2_style(bytes: &[u8], weight: u16, selection: u16) -> Vec<u8> {
    let mut changed = bytes.to_vec();
    let face = skrifa::FontRef::new(bytes).expect("embedded font");
    let offset = face
        .table_directory()
        .table_records()
        .iter()
        .find(|record| record.tag() == skrifa::Tag::new(b"OS/2"))
        .map(|record| record.offset() as usize)
        .expect("OS/2 table");
    changed[offset + OS2_WEIGHT_OFFSET..offset + OS2_WEIGHT_OFFSET + OS2_FIELD_BYTES]
        .copy_from_slice(&weight.to_be_bytes());
    changed[offset + OS2_SELECTION_OFFSET..offset + OS2_SELECTION_OFFSET + OS2_FIELD_BYTES]
        .copy_from_slice(&selection.to_be_bytes());
    changed
}
