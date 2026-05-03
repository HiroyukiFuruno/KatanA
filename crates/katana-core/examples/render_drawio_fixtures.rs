use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::drawio_renderer::DrawioRenderOps;
use katana_core::markdown::svg_rasterize::{RasterizedSvg, SvgRasterizeOps};
use std::path::{Path, PathBuf};

const CAPTURE_PADDING: u32 = 12;
const CAPTURE_BACKGROUND: [u8; 4] = [0x1e, 0x1e, 0x1e, 0xff];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .try_init();
    let options = RenderOptions::parse(std::env::args().skip(1).collect());
    std::fs::create_dir_all(&options.output_dir)?;
    for fixture_path in FixtureRepository::new(options.fixtures_dir).list()? {
        FixtureRenderer::new(&options.output_dir).render(&fixture_path)?;
    }
    Ok(())
}

struct RenderOptions {
    fixtures_dir: PathBuf,
    output_dir: PathBuf,
}

impl RenderOptions {
    fn parse(args: Vec<String>) -> Self {
        Self {
            fixtures_dir: path_arg(
                &args,
                "--fixtures",
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fixtures/drawio/basic"),
            ),
            output_dir: path_arg(
                &args,
                "--output",
                PathBuf::from("tmp/drawio-katana-rendered"),
            ),
        }
    }
}

struct FixtureRepository {
    fixtures_dir: PathBuf,
}

impl FixtureRepository {
    fn new(fixtures_dir: PathBuf) -> Self {
        Self { fixtures_dir }
    }

    fn list(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut fixtures = std::fs::read_dir(&self.fixtures_dir)?
            .map(|entry| entry.map(|it| it.path()))
            .collect::<Result<Vec<_>, _>>()?;
        fixtures.retain(|path| path.extension().is_some_and(|it| it == "drawio"));
        fixtures.sort();
        Ok(fixtures)
    }
}

struct FixtureRenderer<'a> {
    output_dir: &'a Path,
}

impl<'a> FixtureRenderer<'a> {
    fn new(output_dir: &'a Path) -> Self {
        Self { output_dir }
    }

    fn render(&self, fixture_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let slug = fixture_path
            .file_stem()
            .and_then(|it| it.to_str())
            .unwrap_or("fixture");
        let source = std::fs::read_to_string(fixture_path)?;
        let svg = self.render_svg(slug, &source)?;
        std::fs::write(self.output_dir.join(format!("{slug}.svg")), &svg)?;
        self.write_png(slug, &svg)?;
        println!("rendered {slug}");
        Ok(())
    }

    fn render_svg(&self, slug: &str, source: &str) -> Result<String, Box<dyn std::error::Error>> {
        let block = DiagramBlock {
            kind: DiagramKind::DrawIo,
            source: source.to_string(),
        };
        match DrawioRenderOps::render_drawio(&block) {
            DiagramResult::Ok(svg) => Ok(svg),
            other => Err(format!("{slug} did not render: {other:?}").into()),
        }
    }

    fn write_png(&self, slug: &str, svg: &str) -> Result<(), Box<dyn std::error::Error>> {
        let image = SvgRasterizeOps::rasterize_svg(svg, 1.0)?;
        let canvas = CaptureCanvas::from_rasterized(&image);
        image::save_buffer_with_format(
            self.output_dir.join(format!("{slug}.png")),
            &canvas.rgba,
            canvas.width,
            canvas.height,
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        )?;
        Ok(())
    }
}

struct CaptureCanvas {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl CaptureCanvas {
    fn from_rasterized(image: &RasterizedSvg) -> Self {
        let width = image.width + CAPTURE_PADDING * 2;
        let height = image.height + CAPTURE_PADDING * 2;
        let mut rgba = vec![0; (width * height * 4) as usize];
        rgba.chunks_exact_mut(4)
            .for_each(|pixel| pixel.copy_from_slice(&CAPTURE_BACKGROUND));
        CaptureCanvas::copy_image(&mut rgba, width, image);
        Self {
            width,
            height,
            rgba,
        }
    }

    fn copy_image(rgba: &mut [u8], canvas_width: u32, image: &RasterizedSvg) {
        for y in 0..image.height {
            for x in 0..image.width {
                let source = CaptureCanvas::source_pixel(image, x, y);
                let target = CaptureCanvas::target_pixel(rgba, canvas_width, x, y);
                target.copy_from_slice(&CaptureCanvas::blend_pixel(source));
            }
        }
    }

    fn source_pixel(image: &RasterizedSvg, x: u32, y: u32) -> &[u8] {
        let index = ((y * image.width + x) * 4) as usize;
        &image.rgba[index..index + 4]
    }

    fn target_pixel(rgba: &mut [u8], canvas_width: u32, x: u32, y: u32) -> &mut [u8] {
        let row = y + CAPTURE_PADDING;
        let col = x + CAPTURE_PADDING;
        let index = ((row * canvas_width + col) * 4) as usize;
        &mut rgba[index..index + 4]
    }

    fn blend_pixel(source: &[u8]) -> [u8; 4] {
        let alpha = f32::from(source[3]) / 255.0;
        [
            CaptureCanvas::blend_channel(source[0], CAPTURE_BACKGROUND[0], alpha),
            CaptureCanvas::blend_channel(source[1], CAPTURE_BACKGROUND[1], alpha),
            CaptureCanvas::blend_channel(source[2], CAPTURE_BACKGROUND[2], alpha),
            0xff,
        ]
    }

    fn blend_channel(source: u8, background: u8, alpha: f32) -> u8 {
        (f32::from(source) * alpha + f32::from(background) * (1.0 - alpha)).round() as u8
    }
}

fn path_arg(args: &[String], name: &str, fallback: PathBuf) -> PathBuf {
    args.iter()
        .position(|it| it == name)
        .and_then(|index| args.get(index + 1))
        .map(PathBuf::from)
        .unwrap_or(fallback)
}
