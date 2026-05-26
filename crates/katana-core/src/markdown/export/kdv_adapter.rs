use super::{
    ExportError, ExportFormat, ExportInput, ExportOutput,
    kdv_markdown_normalizer::KdvMarkdownNormalizer,
};
use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::kdv_theme_adapter::KdvThemeAdapter;
use katana_document_viewer::{
    BuildProfile, BuildRequest, DiagramRenderingBackend, DocumentSnapshotFactory, DocumentSource,
    ExportRequest, ForgePipeline, KrrDiagramRenderEngine, SourceKind, SourceRevision, SourceUri,
};
use katana_markdown_model::{KatanaMarkdownModel, MarkdownInput};

pub(super) struct KdvExportAdapter;

impl KdvExportAdapter {
    pub(super) fn export_html_string(
        source: &str,
        preset: &DiagramColorPreset,
        base_dir: Option<&std::path::Path>,
    ) -> Result<String, ExportError> {
        let source_path = base_dir
            .map(|dir| dir.join("document.md"))
            .unwrap_or_else(|| std::path::PathBuf::from("document.md"));
        let bytes = Self::export_bytes(source, &source_path, ExportFormat::Html, preset)?;
        String::from_utf8(bytes).map_err(ExportError::InvalidHtml)
    }

    pub(super) fn export_to_file(input: &ExportInput) -> Result<ExportOutput, ExportError> {
        let bytes = Self::export_bytes(
            &input.markdown_source,
            &input.source_path,
            input.format,
            &input.config.theme,
        )?;
        std::fs::write(&input.output_path, bytes).map_err(|source| ExportError::Write {
            path: input.output_path.clone(),
            source,
        })?;
        Ok(ExportOutput {
            format: input.format,
            output_path: input.output_path.clone(),
            diagnostics: Vec::new(),
        })
    }

    fn export_bytes(
        source: &str,
        source_path: &std::path::Path,
        format: ExportFormat,
        preset: &DiagramColorPreset,
    ) -> Result<Vec<u8>, ExportError> {
        let theme = KdvThemeAdapter::from_preset(preset);
        let snapshot = Self::document_snapshot(source, source_path)?;
        let pipeline = ForgePipeline::new(DiagramRenderingBackend::new(KrrDiagramRenderEngine));
        let graph = pipeline
            .build(&BuildRequest {
                snapshot,
                profile: BuildProfile::markdown_export(),
                theme: theme.clone(),
            })
            .map_err(|error| ExportError::Kdv(error.to_string()))?;
        let output = pipeline
            .export(&ExportRequest {
                graph,
                format: Self::kdv_format(format),
                theme,
            })
            .map_err(|error| ExportError::Kdv(error.to_string()))?;
        Ok(output.artifact.bytes.bytes)
    }

    fn document_snapshot(
        source: &str,
        source_path: &std::path::Path,
    ) -> Result<katana_document_viewer::DocumentSnapshot, ExportError> {
        let content = KdvMarkdownNormalizer::normalize(source);
        let document = KatanaMarkdownModel::parse(MarkdownInput::from_content(
            source_path.to_path_buf(),
            content.clone(),
        ))
        .map_err(|error| ExportError::Kdv(error.to_string()))?;
        let document_source = DocumentSource {
            uri: SourceUri(format!("file://{}", source_path.display())),
            kind: SourceKind::Markdown,
            revision: SourceRevision(document.fingerprint.value.clone()),
            content,
        };
        Ok(DocumentSnapshotFactory::from_kmm(document_source, document))
    }

    fn kdv_format(format: ExportFormat) -> katana_document_viewer::ExportFormat {
        match format {
            ExportFormat::Html => katana_document_viewer::ExportFormat::Html,
            ExportFormat::Pdf => katana_document_viewer::ExportFormat::Pdf,
            ExportFormat::Png => katana_document_viewer::ExportFormat::Png,
            ExportFormat::Jpeg => katana_document_viewer::ExportFormat::Jpeg,
        }
    }
}
