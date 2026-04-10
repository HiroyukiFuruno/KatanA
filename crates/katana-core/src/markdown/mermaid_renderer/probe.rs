use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

pub(crate) struct OsProber;

impl OsProber {
    pub fn mmdc_filename() -> &'static str {
        #[cfg(windows)]
        return "mmdc.cmd";
        #[cfg(not(windows))]
        return "mmdc";
    }

    pub fn probe_well_known_paths() -> Option<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_default();

        if home.is_empty() {
            return None;
        }

        /* WHY: --- Homebrew --- */
        #[cfg(not(windows))]
        #[allow(clippy::useless_vec)]
        for brew_prefix in vec!["/opt/homebrew/bin/mmdc", "/usr/local/bin/mmdc"] {
            let p = PathBuf::from(brew_prefix);
            if p.is_file() {
                return Some(p);
            }
        }

        let bin_name = Self::mmdc_filename();

        /* WHY: --- Windows AppData paths --- */
        #[cfg(windows)]
        {
            let appdata =
                std::env::var("APPDATA").unwrap_or_else(|_| format!("{home}\\AppData\\Roaming"));
            /* WHY: npm global */
            let npm_global = PathBuf::from(&appdata).join("npm").join(bin_name);
            if npm_global.is_file() {
                return Some(npm_global);
            }

            let local_appdata =
                std::env::var("LOCALAPPDATA").unwrap_or_else(|_| format!("{home}\\AppData\\Local"));
            /* WHY: volta */
            let volta_bin = PathBuf::from(&local_appdata)
                .join("Volta")
                .join("bin")
                .join("mmdc.exe");
            if volta_bin.is_file() {
                return Some(volta_bin);
            }
            let volta_cmd = PathBuf::from(&local_appdata)
                .join("Volta")
                .join("bin")
                .join("mmdc.cmd");
            if volta_cmd.is_file() {
                return Some(volta_cmd);
            }
        }

        /* WHY: --- nvm --- */
        let nvm_dir = std::env::var("NVM_DIR").unwrap_or_else(|_| format!("{home}/.nvm"));
        if let Some(path) = probe_nvm_mmdc(&nvm_dir, bin_name) {
            return Some(path);
        }

        /* WHY: --- volta (Unix) --- */
        #[cfg(not(windows))]
        {
            let volta_bin = PathBuf::from(format!("{home}/.volta/bin/mmdc"));
            if volta_bin.is_file() {
                return Some(volta_bin);
            }
        }

        /* WHY: --- fnm --- */
        #[cfg(not(windows))]
        let fnm_bin = PathBuf::from(format!("{home}/.local/share/fnm/aliases/default/bin/mmdc"));
        #[cfg(windows)]
        let fnm_bin = PathBuf::from(std::env::var("APPDATA").unwrap_or_default())
            .join("fnm")
            .join("aliases")
            .join("default")
            .join(bin_name);

        if fnm_bin.is_file() {
            return Some(fnm_bin);
        }

        None
    }

    pub fn which_from_current_path() -> Option<PathBuf> {
        #[cfg(windows)]
        let cmd_name = "where";
        #[cfg(not(windows))]
        let cmd_name = "which";

        #[cfg(windows)]
        let target_name = "mmdc";
        #[cfg(not(windows))]
        let target_name = "mmdc";

        let output = Command::new(cmd_name)
            .arg(target_name)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;

        if output.status.success() {
            /* WHY: Find the first line (where outputs multiple matching paths if available) */
            let path = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();

            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
        None
    }

    pub fn resolve_via_login_shell() -> Option<PathBuf> {
        #[cfg(windows)]
        return None;

        #[cfg(not(windows))]
        {
            let output = Command::new("/bin/zsh")
                .args(vec!["-l", "-c", "which mmdc"])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
                .ok()?;
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(PathBuf::from(path));
                }
            }
            None
        }
    }
}

fn probe_nvm_mmdc(nvm_dir: &str, bin_name: &str) -> Option<PathBuf> {
    let alias_file = PathBuf::from(format!("{nvm_dir}/alias/default"));
    let alias = std::fs::read_to_string(&alias_file).ok()?;
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

fn find_mmdc_by_prefix(
    versions_dir: &std::path::Path,
    alias: &str,
    bin_name: &str,
) -> Option<PathBuf> {
    let prefix = if alias.starts_with('v') {
        alias.to_string()
    } else {
        format!("v{alias}")
    };
    let entries = std::fs::read_dir(versions_dir).ok()?;
    let mut best: Option<PathBuf> = None;
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
