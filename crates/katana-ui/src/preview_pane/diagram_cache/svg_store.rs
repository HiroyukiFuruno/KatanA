use super::content::{DiagramActiveToken, DiagramCacheIdentity};
use super::metrics::DiagramCacheMetrics;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) struct DiagramSvgCacheStore;

impl DiagramSvgCacheStore {
    pub(super) fn write_svg(
        root: &Path,
        identity: &DiagramCacheIdentity,
        svg: &str,
    ) -> std::io::Result<()> {
        if !is_svg_payload(svg) {
            return Ok(());
        }
        let target_path = identity.cache_path(root);
        let Some(parent) = target_path.parent() else {
            return Ok(());
        };
        std::fs::create_dir_all(parent)?;
        let temp_path = temp_file_path(&target_path);
        std::fs::write(&temp_path, svg)?;
        std::fs::rename(temp_path, target_path)?;
        Ok(())
    }

    pub(super) fn prune_document(
        root: &Path,
        document_dir_name: &str,
        identities: &[DiagramCacheIdentity],
    ) -> std::io::Result<()> {
        let document_dir = root.join(document_dir_name);
        let active = identities
            .iter()
            .map(DiagramCacheIdentity::active_token)
            .collect::<HashSet<_>>();
        prune_document_dir(&document_dir, &active)
    }
}

fn prune_document_dir(
    document_dir: &Path,
    active: &HashSet<DiagramActiveToken>,
) -> std::io::Result<()> {
    let Ok(kind_dirs) = std::fs::read_dir(document_dir) else {
        return Ok(());
    };
    for kind_dir in kind_dirs.flatten() {
        if !kind_dir.path().is_dir() {
            continue;
        }
        prune_kind_dir(&kind_dir.path(), active)?;
        remove_dir_if_empty(&kind_dir.path());
    }
    remove_dir_if_empty(document_dir);
    Ok(())
}

fn prune_kind_dir(kind_dir: &Path, active: &HashSet<DiagramActiveToken>) -> std::io::Result<()> {
    let kind_dir_name = kind_dir_name(kind_dir);
    let Ok(entries) = std::fs::read_dir(kind_dir) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        prune_cache_file(entry.path(), &kind_dir_name, active);
    }
    Ok(())
}

fn prune_cache_file(path: PathBuf, kind_dir_name: &str, active: &HashSet<DiagramActiveToken>) {
    if path.is_dir() {
        return;
    }
    let Some(checksum) = checksum_from_svg_file(&path) else {
        let _ = std::fs::remove_file(path);
        return;
    };
    let token = DiagramActiveToken {
        kind_dir_name: kind_dir_name.to_string(),
        content_checksum: checksum.clone(),
    };
    if !active.contains(&token) {
        let _ = std::fs::remove_file(path);
        DiagramCacheMetrics::emit_pruned(kind_dir_name, &checksum);
    }
}

fn kind_dir_name(kind_dir: &Path) -> String {
    kind_dir
        .file_name()
        .and_then(|it| it.to_str())
        .unwrap_or_default()
        .to_string()
}

fn checksum_from_svg_file(path: &Path) -> Option<String> {
    if path.extension().and_then(|it| it.to_str()) != Some("svg") {
        return None;
    }
    let stem = path.file_stem()?.to_str()?;
    stem.split('_').next().map(str::to_string)
}

fn remove_dir_if_empty(path: &Path) {
    if std::fs::read_dir(path)
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(false)
    {
        let _ = std::fs::remove_dir(path);
    }
}

fn is_svg_payload(svg: &str) -> bool {
    svg.contains("<svg") && svg.contains("</svg>")
}

fn temp_file_path(target_path: &Path) -> PathBuf {
    let file_name = target_path
        .file_name()
        .and_then(|it| it.to_str())
        .unwrap_or("diagram.svg");
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    target_path.with_file_name(format!("{file_name}.tmp.{}-{nanos}", std::process::id()))
}
