use skrifa::raw::tables::os2::SelectionFlags;
use skrifa::{FontRef, MetadataProvider, attribute::Style, raw::TableProvider, string::StringId};

use super::types::{BOLD_WEIGHT, BOLD_WEIGHT_THRESHOLD, MAX_FONT_WEIGHT, REGULAR_WEIGHT};

pub(super) struct FaceMetadata {
    pub(super) family: String,
    pub(super) weight: u16,
    pub(super) bold: bool,
    pub(super) italic: bool,
    pub(super) monospaced: bool,
}

pub(super) fn face_metadata(face: &FontRef<'_>) -> Option<FaceMetadata> {
    let family = family_name(face)?;
    let attributes = face.attributes();
    let weight = attributes
        .weight
        .value()
        .round()
        .clamp(0.0, MAX_FONT_WEIGHT) as u16;
    let (flagged_bold, flagged_italic) = os2_style_flags(face);
    let post = face.post().ok();
    let post_italic = post
        .as_ref()
        .is_some_and(|table| table.italic_angle().to_f64() != 0.0);
    let monospaced = post.is_some_and(|table| table.is_fixed_pitch() != 0);
    Some(FaceMetadata {
        family,
        weight,
        bold: weight >= BOLD_WEIGHT_THRESHOLD || flagged_bold,
        italic: matches!(attributes.style, Style::Italic | Style::Oblique(_))
            || flagged_italic
            || post_italic,
        monospaced,
    })
}

pub(super) fn matches_request(
    metadata: &FaceMetadata,
    family: &str,
    bold: bool,
    italic: bool,
) -> bool {
    metadata.family.eq_ignore_ascii_case(family)
        && metadata.bold == bold
        && metadata.italic == italic
}

pub(super) fn weight_distance(metadata: &FaceMetadata, bold: bool) -> u16 {
    let target = if bold { BOLD_WEIGHT } else { REGULAR_WEIGHT };
    metadata.weight.abs_diff(target)
}

fn os2_style_flags(face: &FontRef<'_>) -> (bool, bool) {
    face.os2()
        .ok()
        .map(|table| {
            let flags = table.fs_selection();
            (
                flags.contains(SelectionFlags::BOLD),
                flags.contains(SelectionFlags::ITALIC),
            )
        })
        .unwrap_or_default()
}

fn family_name(face: &FontRef<'_>) -> Option<String> {
    [StringId::TYPOGRAPHIC_FAMILY_NAME, StringId::FAMILY_NAME]
        .into_iter()
        .find_map(|id| family_name_for_id(face, id))
}

fn family_name_for_id(face: &FontRef<'_>, id: StringId) -> Option<String> {
    face.localized_strings(id)
        .clone()
        .english_or_first()
        .into_iter()
        .chain(face.localized_strings(id))
        .map(|record| record.to_string().trim().to_owned())
        .find(|name| !name.is_empty() && !name.chars().any(char::is_control))
}
