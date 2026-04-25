use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::markdown::color_preset::DiagramColorPreset;

const DEFAULT_RENDER_TIMEOUT_MILLIS: u64 = 10_000;
const DEFAULT_SCALE_PERCENT: u16 = 100;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagramBackendLanguage {
    Mermaid,
    PlantUml,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiagramBackendId {
    pub language: DiagramBackendLanguage,
    pub implementation: String,
}

impl DiagramBackendId {
    pub fn new(language: DiagramBackendLanguage, implementation: impl Into<String>) -> Self {
        Self {
            language,
            implementation: implementation.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiagramBackendVersion {
    pub value: String,
}

impl DiagramBackendVersion {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagramOutputFormat {
    Html,
    Png,
    Svg,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiagramRenderOptions {
    pub output_format: DiagramOutputFormat,
    pub timeout_millis: u64,
    pub scale_percent: u16,
}

impl Default for DiagramRenderOptions {
    fn default() -> Self {
        Self {
            output_format: DiagramOutputFormat::Png,
            timeout_millis: DEFAULT_RENDER_TIMEOUT_MILLIS,
            scale_percent: DEFAULT_SCALE_PERCENT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiagramThemeSnapshot {
    pub name: String,
    pub is_dark: bool,
    pub background: String,
    pub text: String,
    pub fill: String,
    pub stroke: String,
    pub arrow: String,
    pub mermaid_theme: String,
    pub plantuml_class_background: String,
    pub plantuml_note_background: String,
    pub plantuml_note_text: String,
}

impl DiagramThemeSnapshot {
    pub fn from_preset(
        name: impl Into<String>,
        is_dark: bool,
        preset: &DiagramColorPreset,
    ) -> Self {
        Self {
            name: name.into(),
            is_dark,
            background: preset.background.to_string(),
            text: preset.text.to_string(),
            fill: preset.fill.to_string(),
            stroke: preset.stroke.to_string(),
            arrow: preset.arrow.to_string(),
            mermaid_theme: preset.mermaid_theme.to_string(),
            plantuml_class_background: preset.plantuml_class_bg.to_string(),
            plantuml_note_background: preset.plantuml_note_bg.to_string(),
            plantuml_note_text: preset.plantuml_note_text.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagramDocumentContext {
    WorkspaceFile {
        workspace_root: PathBuf,
        document_path: PathBuf,
    },
    Detached {
        display_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiagramBackendInput {
    pub language: DiagramBackendLanguage,
    pub source: String,
    pub options: DiagramRenderOptions,
    pub theme: DiagramThemeSnapshot,
    pub document: DiagramDocumentContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiagramBackendCacheKey {
    pub backend_id: DiagramBackendId,
    pub backend_version: DiagramBackendVersion,
    pub language: DiagramBackendLanguage,
    pub source: String,
    pub options: DiagramRenderOptions,
    pub theme: DiagramThemeSnapshot,
}

impl DiagramBackendCacheKey {
    pub fn new(
        backend_id: DiagramBackendId,
        backend_version: DiagramBackendVersion,
        input: &DiagramBackendInput,
    ) -> Self {
        Self {
            backend_id,
            backend_version,
            language: input.language.clone(),
            source: input.source.clone(),
            options: input.options.clone(),
            theme: input.theme.clone(),
        }
    }
}
