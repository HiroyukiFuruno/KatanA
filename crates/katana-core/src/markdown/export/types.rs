use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::diagram_backend::DiagramThemeSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Html,
    Pdf,
    Png,
    Jpeg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaperSize {
    A4,
}

#[derive(Debug, Clone)]
pub struct ExportConfig {
    pub theme: DiagramThemeSnapshot,
    pub paper_size: PaperSize,
}

#[derive(Debug, Clone)]
pub struct ExportInput {
    pub format: ExportFormat,
    pub markdown_source: String,
    pub source_path: std::path::PathBuf,
    pub output_path: std::path::PathBuf,
    pub config: ExportConfig,
}

#[derive(Debug, Clone)]
pub struct ExportOutput {
    pub format: ExportFormat,
    pub output_path: std::path::PathBuf,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("KDV export failed: {0}")]
    Kdv(String),
    #[error("failed to write export output to {path}: {source}")]
    Write {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("HTML export was not valid UTF-8: {0}")]
    InvalidHtml(std::string::FromUtf8Error),
}

pub trait ExporterTrait: Send + Sync {
    fn export(&self, input: &ExportInput) -> Result<ExportOutput, ExportError>;

    fn supported_formats(&self) -> &[ExportFormat];

    fn is_available(&self) -> bool {
        true
    }
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            theme: DiagramThemeSnapshot::current(),
            paper_size: PaperSize::A4,
        }
    }
}

impl ExportConfig {
    pub fn from_preset(preset: &DiagramColorPreset) -> Self {
        Self {
            theme: Self::theme_from_preset(preset),
            paper_size: PaperSize::A4,
        }
    }

    pub fn with_theme(theme: DiagramThemeSnapshot) -> Self {
        Self {
            theme,
            paper_size: PaperSize::A4,
        }
    }

    pub fn theme_from_preset(preset: &DiagramColorPreset) -> DiagramThemeSnapshot {
        DiagramThemeSnapshot::from_preset(
            if preset.dark_mode { "dark" } else { "light" },
            preset.dark_mode,
            preset,
        )
    }
}
