use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const DEFAULT_VIEWPORT_WIDTH_PX: u32 = 960;
const DEFAULT_VIEWPORT_HEIGHT_PX: u32 = 720;
const DEFAULT_SCALE_PERCENT: u16 = 100;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PreviewInput {
    pub source: String,
    pub options: PreviewRenderOptions,
    pub theme: PreviewThemeSnapshot,
    pub workspace: PreviewWorkspaceContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct PreviewRenderOptions {
    pub viewport: PreviewViewport,
    pub enabled_extensions: Vec<PreviewExtension>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PreviewViewport {
    pub width_px: u32,
    pub height_px: u32,
    pub scale_percent: u16,
}

impl Default for PreviewViewport {
    fn default() -> Self {
        Self {
            width_px: DEFAULT_VIEWPORT_WIDTH_PX,
            height_px: DEFAULT_VIEWPORT_HEIGHT_PX,
            scale_percent: DEFAULT_SCALE_PERCENT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PreviewExtension {
    GfmTable,
    Math,
    Diagram,
    Emoji,
    Anchor,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PreviewThemeSnapshot {
    pub name: String,
    pub is_dark: bool,
    pub background: String,
    pub text: String,
    pub link: String,
    pub code_background: String,
    pub border: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PreviewWorkspaceContext {
    WorkspaceFile {
        workspace_root: PathBuf,
        document_path: PathBuf,
    },
    Detached {
        display_name: String,
    },
}
