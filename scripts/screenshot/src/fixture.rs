use crate::request::{Fixture, FixtureSettings};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct FixtureEnv {
    pub home_dir: PathBuf,
}

pub fn setup(fixture: &Fixture, tmp_root: &Path) -> Result<FixtureEnv> {
    let home_dir = tmp_root.join("home");
    let workspace_dir = tmp_root.join("workspace");
    std::fs::create_dir_all(&home_dir)?;
    std::fs::create_dir_all(&workspace_dir)?;

    let workspace_dir = match &fixture.workspace_dir {
        Some(p) => {
            let resolved = PathBuf::from(p).canonicalize()
                .with_context(|| format!("fixture.workspace_dir not found: {p}"))?;
            anyhow::ensure!(resolved.is_dir(), "fixture.workspace_dir is not a directory: {p}");
            resolved
        }
        None => {
            for wf in &fixture.workspace_files {
                let dest = workspace_dir.join(&wf.name);
                std::fs::write(&dest, &wf.content)
                    .with_context(|| format!("writing fixture file {}", wf.name))?;
            }
            workspace_dir
        }
    };

    let settings_dir = config_dir(&home_dir);
    std::fs::create_dir_all(&settings_dir)?;
    let settings_json = build_settings_json(&fixture.settings, &workspace_dir);
    std::fs::write(settings_dir.join("settings.json"), settings_json)?;

    Ok(FixtureEnv { home_dir })
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

fn build_settings_json(settings: &FixtureSettings, workspace_dir: &Path) -> String {
    let theme_str = settings.theme.as_deref().unwrap_or("dark");
    let locale = settings.locale.as_deref().unwrap_or("en");
    let preset = if theme_str == "light" { "KatanaLight" } else { "KatanaDark" };
    let workspace_str = workspace_dir.display();

    format!(
        r#"{{
  "version": "0.22.0",
  "terms_accepted_version": "1.0",
  "language": "{locale}",
  "theme": {{
    "theme": "{theme_str}",
    "preset": "{preset}"
  }},
  "workspace": {{
    "last_workspace": "{workspace_str}"
  }}
}}"#
    )
}
