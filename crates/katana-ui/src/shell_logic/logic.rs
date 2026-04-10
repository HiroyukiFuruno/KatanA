use super::*;
use chrono::{DateTime, Local};
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

    pub fn collect_matches(
        tree: &[katana_core::workspace::TreeEntry],
        query: &str,
        include: &[regex::Regex],
        exclude: &[regex::Regex],
        root: &Path,
        results: &mut Vec<std::path::PathBuf>,
    ) {
        let query_lower = query.to_lowercase();
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
                    if !query_lower.is_empty()
                        && !name.to_lowercase().contains(&query_lower)
                        && !rel_str.to_lowercase().contains(&query_lower)
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
                    Self::collect_matches(children, query, include, exclude, root, results);
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

        let size_str = Self::format_file_size(metadata.len());
        tooltip.push_str(&format!("\nSize: {}", size_str));

        if let Ok(modified) = metadata.modified() {
            let mod_time = Self::format_modified_time(modified);
            tooltip.push_str(&format!("\nModified: {}", mod_time));
        }
        tooltip
    }

    const KB_UNIT: u64 = 1024;
    const MB_UNIT: u64 = 1024 * 1024;
    const GB_UNIT: u64 = 1024 * 1024 * 1024;

    pub fn format_file_size(bytes: u64) -> String {
        if bytes < Self::KB_UNIT {
            format!("{} B", bytes)
        } else if bytes < Self::MB_UNIT {
            format!("{:.1} KB", bytes as f64 / (Self::KB_UNIT as f64))
        } else if bytes < Self::GB_UNIT {
            format!("{:.1} MB", bytes as f64 / (Self::MB_UNIT as f64))
        } else {
            format!("{:.1} GB", bytes as f64 / (Self::GB_UNIT as f64))
        }
    }

    pub fn format_modified_time(time: std::time::SystemTime) -> String {
        let dt: DateTime<Local> = time.into();
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    const SPLASH_FADE_START: f32 = 0.8;
    const SPLASH_FADE_DURATION: f32 = 0.2;
    const SPLASH_TOTAL_DURATION_SECS: f32 = 1.5;

    pub fn calculate_splash_opacity(progress: f32) -> f32 {
        if progress < Self::SPLASH_FADE_START {
            1.0
        } else {
            (1.0 - progress) / Self::SPLASH_FADE_DURATION
        }
    }

    pub fn calculate_splash_progress(elapsed: f32) -> f32 {
        let p = elapsed / Self::SPLASH_TOTAL_DURATION_SECS;
        p.clamp(0.0, 1.0)
    }

    pub fn prev_tab_index(idx: usize, count: usize) -> usize {
        if count == 0 {
            0
        } else {
            (idx + count - 1) % count
        }
    }

    pub fn next_tab_index(idx: usize, count: usize) -> usize {
        if count == 0 { 0 } else { (idx + 1) % count }
    }
}
