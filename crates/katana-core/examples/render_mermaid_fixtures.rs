use katana_core::markdown::{
    color_preset::DiagramColorPreset,
    diagram::{DiagramBlock, DiagramKind, DiagramResult},
    mermaid_renderer::MermaidRenderOps,
    svg_rasterize::SvgRasterizeOps,
};
#[path = "render_mermaid_fixtures/skipped_report.rs"]
mod skipped_report;
#[path = "render_mermaid_fixtures/source_markdown.rs"]
mod source_markdown;
use skipped_report::SkippedFixtureReport;
use source_markdown::extract_mermaid_block;
use std::path::{Path, PathBuf};

type DynError = Box<dyn std::error::Error>;

fn main() -> Result<(), DynError> {
    let options = RenderOptions::parse(std::env::args().skip(1).collect());
    DiagramColorPreset::set_dark_mode(options.dark_mode);
    std::fs::create_dir_all(&options.output_dir)?;
    let renderer = FixtureRenderer::new(&options.output_dir, options.skip_errors);
    let mut skipped_report = SkippedFixtureReport::new();
    for fixture_path in FixtureRepository::new(options.fixtures_dir).list()? {
        renderer.render(&fixture_path, &mut skipped_report)?;
    }
    skipped_report.write(&options.output_dir)?;
    Ok(())
}

struct RenderOptions {
    fixtures_dir: PathBuf,
    output_dir: PathBuf,
    skip_errors: bool,
    dark_mode: bool,
}

impl RenderOptions {
    fn parse(args: Vec<String>) -> Self {
        Self {
            fixtures_dir: path_arg(
                &args,
                "--fixtures",
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../assets/fixtures/mermaid_parts/en"),
            ),
            output_dir: path_arg(
                &args,
                "--output",
                PathBuf::from("tmp/mermaid-katana-rendered"),
            ),
            skip_errors: args.iter().any(|it| it == "--skip-errors"),
            dark_mode: theme_arg_is_dark(string_arg(&args, "--theme", "dark").as_str()),
        }
    }
}

fn theme_arg_is_dark(value: &str) -> bool {
    match value {
        "light" => false,
        "dark" => true,
        other => panic!("theme must be light or dark: {other}"),
    }
}

struct FixtureRepository {
    fixtures_dir: PathBuf,
}

impl FixtureRepository {
    fn new(fixtures_dir: PathBuf) -> Self {
        Self { fixtures_dir }
    }

    fn list(&self) -> Result<Vec<PathBuf>, DynError> {
        let mut fixtures = std::fs::read_dir(&self.fixtures_dir)?
            .map(|entry| entry.map(|it| it.path()))
            .collect::<Result<Vec<_>, _>>()?;
        fixtures.retain(|path| path.extension().and_then(|it| it.to_str()) == Some("md"));
        fixtures.sort();
        Ok(fixtures)
    }
}

struct FixtureRenderer<'a> {
    output_dir: &'a Path,
    skip_errors: bool,
}

impl<'a> FixtureRenderer<'a> {
    fn new(output_dir: &'a Path, skip_errors: bool) -> Self {
        Self {
            output_dir,
            skip_errors,
        }
    }

    fn render(
        &self,
        fixture_path: &Path,
        skipped_report: &mut SkippedFixtureReport,
    ) -> Result<(), DynError> {
        let slug = fixture_path
            .file_stem()
            .and_then(|it| it.to_str())
            .unwrap_or("fixture");
        match self.render_required(slug, fixture_path) {
            Ok(()) => Ok(()),
            Err(error) => self.handle_render_error(slug, fixture_path, error, skipped_report),
        }
    }

    fn render_required(&self, slug: &str, fixture_path: &Path) -> Result<(), DynError> {
        let markdown = std::fs::read_to_string(fixture_path)?;
        let svg = self.render_svg(slug, &extract_mermaid_block(&markdown))?;
        std::fs::write(self.output_dir.join(format!("{slug}.svg")), &svg)?;
        self.write_png(slug, &svg)?;
        println!("rendered {slug}");
        Ok(())
    }

    fn handle_render_error(
        &self,
        slug: &str,
        fixture_path: &Path,
        error: DynError,
        skipped_report: &mut SkippedFixtureReport,
    ) -> Result<(), DynError> {
        if self.skip_errors {
            let message = ErrorSummary::from(error.as_ref());
            skipped_report.add(slug, fixture_path, &message);
            self.remove_output_files(slug)?;
            eprintln!("skipped {slug}: {message}");
            return Ok(());
        }
        Err(error)
    }

    fn remove_output_files(&self, slug: &str) -> Result<(), DynError> {
        for extension in ["svg", "png"] {
            let path = self.output_dir.join(format!("{slug}.{extension}"));
            if path.exists() {
                std::fs::remove_file(path)?;
            }
        }
        Ok(())
    }

    fn render_svg(&self, slug: &str, source: &str) -> Result<String, DynError> {
        let block = DiagramBlock {
            kind: DiagramKind::Mermaid,
            source: source.to_string(),
        };
        match MermaidRenderOps::render_mermaid(&block) {
            DiagramResult::Ok(svg) => Ok(svg),
            other => Err(format!("{slug} did not render: {other:?}").into()),
        }
    }

    fn write_png(&self, slug: &str, svg: &str) -> Result<(), DynError> {
        let image = SvgRasterizeOps::rasterize_svg(svg, 1.0)?;
        image::save_buffer_with_format(
            self.output_dir.join(format!("{slug}.png")),
            &image.rgba,
            image.width,
            image.height,
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        )?;
        Ok(())
    }
}

struct ErrorSummary;

impl ErrorSummary {
    fn from(error: &dyn std::error::Error) -> String {
        Self::truncate(error.to_string().lines().next().unwrap_or("error"))
    }

    fn truncate(value: &str) -> String {
        const MAX_CHARS: usize = 180;
        if value.chars().count() > MAX_CHARS {
            return format!("{}...", value.chars().take(MAX_CHARS).collect::<String>());
        }
        value.to_string()
    }
}

fn path_arg(args: &[String], name: &str, fallback: PathBuf) -> PathBuf {
    string_arg(args, name, fallback.to_string_lossy().as_ref()).into()
}

fn string_arg(args: &[String], name: &str, fallback: &str) -> String {
    args.iter()
        .position(|it| it == name)
        .and_then(|index| args.get(index + 1))
        .cloned()
        .unwrap_or_else(|| fallback.to_string())
}
