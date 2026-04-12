use super::*;
use std::path::Path;

const FNV1A_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV1A_PRIME: u64 = 0x100000001b3;

impl ShellLogicOps {
    pub fn hash_str(s: &str) -> u64 {
        let mut h: u64 = FNV1A_OFFSET_BASIS;
        for b in s.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(FNV1A_PRIME);
        }
        h
    }

    pub fn relative_full_path(path: &Path, ws_root: Option<&Path>) -> String {
        let rel = match ws_root {
            Some(root) => path.strip_prefix(root).unwrap_or(path),
            None => path,
        };
        rel.display().to_string()
    }

    pub fn export_html_to_tmp(
        path: &std::path::Path,
        source: &str,
        preset: &katana_core::markdown::color_preset::DiagramColorPreset,
    ) -> Result<std::path::PathBuf, String> {
        let renderer = katana_core::markdown::KatanaRenderer;
        let html = katana_core::markdown::HtmlExporter::export(source, &renderer, preset, None)
            .map_err(|e| e.to_string())?;

        let hash = Self::hash_str(&path.to_string_lossy());
        let filename = format!("katana_export_{hash}.html");
        let output_path = std::env::temp_dir().join(filename);

        std::fs::write(&output_path, html.as_bytes()).map_err(|e| e.to_string())?;
        Ok(output_path)
    }

    pub fn download_with_curl(url: &str, dest: &std::path::Path) -> Result<(), String> {
        Self::_download_with_cmd("curl", url, dest)
    }

    pub(crate) fn _download_with_cmd(
        cmd: &str,
        url: &str,
        dest: &std::path::Path,
    ) -> Result<(), String> {
        if let Some(p) = dest.parent() {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        let status = katana_core::system::ProcessService::create_command(cmd)
            .args(vec!["-L", "-o", dest.to_str().unwrap_or(""), url])
            .status()
            .map_err(|e| {
                format!(
                    "{}: {e}",
                    crate::i18n::I18nOps::get().error.curl_launch_failed.clone()
                )
            })?;
        if status.success() {
            Ok(())
        } else {
            Err(crate::i18n::I18nOps::get().error.download_failed.clone())
        }
    }

    pub fn export_named_html_to_tmp(
        source: &str,
        filename: &str,
        preset: &katana_core::markdown::color_preset::DiagramColorPreset,
        base_dir: Option<&std::path::Path>,
    ) -> Result<std::path::PathBuf, String> {
        let renderer = katana_core::markdown::KatanaRenderer;
        let html = katana_core::markdown::HtmlExporter::export(source, &renderer, preset, base_dir)
            .map_err(|e| e.to_string())?;
        let output_path = std::env::temp_dir().join(filename);
        std::fs::write(&output_path, html.as_bytes()).map_err(|e| e.to_string())?;
        Ok(output_path)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn collect_matches(
        tree: &[katana_core::workspace::TreeEntry],
        query: &str,
        include: &[regex::Regex],
        exclude: &[regex::Regex],
        root: &Path,
        match_case: bool,
        match_word: bool,
        use_regex: bool,
        results: &mut Vec<std::path::PathBuf>,
    ) {
        let (pattern, case_insensitive) = if use_regex {
            let p = if match_word {
                format!(r"\b{}\b", query)
            } else {
                query.to_string()
            };
            (p, !match_case)
        } else {
            let p = regex::escape(query);
            let p = if match_word { format!(r"\b{}\b", p) } else { p };
            (p, !match_case)
        };

        /* WHY: Precompile query regex for efficiency so we don't compile inside the loop */
        let query_re = if query.is_empty() {
            None
        } else {
            regex::RegexBuilder::new(&pattern)
                .case_insensitive(case_insensitive)
                .build()
                .ok()
        };
        for entry in tree {
            match entry {
                katana_core::workspace::TreeEntry::File { path } => {
                    let rel = path.strip_prefix(root).unwrap_or(path);
                    let rel_str = rel.to_string_lossy().to_string();
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    let mut is_match = true;
                    if let Some(ref re) = query_re
                        && !re.is_match(&name)
                        && !re.is_match(&rel_str)
                    {
                        is_match = false;
                    }

                    if is_match && !include.is_empty() {
                        is_match = include.iter().any(|re| re.is_match(&rel_str));
                    }
                    if is_match && !exclude.is_empty() {
                        is_match = !exclude.iter().any(|re| re.is_match(&rel_str));
                    }

                    if is_match {
                        results.push(path.clone());
                    }
                }
                katana_core::workspace::TreeEntry::Directory { children, .. } => {
                    Self::collect_matches(
                        children, query, include, exclude, root, match_case, match_word, use_regex,
                        results,
                    );
                }
            }
        }
    }

    pub fn format_window_title(fname: &str, rel: &str, release_notes: &str) -> String {
        if fname == release_notes {
            return format!("{} - KatanA", release_notes);
        }
        format!("{} ({}) - KatanA", fname, rel)
    }

    pub fn format_tree_tooltip(name: &str, path: &Path) -> String {
        let mut tooltip = format!("{name}\n{}", path.display());
        let Ok(metadata) = path.metadata() else {
            tooltip.push_str("\nMetadata unavailable");
            return tooltip;
        };
        let Ok(modified) = metadata.modified() else {
            return tooltip;
        };
        tooltip.push_str(&format!("\nModified: {:?}", modified));
        tooltip
    }
}
