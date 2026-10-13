use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const DEFAULT_RENDER_TIMEOUT_MILLIS: u64 = 10_000;
const DEFAULT_SCALE_PERCENT: u16 = 100;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagramBackendLanguage {
    Mermaid,
    PlantUml,
    DrawIo,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiagramBackendId {
    pub language: DiagramBackendLanguage,
    pub implementation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiagramBackendVersion {
    pub value: String,
    pub runtime_version: String,
    pub renderer_profile: String,
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
    #[serde(default)]
    pub table_border: Option<String>,
    #[serde(default)]
    pub table_header_background: Option<String>,
    #[serde(default)]
    pub table_even_row_background: Option<String>,
    pub fill: String,
    pub stroke: String,
    pub arrow: String,
    pub drawio_label_color: String,
    pub mermaid_theme: String,
    pub plantuml_class_background: String,
    pub plantuml_note_background: String,
    pub plantuml_note_text: String,
    pub syntax_theme_dark: String,
    pub syntax_theme_light: String,
    pub preview_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramThemeOverride {
    pub name: String,
    pub is_dark: bool,
    pub background: String,
    pub text: String,
    pub preview_text: String,
    pub table_border: Option<String>,
    pub table_header_background: Option<String>,
    pub table_even_row_background: Option<String>,
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
    pub runtime_version: String,
    pub renderer_profile: String,
    pub language: DiagramBackendLanguage,
    pub source: String,
    pub options: DiagramRenderOptions,
    pub render_config: String,
    pub render_policy: String,
    pub theme_fingerprint: String,
    pub theme: DiagramThemeSnapshot,
}
