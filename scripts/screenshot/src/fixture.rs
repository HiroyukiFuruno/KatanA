use crate::request::{Fixture, FixtureSettings};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct FixtureEnv {
    pub config_dir: PathBuf,
    /// None means "launch with no workspace" (shows onboarding/welcome state).
    pub workspace_dir: Option<PathBuf>,
}

pub fn setup(fixture: &Fixture, tmp_root: &Path) -> Result<FixtureEnv> {
    let home_dir = tmp_root.join("home");
    std::fs::create_dir_all(&home_dir)?;

    let workspace_dir = if fixture.workspace_files.is_empty() && fixture.workspace_dir.is_none() {
        None
    } else {
        let dir = tmp_root.join("workspace");
        std::fs::create_dir_all(&dir)?;
        let resolved = match &fixture.workspace_dir {
            Some(p) => {
                let r = PathBuf::from(p)
                    .canonicalize()
                    .with_context(|| format!("fixture.workspace_dir not found: {p}"))?;
                anyhow::ensure!(r.is_dir(), "fixture.workspace_dir is not a directory: {p}");
                r
            }
            None => {
                for wf in &fixture.workspace_files {
                    let dest = dir.join(&wf.name);
                    std::fs::write(&dest, &wf.content)
                        .with_context(|| format!("writing fixture file {}", wf.name))?;
                }
                dir
            }
        };
        Some(resolved)
    };

    let cfg_dir = config_dir(&home_dir);
    std::fs::create_dir_all(&cfg_dir)?;
    let settings_json = build_settings_json(&fixture.settings, workspace_dir.as_deref());
    std::fs::write(cfg_dir.join("settings.json"), settings_json)?;

    Ok(FixtureEnv { config_dir: cfg_dir, workspace_dir })
}

fn config_dir(home: &Path) -> PathBuf {
    if cfg!(target_os = "macos") {
        home.join("Library").join("Application Support").join("KatanA")
    } else if cfg!(target_os = "windows") {
        home.join("AppData").join("Roaming").join("KatanA")
    } else {
        home.join(".config").join("KatanA")
    }
}

fn build_settings_json(settings: &FixtureSettings, workspace_dir: Option<&Path>) -> String {
    let theme_str = settings.theme.as_deref().unwrap_or("dark");
    let locale = settings.locale.as_deref().unwrap_or("en");
    let preset = if theme_str == "light" { "KatanaLight" } else { "KatanaDark" };

    let workspace_block = match workspace_dir {
        Some(dir) => format!(
            r#",
  "workspace": {{
    "last_workspace": "{}"
  }}"#,
            dir.display()
        ),
        None => String::new(),
    };

    format!(
        r#"{{
  "version": "0.22.0",
  "terms_accepted_version": "1.0",
  "language": "{locale}",
  "theme": {{
    "theme": "{theme_str}",
    "preset": "{preset}"
  }}{workspace_block}
}}"#
    )
}
