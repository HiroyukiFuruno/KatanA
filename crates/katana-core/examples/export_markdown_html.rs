use katana_core::markdown::{
    ExportFormat as BackendExportFormat, ExportInput, ExporterTrait, HtmlExporter, ImageExporter,
    PdfExporter, color_preset::DiagramColorPreset,
};
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ExportOptions::parse(std::env::args().skip(1).collect())?;
    let source = std::fs::read_to_string(&options.input_path)?;
    prepare_output_parent(&options.output_path)?;
    options
        .format
        .export(&source, &options.input_path, &options.output_path)?;
    println!("{}", options.output_path.display());
    Ok(())
}

struct ExportOptions {
    input_path: PathBuf,
    output_path: PathBuf,
    format: ExportFormat,
}

impl ExportOptions {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let output_path = required_path_arg(&args, "--output")?;
        let format = optional_arg(&args, "--format")
            .map(ExportFormat::parse)
            .unwrap_or_else(|| ExportFormat::from_output_path(&output_path))?;
        Ok(Self {
            input_path: required_path_arg(&args, "--input")?,
            output_path,
            format,
        })
    }
}

enum ExportFormat {
    Html,
    Pdf,
    Png,
    Jpeg,
}

impl ExportFormat {
    fn parse(value: String) -> Result<Self, String> {
        match value.as_str() {
            "html" => Ok(Self::Html),
            "pdf" => Ok(Self::Pdf),
            "png" => Ok(Self::Png),
            "jpg" | "jpeg" => Ok(Self::Jpeg),
            _ => Err(format!("unsupported export format: {value}")),
        }
    }

    fn from_output_path(path: &Path) -> Result<Self, String> {
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .map(Self::parse)
            .unwrap_or_else(|| Ok(Self::Html))
    }

    fn export(
        &self,
        source: &str,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Html => {
                let html = HtmlExporter.export_markdown_to_html(
                    source,
                    DiagramColorPreset::dark(),
                    input_path.parent(),
                )?;
                Ok(std::fs::write(output_path, html)?)
            }
            Self::Pdf => {
                PdfExporter
                    .export(&ExportInput {
                        format: BackendExportFormat::Pdf,
                        markdown_source: source.to_string(),
                        source_path: input_path.to_path_buf(),
                        output_path: output_path.to_path_buf(),
                        config: Default::default(),
                    })
                    .map(|_| ())
                    .map_err(|e| std::io::Error::other(e.to_string()))?;
                Ok(())
            }
            Self::Png => {
                ImageExporter
                    .export(&ExportInput {
                        format: BackendExportFormat::Png,
                        markdown_source: source.to_string(),
                        source_path: input_path.to_path_buf(),
                        output_path: output_path.to_path_buf(),
                        config: Default::default(),
                    })
                    .map(|_| ())
                    .map_err(|e| std::io::Error::other(e.to_string()))?;
                Ok(())
            }
            Self::Jpeg => {
                ImageExporter
                    .export(&ExportInput {
                        format: BackendExportFormat::Jpeg,
                        markdown_source: source.to_string(),
                        source_path: input_path.to_path_buf(),
                        output_path: output_path.to_path_buf(),
                        config: Default::default(),
                    })
                    .map(|_| ())
                    .map_err(|e| std::io::Error::other(e.to_string()))?;
                Ok(())
            }
        }
    }
}

fn required_path_arg(args: &[String], name: &str) -> Result<PathBuf, String> {
    optional_arg(args, name)
        .map(PathBuf::from)
        .ok_or_else(|| usage(name))
}

fn optional_arg(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|it| it == name)
        .and_then(|index| args.get(index + 1))
        .cloned()
}

fn prepare_output_parent(output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn usage(missing_name: &str) -> String {
    format!(
        "usage: export_markdown_html --input FILE --output FILE [--format html|pdf|png|jpeg]; missing {missing_name}"
    )
}
