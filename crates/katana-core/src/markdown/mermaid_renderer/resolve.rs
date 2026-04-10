use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::RwLock,
};

use super::probe::OsProber;

static MMDC_RESOLVED_PATH: RwLock<Option<PathBuf>> = RwLock::new(None);

pub struct MermaidBinaryOps;

impl MermaidBinaryOps {
    pub fn resolve_mmdc_binary() -> PathBuf {
        /* WHY: Always check env var first (not cached — allows runtime override) */
        #[allow(clippy::single_match)]
        match std::env::var("MERMAID_MMDC") {
            Ok(p) => return PathBuf::from(p),
            Err(_) => {}
        }

        if let Ok(guard) = MMDC_RESOLVED_PATH.read()
            && let Some(path) = &*guard
        {
            return path.clone();
        }

        let resolved = OsProber::probe_well_known_paths()
            .or_else(OsProber::which_from_current_path)
            .or_else(OsProber::resolve_via_login_shell);

        if let Some(path) = &resolved
            && let Ok(mut guard) = MMDC_RESOLVED_PATH.write()
        {
            *guard = Some(path.clone());
        }

        resolved.unwrap_or_else(|| {
            #[cfg(windows)]
            {
                PathBuf::from("mmdc.cmd")
            }
            #[cfg(not(windows))]
            {
                PathBuf::from("mmdc")
            }
        })
    }

    pub fn build_mmdc_command() -> Command {
        let mmdc = Self::resolve_mmdc_binary();
        Self::build_mmdc_command_for_binary(&mmdc)
    }

    pub(crate) fn build_mmdc_command_for_binary(mmdc: &Path) -> Command {
        let mut cmd = crate::system::ProcessService::create_command(&mmdc.to_string_lossy(), true);

        /* WHY: Enrich PATH so that `#!/usr/bin/env node` can find `node`. */
        if let Some(bin_dir) = mmdc.parent() {
            let current_path = std::env::var("PATH").unwrap_or_default();
            let bin_dir_str = bin_dir.to_string_lossy();

            /* WHY: Handle both : and ; path separators */
            #[cfg(windows)]
            let path_sep = ";";
            #[cfg(not(windows))]
            let path_sep = ":";

            if !current_path
                .split(path_sep)
                .any(|p| p == bin_dir_str.as_ref())
            {
                cmd.env("PATH", format!("{bin_dir_str}{path_sep}{current_path}"));
            }
        }
        cmd
    }
}
