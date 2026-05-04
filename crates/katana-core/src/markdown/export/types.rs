pub struct HtmlExporter;
pub struct ImageExporter;
pub struct PdfExporter;

/// Output format for file-level export (HTML is the intermediate; Pdf/Png/Jpeg are file targets).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    Html,
    Pdf,
    Png,
    Jpeg,
}

/// Input to `ExporterTrait::export`.
///
/// `html_source` is the already-rendered HTML string.  Callers are responsible
/// for converting Markdown → HTML (e.g. via `HtmlExporter::export_html`) before
/// passing it here.
#[derive(Clone, Debug)]
pub struct ExportInput {
    pub format: ExportFormat,
    pub html_source: String,
    pub output_path: std::path::PathBuf,
}

/// Successful result of `ExporterTrait::export`.
#[derive(Debug)]
pub struct ExportOutput {
    pub output_path: std::path::PathBuf,
    pub format: ExportFormat,
}

/// Error returned by `ExporterTrait::export`.
#[derive(Debug)]
pub enum ExportError {
    IoError(String),
    RenderFailed(String),
    UnsupportedFormat,
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {e}"),
            Self::RenderFailed(e) => write!(f, "render failed: {e}"),
            Self::UnsupportedFormat => write!(f, "unsupported format"),
        }
    }
}

/// Neutral export interface.  `PdfExporter` and `ImageExporter` implement this.
/// At kcf intake the KatanA-internal impl can be swapped for the external one
/// without touching call sites in `katana-ui`.
pub trait ExporterTrait: Send + Sync {
    fn export(&self, input: &ExportInput) -> Result<ExportOutput, ExportError>;
    fn supported_formats(&self) -> &[ExportFormat];
    /// Returns `true` when the underlying runtime is available on this machine.
    fn is_available(&self) -> bool {
        true
    }
}
