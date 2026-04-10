mod nvm;

use nvm::probe_nvm_mmdc;
use std::{path::PathBuf, process::Stdio};

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
            .ok()?;

        /* WHY: --- Homebrew --- */
        #[cfg(not(windows))]
        for brew_path in ["/opt/homebrew/bin/mmdc", "/usr/local/bin/mmdc"] {
            let p = PathBuf::from(brew_path);
            if p.is_file() {
                return Some(p);
            }
        }

        #[cfg(windows)]
        if let Some(p) = Self::probe_windows_paths(&home) {
            return Some(p);
        }

        /* WHY: --- nvm & volta --- */
        let bin_name = Self::mmdc_filename();
        let nvm_dir = std::env::var("NVM_DIR").unwrap_or_else(|_| format!("{home}/.nvm"));
        if let Some(path) = probe_nvm_mmdc(&nvm_dir, bin_name) {
            return Some(path);
        }

        #[cfg(not(windows))]
        if let Some(p) = Self::probe_unix_volta(&home) {
            return Some(p);
        }

        Self::probe_fnm_path(&home, bin_name)
    }

    #[cfg(windows)]
    fn probe_windows_paths(home: &str) -> Option<PathBuf> {
        let appdata =
            std::env::var("APPDATA").unwrap_or_else(|_| format!("{home}\\AppData\\Roaming"));
        for ext in ["cmd", "ps1", "exe"] {
            let p = PathBuf::from(&appdata)
                .join("npm")
                .join(format!("mmdc.{ext}"));
            if p.is_file() {
                return Some(p);
            }
        }
        let local =
            std::env::var("LOCALAPPDATA").unwrap_or_else(|_| format!("{home}\\AppData\\Local"));
        for ext in ["exe", "cmd", "ps1"] {
            let p = PathBuf::from(&local)
                .join("Volta")
                .join("bin")
                .join(format!("mmdc.{ext}"));
            if p.is_file() {
                return Some(p);
            }
        }
        None
    }

    #[cfg(not(windows))]
    fn probe_unix_volta(home: &str) -> Option<PathBuf> {
        let p = PathBuf::from(format!("{home}/.volta/bin/mmdc"));
        if p.is_file() { Some(p) } else { None }
    }

    fn probe_fnm_path(_home: &str, bin_name: &str) -> Option<PathBuf> {
        #[cfg(not(windows))]
        let p = PathBuf::from(format!(
            "{_home}/.local/share/fnm/aliases/default/bin/{bin_name}"
        ));
        #[cfg(windows)]
        let p = PathBuf::from(std::env::var("APPDATA").unwrap_or_default())
            .join("fnm")
            .join("aliases")
            .join("default")
            .join(bin_name);
        if p.is_file() { Some(p) } else { None }
    }

    pub fn which_from_current_path() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            for ext in ["cmd", "ps1", "exe", ""] {
                let target = if ext.is_empty() {
                    "mmdc".to_string()
                } else {
                    format!("mmdc.{ext}")
                };
                if let Some(p) = Self::run_where(&target) {
                    return Some(p);
                }
            }
            None
        }
        #[cfg(not(windows))]
        Self::run_which("mmdc")
    }

    #[cfg(windows)]
    fn run_where(target: &str) -> Option<PathBuf> {
        let out = crate::system::ProcessService::create_command("where")
            .arg(target)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout)
                .lines()
                .next()?
                .trim()
                .to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
        None
    }

    #[cfg(not(windows))]
    fn run_which(target: &str) -> Option<PathBuf> {
        let out = crate::system::ProcessService::create_command("which")
            .arg(target)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
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
            /* WHY: Try common shells to resolve mmdc path from user environment. */
            let shells = [
                std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()),
                "/bin/zsh".to_string(),
                "/bin/bash".to_string(),
            ];

            for shell in shells {
                if let Some(p) = Self::run_which_in_shell(&shell) {
                    return Some(p);
                }
            }
            None
        }
    }

    #[cfg(not(windows))]
    fn run_which_in_shell(shell: &str) -> Option<PathBuf> {
        let out = crate::system::ProcessService::create_command(shell)
            .args(["-l", "-c", "which mmdc"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !path.is_empty() && path.starts_with('/') {
                return Some(PathBuf::from(path));
            }
        }
        None
    }
}
