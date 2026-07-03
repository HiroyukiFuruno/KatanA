use crate::request::{Fixture, FixtureSettings, WorkspaceFile};
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
                    write_workspace_file(wf, &dir)?;
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

    Ok(FixtureEnv {
        config_dir: cfg_dir,
        workspace_dir,
    })
}

fn write_workspace_file(file: &WorkspaceFile, workspace_dir: &Path) -> Result<()> {
    let dest = workspace_dir.join(file.name());
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating fixture dir for {}", file.name()))?;
    }

    match file {
        WorkspaceFile::Text { name, content } => std::fs::write(&dest, content)
            .with_context(|| format!("writing fixture file {name}"))?,
        WorkspaceFile::Copy { name, source } => {
            let source = PathBuf::from(source)
                .canonicalize()
                .with_context(|| format!("copy source not found for fixture file {name}"))?;
            std::fs::copy(source, &dest).with_context(|| format!("copying fixture file {name}"))?;
        }
    }

    Ok(())
}

fn config_dir(home: &Path) -> PathBuf {
    if cfg!(target_os = "macos") {
        home.join("Library")
            .join("Application Support")
            .join("KatanA")
    } else if cfg!(target_os = "windows") {
        home.join("AppData").join("Roaming").join("KatanA")
    } else {
        home.join(".config").join("KatanA")
    }
}

fn build_settings_json(settings: &FixtureSettings, workspace_dir: Option<&Path>) -> String {
    let theme_str = settings.theme.as_deref().unwrap_or("dark");
    let locale = settings.locale.as_deref().unwrap_or("en");
    let preset = settings.preset.as_deref().unwrap_or_else(|| {
        if theme_str == "light" {
            "KatanaLight"
        } else {
            "KatanaDark"
        }
    });
    let explorer_visible = settings.explorer_visible.unwrap_or(false);
    let linter_enabled = settings.linter_enabled.unwrap_or(true);
    let show_diagram_controls = settings.slideshow_show_diagram_controls.unwrap_or(true);

    let no_extension = settings.no_extension.unwrap_or(false);

    let linter_block = if settings.linter_enabled.is_some() {
        format!(
            r#",
  "linter": {{
    "enabled": {}
  }}"#,
            linter_enabled
        )
    } else {
        String::new()
    };

    let workspace_block = match (workspace_dir, no_extension) {
        (Some(dir), true) => format!(
            r#",
  "workspace": {{
    "last_workspace": "{}",
    "visible_extensions": ["md", "markdown", "txt", ""]
  }}"#,
            dir.display()
        ),
        (Some(dir), false) => format!(
            r#",
  "workspace": {{
    "last_workspace": "{}"
  }}"#,
            dir.display()
        ),
        (None, true) => r#",
  "workspace": {
    "visible_extensions": ["md", "markdown", "txt", ""]
  }"#
        .to_string(),
        (None, false) => String::new(),
    };

    format!(
        r#"{{
  "version": "0.22.0",
  "terms_accepted_version": "1.0",
  "language": "{locale}",
  "theme": {{
    "theme": "{theme_str}",
    "preset": "{preset}"
  }},
  "layout": {{
    "explorer_default_visible": {explorer_visible}
  }},
  "behavior": {{
    "slideshow_show_diagram_controls": {show_diagram_controls}
  }}{workspace_block}{linter_block}
}}"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screenshot_settings_can_disable_diagram_controls() {
        let settings = FixtureSettings {
            slideshow_show_diagram_controls: Some(false),
            ..FixtureSettings::default()
        };

        let json = build_settings_json(&settings, None);

        assert!(json.contains(r#""slideshow_show_diagram_controls": false"#));
    }
}
