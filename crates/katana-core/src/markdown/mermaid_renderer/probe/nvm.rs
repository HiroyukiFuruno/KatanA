use std::path::PathBuf;

pub(super) fn probe_nvm_mmdc(nvm_dir: &str, bin_name: &str) -> Option<PathBuf> {
    let alias_path = PathBuf::from(format!("{nvm_dir}/alias/default"));
    let alias = std::fs::read_to_string(alias_path).ok()?;
    let alias = alias.trim();
    if alias.is_empty() {
        return None;
    }
    let versions_dir = PathBuf::from(format!("{nvm_dir}/versions/node"));
    let exact = versions_dir.join(alias).join("bin").join(bin_name);
    if exact.is_file() {
        return Some(exact);
    }
    find_mmdc_by_prefix(&versions_dir, alias, bin_name)
}

fn find_mmdc_by_prefix(dir: &std::path::Path, alias: &str, bin_name: &str) -> Option<PathBuf> {
    let prefix = if alias.starts_with('v') {
        alias.to_string()
    } else {
        format!("v{alias}")
    };
    let entries = std::fs::read_dir(dir).ok()?;
    let mut best = None;
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().starts_with(&prefix) {
            let candidate = entry.path().join("bin").join(bin_name);
            if candidate.is_file() {
                best = Some(candidate);
            }
        }
    }
    best
}
