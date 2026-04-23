use std::path::Path;

pub struct LocaleAuditOps;

impl LocaleAuditOps {
    /// Rule: i18n-unused-keys
    ///
    /// Scans en.json and checks whether each leaf key is referenced in Rust source files
    /// under `crates/`. Keys with no usage should be removed.
    /// A key `section.key` is considered used if any `.rs` file contains the string
    /// `section.key` or just the leaf name (e.g. `key`).
    pub fn lint_unused_keys(workspace_root: &Path, locale_dir: &Path) -> Vec<crate::Violation> {
        use crate::utils::{LinterFileOps, LinterJsonOps};

        let en_path = locale_dir.join("en.json");
        let Ok(en_json) = LinterJsonOps::parse_json_file(&en_path) else {
            return vec![];
        };

        let mut leaf_keys: std::collections::BTreeMap<String, String> = Default::default();
        LinterJsonOps::collect_json_values(&en_json, None, &mut leaf_keys);

        /* WHY: Collect all Rust source text once to avoid per-key file reads. */
        let rust_source_dir = workspace_root.join("crates");
        let rust_files = LinterFileOps::collect_files_by_extension(&rust_source_dir, "rs");
        let source_corpus: String = rust_files
            .iter()
            .filter_map(|p| std::fs::read_to_string(p).ok())
            .collect::<Vec<_>>()
            .join("\n");

        let mut violations = Vec::new();
        for key_path in leaf_keys.keys() {
            /* WHY: Check the full dotted path, the last-two-segment path (e.g. `section.key`),
             * and the bare leaf name, since Rust accesses take forms like `msgs.section.key`. */
            let parts: Vec<&str> = key_path.split('.').collect();
            let leaf = *parts.last().unwrap_or(&"");
            let section_leaf = if parts.len() >= 2 {
                format!("{}.{}", parts[parts.len() - 2], leaf)
            } else {
                key_path.clone()
            };

            let used = source_corpus.contains(key_path.as_str())
                || source_corpus.contains(section_leaf.as_str())
                || source_corpus.contains(leaf)
                || key_path.starts_with("linter.rule_descriptions.");

            if !used {
                violations.push(crate::utils::ViolationReporterOps::locale_violation(
                    &en_path,
                    format!(
                        "Locale key `{key_path}` is not referenced in any Rust source file. \
                         Consider removing it to keep translations lean."
                    ),
                ));
            }
        }

        violations
    }

    /// Rule: i18n-duplicate-values
    ///
    /// Within a single locale file, detects leaf string values that appear under two or more
    /// keys belonging to *different* top-level sections. This pattern suggests the string
    /// should live in `common` rather than being duplicated across sections.
    pub fn lint_duplicate_values_within_file(locale_file: &Path) -> Vec<crate::Violation> {
        use crate::utils::LinterJsonOps;

        /* WHY: Grandfathered duplicates that existed before this rule was introduced.
         * Each entry is the English value that is intentionally shared across sections.
         * To fix: consolidate into `common` and remove from this list. */
        #[rustfmt::skip]
        const GRANDFATHERED_DUPLICATES: &[&str] = &[
            "Settings", "Workspace", "Refresh Document", "Toggle Slideshow",
            "Use Workspace-Local Configuration", "Zoom Out", "Zoom In",
            "Check for Updates", "Official Website", "Behavior",
            "Modified", "Open Workspace", "Markdown Document",
            "Delete", "Open File", "Open Folder", "Pinned", "Name",
            "Refresh", "Reset", "Create File", "Create Folder",
            "Close", "Save", "Undo", "Redo", "Copy", "Paste", "Cut",
        ];

        /* WHY: Strings shorter than this threshold are typically single words that
         * appear naturally across sections (e.g. "Version", "Layout") and would
         * generate too much noise. 8 characters filters those out cleanly. */
        const MIN_DUPLICATE_VALUE_LEN: usize = 8;

        let Ok(json) = LinterJsonOps::parse_json_file(locale_file) else {
            return vec![];
        };

        let mut leaf_values: std::collections::BTreeMap<String, String> = Default::default();
        LinterJsonOps::collect_json_values(&json, None, &mut leaf_values);

        /* WHY: Build a reverse index: value → list of key paths that carry it. */
        let mut value_to_keys: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        /* WHY: Values already defined in `common` are the canonical source —
         * their presence elsewhere is intentional (referencing the shared key). */
        let common_values: std::collections::HashSet<String> = leaf_values
            .iter()
            .filter(|(k, _)| k.starts_with("common."))
            .map(|(_, v)| v.clone())
            .collect();

        for (key, val) in &leaf_values {
            /* WHY: Skip trivial or template-only strings to reduce noise.
             * Also skip rule_descriptions which intentionally share long-form prose.
             * Skip values registered in `common` — they are the canonical source.
             * Skip grandfathered duplicates (pre-existing, tracked for future cleanup).
             * Threshold of 8 chars avoids flagging short words naturally shared. */
            let is_trivial = val.len() < MIN_DUPLICATE_VALUE_LEN
                || val
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
                || val.starts_with('{')
                || key.contains("rule_descriptions")
                || common_values.contains(val.as_str())
                || GRANDFATHERED_DUPLICATES.contains(&val.as_str());
            if !is_trivial {
                value_to_keys
                    .entry(val.clone())
                    .or_default()
                    .push(key.clone());
            }
        }

        let mut violations = Vec::new();
        for (val, keys) in &value_to_keys {
            if keys.len() < 2 {
                continue;
            }
            /* WHY: Only report duplicates that span different top-level sections —
             * a cross-section duplicate is a consolidation candidate for `common`. */
            let sections: std::collections::HashSet<&str> = keys
                .iter()
                .map(|k| k.split('.').next().unwrap_or(""))
                .collect();
            if sections.len() < 2 {
                continue;
            }
            let key_list = keys.join("`, `");
            violations.push(crate::utils::ViolationReporterOps::locale_violation(
                locale_file,
                format!(
                    "Locale value \"{val}\" appears in multiple sections under keys: \
                     `{key_list}`. Consider consolidating into `common` to avoid \
                     maintaining the same translation in multiple places."
                ),
            ));
        }

        violations
    }
}
