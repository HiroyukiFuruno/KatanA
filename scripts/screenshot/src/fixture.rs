use crate::request::{Fixture, FixtureSettings, WorkspaceFile};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct FixtureEnv {
    pub config_dir: PathBuf,
    /// None means "launch with no workspace" (shows onboarding/welcome state).
    pub workspace_dir: Option<PathBuf>,
}

pub fn setup(fixture: &Fixture, tmp_root: &Path) -> Result<FixtureEnv> {
    if let Some(size) = fixture.settings.font_size {
        anyhow::ensure!(
            size.is_finite() && (8.0..=32.0).contains(&size),
            "fixture font_size must be between 8 and 32"
        );
    }
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
    let mut settings_value: serde_json::Value = serde_json::from_str(&settings_json)?;
    if let Some(size) = fixture.settings.font_size {
        settings_value["font"] = serde_json::json!({ "size": size });
    }
    let settings_json = serde_json::to_vec_pretty(&settings_value)?;
    std::fs::write(cfg_dir.join("settings.json"), settings_json)?;

    Ok(FixtureEnv {
        config_dir: cfg_dir,
        workspace_dir,
    })
}

fn write_workspace_file(file: &WorkspaceFile, workspace_dir: &Path) -> Result<()> {
    let dest = fixture_destination(file.name(), workspace_dir)?;
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

fn fixture_destination(name: &str, workspace_dir: &Path) -> Result<PathBuf> {
    let path = Path::new(name);
    anyhow::ensure!(
        !name.is_empty()
            && !name.contains(['\\', ':'])
            && path
                .components()
                .all(|part| matches!(part, std::path::Component::Normal(_))),
        "fixture filename must be a relative workspace path: {name}"
    );
    let mut dest = workspace_dir.canonicalize()?;
    for part in path.components() {
        dest.push(part.as_os_str());
        match std::fs::symlink_metadata(&dest) {
            Ok(metadata) => anyhow::ensure!(
                !metadata.file_type().is_symlink(),
                "fixture destination must not follow a symlink: {name}"
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error).context("checking fixture destination"),
        }
    }
    Ok(dest)
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
    let app_version = katana_ui::about_info::APP_VERSION;
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
    let auto_refresh = settings.auto_refresh.unwrap_or(true);
    let auto_refresh_interval_secs = settings.auto_refresh_interval_secs.unwrap_or(2.0);
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

    let workspace_dir = workspace_dir.map(|dir| serde_json::json!(dir.to_string_lossy()));
    let workspace_block = match (workspace_dir, no_extension) {
        (Some(dir), true) => format!(
            r#",
  "workspace": {{
    "last_workspace": {},
    "visible_extensions": ["md", "markdown", "txt", ""]
  }}"#,
            dir
        ),
        (Some(dir), false) => format!(
            r#",
  "workspace": {{
    "last_workspace": {}
  }}"#,
            dir
        ),
        (None, true) => r#",
  "workspace": {
    "visible_extensions": ["md", "markdown", "txt", ""]
  }"#
        .to_string(),
        (None, false) => String::new(),
    };

    let locale = serde_json::json!(locale);
    let theme_str = serde_json::json!(theme_str);
    let preset = serde_json::json!(preset);

    format!(
        r#"{{
  "version": "{app_version}",
  "terms_accepted_version": "1.0",
  "language": {locale},
  "theme": {{
    "theme": {theme_str},
    "preset": {preset}
  }},
  "layout": {{
    "explorer_default_visible": {explorer_visible}
  }},
  "behavior": {{
    "auto_refresh": {auto_refresh},
    "auto_refresh_interval_secs": {auto_refresh_interval_secs},
    "slideshow_show_diagram_controls": {show_diagram_controls}
  }},
  "updates": {{
    "interval": "Never",
    "previous_app_version": "{app_version}"
  }}{workspace_block}{linter_block}
}}"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use katana_platform::{JsonFileRepository, SettingsRepository};

    #[test]
    fn fixture_files_reject_escaping_names_before_writing() -> Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = root.path().join("workspace");
        std::fs::create_dir(&workspace)?;
        let absolute = root.path().join("escaped.txt");
        for name in [
            "",
            "../escaped.txt",
            "nested/../../escaped.txt",
            "C:\\escaped.txt",
            "nested\\..\\escaped.txt",
            "name:stream",
            absolute.to_str().context("UTF-8 temporary path")?,
        ] {
            let file = WorkspaceFile::Text {
                name: name.to_owned(),
                content: "must not escape".to_owned(),
            };
            assert!(
                write_workspace_file(&file, &workspace).is_err(),
                "accepted {name:?}"
            );
            assert!(!absolute.exists());
        }
        assert!(std::fs::read_dir(&workspace)?.next().is_none());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn fixture_files_reject_symlink_destinations() -> Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = root.path().join("workspace");
        let outside = root.path().join("outside");
        std::fs::create_dir(&workspace)?;
        std::fs::create_dir(&outside)?;
        let protected = outside.join("protected.txt");
        std::fs::write(&protected, "unchanged")?;
        std::os::unix::fs::symlink(&outside, workspace.join("linked"))?;
        std::os::unix::fs::symlink(&protected, workspace.join("leaf.txt"))?;
        for name in ["linked/protected.txt", "leaf.txt"] {
            let file = WorkspaceFile::Text {
                name: name.to_owned(),
                content: "overwrite".to_owned(),
            };
            assert!(write_workspace_file(&file, &workspace).is_err());
            assert_eq!(std::fs::read_to_string(&protected)?, "unchanged");
        }
        Ok(())
    }

    #[test]
    fn fixture_files_preserve_nested_text_and_copy() -> Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = root.path().join("workspace");
        std::fs::create_dir(&workspace)?;
        let source = root.path().join("source.txt");
        std::fs::write(&source, "copied")?;
        let files = [
            WorkspaceFile::Text {
                name: "nested/text.txt".to_owned(),
                content: "text".to_owned(),
            },
            WorkspaceFile::Copy {
                name: "nested/copy.txt".to_owned(),
                source: source.to_str().context("UTF-8 temporary path")?.to_owned(),
            },
        ];
        for file in &files {
            write_workspace_file(file, &workspace)?;
        }
        assert_eq!(
            std::fs::read_to_string(workspace.join("nested/text.txt"))?,
            "text"
        );
        assert_eq!(
            std::fs::read_to_string(workspace.join("nested/copy.txt"))?,
            "copied"
        );
        Ok(())
    }

    #[test]
    fn screenshot_settings_preserve_escaped_string_values() -> Result<()> {
        for input in ["quoted\"value", "back\\slash", "line\nwith\ttab", "日本語"] {
            let settings = FixtureSettings {
                locale: Some(input.to_owned()),
                theme: Some(input.to_owned()),
                preset: Some(input.to_owned()),
                ..FixtureSettings::default()
            };
            let value: serde_json::Value =
                serde_json::from_str(&build_settings_json(&settings, None))?;
            assert_eq!(value["language"], input);
            assert_eq!(value["theme"]["theme"], input);
            assert_eq!(value["theme"]["preset"], input);
            assert_eq!(value["updates"]["interval"], "Never");
        }
        Ok(())
    }

    #[test]
    fn screenshot_settings_preserve_escaped_workspace_paths() -> Result<()> {
        for workspace in [
            r"D:\a\KatanA\KatanA\target\workspace",
            r"\\?\D:\a\KatanA\KatanA\target\workspace",
            r"\\server\share\workspace",
            "/tmp/日本語 workspace/\"quoted\"",
            "/tmp/newline\nworkspace\ttab",
        ] {
            for no_extension in [false, true] {
                let settings = FixtureSettings {
                    no_extension: Some(no_extension),
                    ..FixtureSettings::default()
                };
                let json = build_settings_json(&settings, Some(Path::new(workspace)));
                let value: serde_json::Value = serde_json::from_str(&json)?;
                assert_eq!(value["workspace"]["last_workspace"], workspace);
                assert_eq!(
                    value["workspace"].get("visible_extensions").is_some(),
                    no_extension
                );
                assert_eq!(value["version"], katana_ui::about_info::APP_VERSION);
                assert_eq!(value["updates"]["interval"], "Never");
            }
        }
        Ok(())
    }

    #[test]
    fn typography_font_setting_is_written_and_invalid_sizes_are_rejected() -> Result<()> {
        let root = tempfile::tempdir()?;
        let mut fixture = Fixture::default();
        fixture.settings.font_size = Some(14.0);
        let env = setup(&fixture, root.path())?;
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(env.config_dir.join("settings.json"))?)?;
        assert_eq!(value["font"]["size"], 14.0);
        for size in [0.0, 33.0, f32::NAN, f32::INFINITY] {
            fixture.settings.font_size = Some(size);
            assert!(setup(&fixture, root.path()).is_err());
        }
        Ok(())
    }

    #[test]
    fn screenshot_settings_can_disable_diagram_controls() {
        let settings = FixtureSettings {
            slideshow_show_diagram_controls: Some(false),
            ..FixtureSettings::default()
        };

        let json = build_settings_json(&settings, None);

        assert!(json.contains(r#""slideshow_show_diagram_controls": false"#));
    }

    #[test]
    fn screenshot_settings_can_force_the_minimum_auto_refresh_interval() {
        let settings = FixtureSettings {
            auto_refresh: Some(true),
            auto_refresh_interval_secs: Some(0.25),
            ..FixtureSettings::default()
        };

        let json = build_settings_json(&settings, None);

        assert!(json.contains(r#""auto_refresh": true"#));
        assert!(json.contains(r#""auto_refresh_interval_secs": 0.25"#));
    }

    #[test]
    fn screenshot_settings_use_the_target_katana_version() {
        let json = build_settings_json(&FixtureSettings::default(), None);

        assert!(json.contains(r#""interval": "Never""#));
        assert!(json.contains(&format!(
            r#""version": "{}""#,
            katana_ui::about_info::APP_VERSION
        )));
        assert!(json.contains(&format!(
            r#""previous_app_version": "{}""#,
            katana_ui::about_info::APP_VERSION
        )));
    }

    #[test]
    fn screenshot_settings_restore_the_fixture_workspace() {
        let root = tempfile::tempdir().unwrap();
        let settings_path = root.path().join("settings.json");
        let workspace = root.path().join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::write(
            &settings_path,
            build_settings_json(&FixtureSettings::default(), Some(&workspace)),
        )
        .unwrap();

        let loaded = JsonFileRepository::new(settings_path).load();

        assert_eq!(
            loaded.workspace.last_workspace.as_deref(),
            workspace.to_str()
        );
    }
}
