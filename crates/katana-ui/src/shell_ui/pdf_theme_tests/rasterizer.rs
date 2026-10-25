use katana_core::system::ProcessService;
use std::path::{Path, PathBuf};

const PDF_RASTER_DPI: &str = "120";

pub(super) struct PdfRasterizer {
    executable: PathBuf,
}

impl PdfRasterizer {
    pub(super) fn from_required_environment() -> Self {
        let executable = std::env::var_os("KATANA_PDFTOPPM")
            .map(PathBuf::from)
            .filter(|path| !path.as_os_str().is_empty())
            .expect("KATANA_PDFTOPPM must name the Poppler pdftoppm executable");
        assert!(
            executable.is_file(),
            "KATANA_PDFTOPPM must point to an existing file: {}",
            executable.display()
        );
        Self { executable }
    }

    pub(super) fn rasterize_first_page(&self, pdf: &Path, output_prefix: &Path) -> PathBuf {
        let program = self
            .executable
            .to_str()
            .expect("KATANA_PDFTOPPM path must be valid UTF-8");
        let mut command = ProcessService::create_command(program);
        command.args([
            "-f",
            "1",
            "-l",
            "1",
            "-singlefile",
            "-r",
            PDF_RASTER_DPI,
            "-png",
        ]);
        command.arg(pdf).arg(output_prefix);
        let output = ProcessService::output(command).expect("start configured pdftoppm process");
        assert!(
            output.status.success(),
            "pdftoppm failed for {}: {}",
            pdf.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        let png_path = output_prefix.with_extension("png");
        assert!(
            png_path.is_file(),
            "pdftoppm must create {}",
            png_path.display()
        );
        png_path
    }
}
