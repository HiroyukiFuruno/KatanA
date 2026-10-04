use skrifa::raw::tables::os2::SelectionFlags;
use skrifa::{FontRef, MetadataProvider, attribute::Style, raw::TableProvider, string::StringId};

use super::types::{BOLD_WEIGHT, BOLD_WEIGHT_THRESHOLD, MAX_FONT_WEIGHT, REGULAR_WEIGHT};

#[derive(Clone)]
pub(super) struct FaceMetadata {
    pub(super) family: String,
    family_aliases: Vec<String>,
    pub(super) weight: u16,
    pub(super) bold: bool,
    pub(super) italic: bool,
    pub(super) monospaced: bool,
}

pub(super) fn face_metadata(face: &FontRef<'_>) -> Option<FaceMetadata> {
    let family_aliases = family_names(face)?;
    let family = family_aliases.first()?.clone();
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
        family_aliases,
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
    metadata
        .family_aliases
        .iter()
        .any(|alias| alias.eq_ignore_ascii_case(family))
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

fn family_names(face: &FontRef<'_>) -> Option<Vec<String>> {
    let mut names = Vec::new();
    for id in [StringId::TYPOGRAPHIC_FAMILY_NAME, StringId::FAMILY_NAME] {
        let records = face.localized_strings(id);
        if let Some(record) = records.clone().english_or_first() {
            add_valid_name(&mut names, record.to_string());
        }
        for record in records {
            add_valid_name(&mut names, record.to_string());
        }
    }
    (!names.is_empty()).then_some(names)
}

fn add_valid_name(names: &mut Vec<String>, raw: String) {
    let name = raw.trim();
    if !name.is_empty()
        && !name.chars().any(char::is_control)
        && !names.iter().any(|known| known.eq_ignore_ascii_case(name))
    {
        names.push(name.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::add_valid_name;

    #[test]
    fn valid_name_filter_rejects_invalid_and_case_duplicate_aliases() {
        let mut names = Vec::new();
        add_valid_name(&mut names, " Alias ".into());
        add_valid_name(&mut names, "ALIAS".into());
        add_valid_name(&mut names, "".into());
        add_valid_name(&mut names, "\u{0000}invalid".into());

        assert_eq!(names, ["Alias"]);
    }
}
