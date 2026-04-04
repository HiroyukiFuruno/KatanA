mod types;
pub use types::*;

use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use super::color_preset::DiagramColorPreset;
use crate::markdown::{DiagramBlock, DiagramResult};

impl PlantUmlRendererOps {
    pub fn jar_candidate_paths() -> Vec<PathBuf> {
        #[allow(clippy::single_match)]
        match std::env::var("PLANTUML_JAR") {
            Ok(env_path) => return vec![PathBuf::from(env_path)],
            Err(_) => {}
        }
        let mut paths = Vec::new();
        #[allow(clippy::useless_vec)]
        for prefix in vec!["/opt/homebrew", "/usr/local"] {
            paths.push(PathBuf::from(prefix).join("opt/plantuml/libexec/plantuml.jar"));
        }
        #[allow(clippy::single_match)]
        match std::env::current_exe() {
            Ok(exe) => {
                if let Some(dir) = exe.parent() {
                    paths.push(dir.join("plantuml.jar"));
                    paths.push(dir.join("renderers").join("plantuml.jar"));
                }
            }
            Err(_) => {}
        }
        if let Some(home) = dirs_sys::home_dir() {
            paths.push(home.join(".local").join("katana").join("plantuml.jar"));
        }
        paths
    }

    pub fn default_install_path() -> Option<PathBuf> {
        dirs_sys::home_dir().map(|h| h.join(".local").join("katana").join("plantuml.jar"))
    }

    pub fn find_plantuml_jar() -> Option<PathBuf> {
        Self::jar_candidate_paths().into_iter().find(|p| p.exists())
    }

    pub fn render_plantuml(block: &DiagramBlock) -> DiagramResult {
        let Some(jar) = Self::find_plantuml_jar() else {
            let install_path =
                Self::default_install_path().unwrap_or_else(|| PathBuf::from("plantuml.jar"));
            return DiagramResult::NotInstalled {
                kind: "PlantUML".to_string(),
                download_url:
                    "https://github.com/plantuml/plantuml/releases/latest/download/plantuml.jar"
                        .to_string(),
                install_path,
            };
        };
        match Self::run_plantuml_process(&jar, &block.source) {
            Ok(svg) => DiagramResult::Ok(Self::svg_to_html_fragment(&svg)),
            Err(e) => DiagramResult::Err {
                source: block.source.clone(),
                error: e,
            },
        }
    }

    pub fn run_plantuml_process(jar: &Path, source: &str) -> Result<String, String> {
        let preset = DiagramColorPreset::current();
        let themed_source = inject_theme(source, preset);
        let args = build_plantuml_args(jar);

        let mut child = Command::new("java")
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("java startup failed: {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(themed_source.as_bytes())
                .map_err(|e| format!("stdin write failed: {e}"))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("process wait failed: {e}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(format!("PlantUML rendering error: {stderr}"));
        }
        String::from_utf8(output.stdout).map_err(|e| format!("SVG decode error: {e}"))
    }

    pub fn svg_to_html_fragment(svg: &str) -> String {
        format!(r#"<div class="katana-diagram plantuml">{svg}</div>"#)
    }
}

fn inject_theme(source: &str, preset: &DiagramColorPreset) -> String {
    let skinparams = generate_skinparams(preset);
    if let Some(pos) = source.find("@startuml") {
        let insert_at = source[pos..]
            .find('\n')
            .map(|n| pos + n + 1)
            .unwrap_or(source.len());
        format!(
            "{}{}{}",
            &source[..insert_at],
            skinparams,
            &source[insert_at..]
        )
    } else {
        format!("@startuml\n{skinparams}{source}\n@enduml")
    }
}

fn generate_skinparams(preset: &DiagramColorPreset) -> String {
    format!(
        "\
skinparam backgroundColor {bg}
skinparam defaultFontColor {text}
skinparam classBorderColor {stroke}
skinparam classFontColor {text}
skinparam classBackgroundColor {fill}
skinparam arrowColor {arrow}
skinparam noteBorderColor {stroke}
skinparam noteBackgroundColor {note_bg}
skinparam noteFontColor {note_text}
skinparam sequenceLifeLineBorderColor {stroke}
skinparam sequenceParticipantBackgroundColor {fill}
skinparam sequenceParticipantBorderColor {stroke}
skinparam sequenceParticipantFontColor {text}
skinparam sequenceArrowColor {arrow}
",
        bg = preset.background,
        text = preset.text,
        stroke = preset.stroke,
        fill = preset.plantuml_class_bg,
        arrow = preset.arrow,
        note_bg = preset.plantuml_note_bg,
        note_text = preset.plantuml_note_text,
    )
}

fn build_plantuml_args(jar: &Path) -> Vec<String> {
    let mut args = vec![
        "-Djava.awt.headless=true".to_string(),
        "-jar".to_string(),
        jar.to_str().unwrap_or("plantuml.jar").to_string(),
        "-pipe".to_string(),
        "-tsvg".to_string(),
    ];
    if DiagramColorPreset::is_dark_mode() {
        args.push("-darkmode".to_string());
    }
    args
}

#[cfg(test)]
mod tests;
