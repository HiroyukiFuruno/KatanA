use egui::{FontDefinitions, FontFamily};
use std::collections::HashSet;
use ttf_parser::{Face, name_id};

#[path = "tests/named_families.rs"]
#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tests/named_faces.rs"]
mod named_faces;

const TYPOGRAPHIC_FAMILY: u16 = name_id::TYPOGRAPHIC_FAMILY;
const FAMILY: u16 = name_id::FAMILY;
const MAX_REGULAR_WEIGHT: u16 = 500;
const ENGLISH_US_LANGUAGE_ID: u16 = 0x0409;

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
    let face = Face::parse(data.font.as_ref(), data.index).ok()?;
    if face.is_italic() || face.weight().to_number() > MAX_REGULAR_WEIGHT {
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
    Some((name, named_fallback_chain(fonts, key, face.is_monospaced())))
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

fn family_name(face: &Face<'_>) -> Option<String> {
    [TYPOGRAPHIC_FAMILY, FAMILY]
        .into_iter()
        .find_map(|id| name_for_id(face, id))
}

fn name_for_id(face: &Face<'_>, id: u16) -> Option<String> {
    let names: Vec<_> = face
        .names()
        .into_iter()
        .filter(|record| record.name_id == id)
        .collect();
    let english = names
        .iter()
        .find(|record| record.language_id == ENGLISH_US_LANGUAGE_ID);
    english
        .into_iter()
        .chain(names.iter())
        .find_map(valid_family_name)
}

fn valid_family_name(record: &ttf_parser::name::Name<'_>) -> Option<String> {
    record
        .to_string()
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty() && !name.chars().any(char::is_control))
}
