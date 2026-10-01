use egui::{FontDefinitions, FontFamily};
use skrifa::{FontRef, MetadataProvider, attribute::Style, raw::TableProvider, string::StringId};
use std::collections::HashSet;

#[path = "tests/named_families.rs"]
#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tests/named_faces.rs"]
mod named_faces;

const MAX_REGULAR_WEIGHT: f32 = 500.0;

#[derive(Debug)]
pub(super) struct NamedFontFamiliesOps;

impl NamedFontFamiliesOps {
    pub(super) fn register(fonts: &mut FontDefinitions) {
        let started = std::time::Instant::now();
        let mut registered_names = HashSet::new();
        let mut aliases = Vec::new();
        for key in ordered_font_keys(fonts) {
            if let Some(alias) = candidate_alias(fonts, &key, &registered_names) {
                registered_names.insert(alias.0.clone());
                aliases.push(alias);
            }
        }
        for (name, chain) in aliases {
            fonts.families.insert(FontFamily::Name(name.into()), chain);
        }
        crate::debug_log::DebugLog::write(
            "font_families_registered",
            format_args!(
                "added={} elapsed_us={}",
                registered_names.len(),
                started.elapsed().as_micros()
            ),
        );
    }
}

fn ordered_font_keys(fonts: &FontDefinitions) -> Vec<String> {
    [FontFamily::Proportional, FontFamily::Monospace]
        .into_iter()
        .filter_map(|family| fonts.families.get(&family))
        .flatten()
        .cloned()
        .collect()
}

fn candidate_alias(
    fonts: &FontDefinitions,
    key: &str,
    registered_names: &HashSet<String>,
) -> Option<(String, Vec<String>)> {
    let data = fonts.font_data.get(key)?;
    let face = FontRef::from_index(data.font.as_ref(), data.index).ok()?;
    let attributes = face.attributes();
    if attributes.style != Style::Normal || attributes.weight.value() > MAX_REGULAR_WEIGHT {
        return None;
    }
    let name = family_name(&face)?;
    if fonts
        .families
        .contains_key(&FontFamily::Name(name.clone().into()))
        || registered_names.contains(&name)
    {
        return None;
    }
    let monospaced = face.post().is_ok_and(|post| post.is_fixed_pitch() != 0);
    Some((name, named_fallback_chain(fonts, key, monospaced)))
}

fn named_fallback_chain(fonts: &FontDefinitions, key: &str, monospaced: bool) -> Vec<String> {
    let family = if monospaced {
        FontFamily::Monospace
    } else {
        FontFamily::Proportional
    };
    let mut chain = vec![key.to_owned()];
    if let Some(keys) = fonts.families.get(&family) {
        chain.extend(
            keys.iter()
                .filter(|candidate| candidate.as_str() != key)
                .cloned(),
        );
    }
    chain
}

fn family_name(face: &FontRef<'_>) -> Option<String> {
    [StringId::TYPOGRAPHIC_FAMILY_NAME, StringId::FAMILY_NAME]
        .into_iter()
        .find_map(|id| name_for_id(face, id))
}

fn name_for_id(face: &FontRef<'_>, id: StringId) -> Option<String> {
    let names = face.localized_strings(id);
    names
        .clone()
        .english_or_first()
        .into_iter()
        .chain(names)
        .find_map(valid_family_name)
}

fn valid_family_name(record: skrifa::string::LocalizedString<'_>) -> Option<String> {
    let name = record.to_string().trim().to_owned();
    (!name.is_empty() && !name.chars().any(char::is_control)).then_some(name)
}
