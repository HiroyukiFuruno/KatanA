use crate::utils::{locale_violation, parse_json_file};
use crate::Violation;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::discovery::locale_code_from_path;

pub fn parse_languages_catalog(locale_dir: &Path) -> Result<BTreeSet<String>, Vec<Violation>> {
    let path = locale_dir.join("languages.json");
    let value = parse_json_file(&path)?;
    let Value::Array(entries) = value else {
        return Err(vec![locale_violation(
            &path,
            "languages.json must be a JSON array.".to_string(),
        )]);
    };

    let mut codes = BTreeSet::new();
    let mut violations = Vec::new();

    for (index, entry) in entries.iter().enumerate() {
        match validate_catalog_entry(entry, &path, index) {
            Ok(code) => {
                if !codes.insert(code.clone()) {
                    violations.push(locale_violation(
                        &path,
                        format!("languages.json contains duplicate code `{code}`."),
                    ));
                }
            }
            Err(violation) => violations.push(violation),
        }
    }

    if violations.is_empty() { Ok(codes) } else { Err(violations) }
}

fn validate_catalog_entry(entry: &Value, path: &Path, index: usize) -> Result<String, Violation> {
    let Value::Object(entry_obj) = entry else {
        return Err(locale_violation(path, format!("languages.json entry at index {index} must be an object.")));
    };

    let Some(code_value) = entry_obj.get("code") else {
        return Err(locale_violation(path, format!("languages.json entry at index {index} is missing `code`.")));
    };
    let Some(name_value) = entry_obj.get("name") else {
        return Err(locale_violation(path, format!("languages.json entry at index {index} is missing `name`.")));
    };

    let Value::String(code) = code_value else {
        return Err(locale_violation(path, format!("languages.json entry at index {index} has non-string `code`.")));
    };
    let Value::String(_) = name_value else {
        return Err(locale_violation(path, format!("languages.json entry at index {index} has non-string `name`.")));
    };

    Ok(code.clone())
}



pub fn compare_languages_catalog(
    locale_dir: &Path,
    locale_files: &[PathBuf],
    language_codes: &BTreeSet<String>,
) -> Vec<Violation> {
    let languages_path = locale_dir.join("languages.json");
    let locale_codes: BTreeSet<String> = locale_files
        .iter()
        .filter_map(|path| locale_code_from_path(path))
        .collect();
    let mut violations = Vec::new();

    for code in locale_codes
        .iter()
        .filter(|code| !language_codes.contains(code.as_str()))
    {
        violations.push(locale_violation(
            &languages_path,
            format!("Locale file `{code}.json` exists but is missing from languages.json."),
        ));
    }

    for code in language_codes
        .iter()
        .filter(|code| !locale_codes.contains(code.as_str()))
    {
        violations.push(locale_violation(
            &languages_path,
            format!("Missing locale file `{code}.json` declared in languages.json."),
        ));
    }

    violations
}
