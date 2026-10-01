use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use skrifa::MetadataProvider;

const GLYPH_BOUNDS_COMPONENTS: usize = 4;
use skrifa::outline::{DrawSettings, pen::ControlBoundsPen};
use skrifa::prelude::{LocationRef, Size};

use super::font_metadata::{FaceMetadata, face_metadata};
use super::resolver::FontFaceResolver;
use super::types::FontFaceRequest;

#[test]
fn installed_italic_helper_returns_named_italic_face_with_usable_glyph() {
    let (family, path) = installed_italic_face();
    let bytes = fs::read(path).expect("read installed italic font");
    let font = skrifa::FontRef::new(&bytes).expect("installed italic font is valid");
    let metadata = face_metadata(&font).expect("installed italic font has family metadata");
    let glyph_id = font.charmap().map('A').expect("italic face maps A");

    assert_eq!(metadata.family, family);
    assert!(metadata.italic);
    assert!(!metadata.bold);
    assert!(font.outline_glyphs().get(glyph_id).is_some());
}

#[test]
fn installed_bold_face_is_selected_with_usable_glyph_distinct_from_regular() {
    let (family, regular_path, bold_path) = installed_regular_bold_pair();
    let bytes = fs::read(&bold_path).expect("read installed bold font");
    let source = skrifa::FontRef::new(&bytes).expect("installed font is valid");
    let metadata = face_metadata(&source).expect("installed font has family metadata");
    assert_eq!(metadata.family, family);
    let request = FontFaceRequest {
        family,
        bold: true,
        italic: false,
    };
    let candidates = vec![(
        "ignored filename stem".into(),
        bold_path.to_string_lossy().into_owned(),
    )];
    let report = FontFaceResolver::resolve(&candidates, &[request], &AtomicBool::new(false));

    assert!(report.diagnostics.is_empty());
    assert_eq!(report.faces.len(), 1);
    let resolved = &report.faces[0];
    assert_eq!(resolved.path, bold_path);
    assert!(resolved.bold);
    assert_eq!(
        resolved.weight,
        source.attributes().weight.value().round() as u16
    );
    assert_eq!(resolved.payload.font.as_ref(), bytes.as_slice());
    assert_glyph_matches_payload(resolved);
    assert_ne!(glyph_bounds(&regular_path), glyph_bounds(&bold_path));
}

pub(super) fn installed_regular_bold_pair() -> (String, PathBuf, PathBuf) {
    font_pairs()
        .into_iter()
        .find_map(|(regular, bold)| matching_pair(regular, bold))
        .expect("install DejaVu Sans, Liberation Sans, Verdana, or Windows Arial fonts")
}

fn font_pairs() -> Vec<(PathBuf, PathBuf)> {
    let mut pairs = vec![
        pair(
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
        ),
        pair(
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        ),
        pair(
            "/usr/share/fonts/truetype/liberation2/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/liberation2/LiberationSans-Bold.ttf",
        ),
        pair(
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
        ),
    ];
    if let Some(windows_dir) = std::env::var_os("WINDIR") {
        let fonts = PathBuf::from(windows_dir).join("Fonts");
        pairs.push((fonts.join("arial.ttf"), fonts.join("arialbd.ttf")));
    }
    pairs.push(pair(
        "/System/Library/Fonts/Supplemental/Verdana.ttf",
        "/System/Library/Fonts/Supplemental/Verdana Bold.ttf",
    ));
    pairs
}

fn pair(regular: &str, bold: &str) -> (PathBuf, PathBuf) {
    (PathBuf::from(regular), PathBuf::from(bold))
}

fn matching_pair(regular_path: PathBuf, bold_path: PathBuf) -> Option<(String, PathBuf, PathBuf)> {
    let regular = face_metadata_at(&regular_path)?;
    let bold = face_metadata_at(&bold_path)?;
    (regular.family.eq_ignore_ascii_case(&bold.family)
        && !regular.bold
        && bold.bold
        && !regular.italic
        && !bold.italic)
        .then_some((bold.family, regular_path, bold_path))
}

pub(super) fn installed_italic_face() -> (String, PathBuf) {
    italic_candidates()
        .into_iter()
        .find_map(matching_italic_face)
        .expect("install Arial Italic, DejaVu Sans Oblique, or Liberation Sans Italic")
}

fn italic_candidates() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("/System/Library/Fonts/Supplemental/Arial Italic.ttf"),
        PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSans-Oblique.ttf"),
        PathBuf::from("/usr/share/fonts/truetype/liberation2/LiberationSans-Italic.ttf"),
        PathBuf::from("/usr/share/fonts/truetype/liberation/LiberationSans-Italic.ttf"),
    ];
    if let Some(windows_dir) = std::env::var_os("WINDIR") {
        paths.push(PathBuf::from(windows_dir).join("Fonts").join("ariali.ttf"));
    }
    paths
}

fn matching_italic_face(path: PathBuf) -> Option<(String, PathBuf)> {
    let bytes = fs::read(&path).ok()?;
    let font = skrifa::FontRef::new(&bytes).ok()?;
    let metadata = face_metadata(&font)?;
    let glyph_id = font.charmap().map('A')?;
    font.outline_glyphs().get(glyph_id)?;
    (metadata.italic && !metadata.bold).then_some((metadata.family, path))
}

fn face_metadata_at(path: &Path) -> Option<FaceMetadata> {
    let bytes = fs::read(path).ok()?;
    let font = skrifa::FontRef::new(&bytes).ok()?;
    face_metadata(&font)
}

fn assert_glyph_matches_payload(resolved: &super::types::ResolvedFontFace) {
    let font = skrifa::FontRef::from_index(resolved.payload.font.as_ref(), resolved.face_index)
        .expect("resolved face index is valid");
    let glyph_id = font.charmap().map('A').expect("bold face maps A");
    assert!(
        font.outline_glyphs().get(glyph_id).is_some(),
        "bold glyph has an outline"
    );
}

fn glyph_bounds(path: &Path) -> [f32; GLYPH_BOUNDS_COMPONENTS] {
    let bytes = fs::read(path).expect("read font pair member");
    let font = skrifa::FontRef::new(&bytes).expect("font pair member is valid");
    let glyph_id = font.charmap().map('A').expect("font maps A");
    let glyph = font.outline_glyphs().get(glyph_id).expect("A has outline");
    let mut pen = ControlBoundsPen::new();
    glyph
        .draw(
            DrawSettings::unhinted(Size::unscaled(), LocationRef::default()),
            &mut pen,
        )
        .expect("draw A outline");
    let bounds = pen.bounding_box().expect("A outline bounds");
    [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max]
}
