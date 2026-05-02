use katana_core::markdown::{
    HtmlExporter, ImageExporter, KatanaRenderer, PdfExporter, color_preset::DiagramColorPreset,
};
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ExportOptions::parse(std::env::args().skip(1).collect())?;
    let source = std::fs::read_to_string(&options.input_path)?;
    let html = HtmlExporter::export(
        &source,
        &KatanaRenderer,
        DiagramColorPreset::dark(),
        options.input_path.parent(),
    )?;
    prepare_output_parent(&options.output_path)?;
    options.format.export(&html, &options.output_path)?;
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

    fn export(&self, html: &str, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Html => Ok(std::fs::write(output_path, html)?),
            Self::Pdf => Ok(PdfExporter::export(html, output_path)?),
            Self::Png | Self::Jpeg => Ok(ImageExporter::export(html, output_path)?),
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
