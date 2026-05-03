use crate::markdown::MarkdownError;
use crate::markdown::export::native_style::NativeDocumentStyle;
use crate::markdown::svg_rasterize::{RasterizedSvg, SvgRasterizeOps};

const DOCUMENT_FALLBACK_TITLE: &str = "Exported document";
const JPEG_QUALITY_PERCENT: u8 = 90;
const RGB_CHANNELS: usize = 3;
const ALPHA_CHANNEL_INDEX: usize = 3;
const MAX_ALPHA: u16 = 255;
const TEXT_CAPTION_TRUNCATED: &str = "... Export truncated by native export safety limit.";
const PAGE_WIDTH: u32 = 900;
const MIN_PAGE_HEIGHT: u32 = 480;
const PAGE_MARGIN: u32 = 48;
const BLOCK_GAP: u32 = 22;
const HEADING_EXTRA_GAP: u32 = 10;
const CODE_FONT_FAMILY: &str =
    "Menlo, Consolas, Monaco, Liberation Mono, Lucida Console, monospace";
const MAX_TEXT_LINES: usize = 600;
const RGBA_BYTES: usize = 4;

pub(crate) struct NativeHtmlDocument {
    blocks: Vec<super::native_blocks::NativeDocumentBlock>,
    style: NativeDocumentStyle,
}

pub(crate) struct NativeDocumentImage {
    pub(crate) width: u32,
    pub(crate) height: u32,
    rgba: Vec<u8>,
}

impl NativeHtmlDocument {
    pub(crate) fn parse(html: &str) -> Result<Self, MarkdownError> {
        let style = NativeDocumentStyle::parse(html);
        let is_dark = super::native_text::is_dark_background(style.background_color());
        super::native_blocks::NativeDocumentBlocks::parse(html, is_dark)
            .map(|blocks| Self { blocks, style })
    }

    pub(crate) fn render_image(&self) -> Result<NativeDocumentImage, MarkdownError> {
        let svg = self.render_svg()?;
        SvgRasterizeOps::rasterize_svg(&svg, 1.0)
            .map(NativeDocumentImage::from)
            .map_err(|error| MarkdownError::ExportFailed(error.to_string()))
    }

    fn render_svg(&self) -> Result<String, MarkdownError> {
        let blocks = self.visible_blocks();
        let mut content = String::new();
        let mut y = PAGE_MARGIN;
        for block in &blocks {
            match block {
                super::native_blocks::NativeDocumentBlock::Text(line) => {
                    if line.is_heading() {
                        y += HEADING_EXTRA_GAP;
                    }
                    y += line.line_height();
                    content.push_str(&self.text_element(line, y));
                }
                super::native_blocks::NativeDocumentBlock::Svg(svg) => {
                    y += BLOCK_GAP;
                    let scale = svg.scale_for(PAGE_WIDTH - PAGE_MARGIN * 2);
                    content.push_str(&self.svg_element(svg, y, scale));
                    y += (svg.height as f32 * scale).ceil() as u32 + BLOCK_GAP;
                }
            }
        }
        let page_height = (y + PAGE_MARGIN).max(MIN_PAGE_HEIGHT);
        let background_color = self.style.background_color();
        Ok(format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{PAGE_WIDTH}" height="{page_height}" viewBox="0 0 {PAGE_WIDTH} {page_height}"><rect width="100%" height="100%" fill="{background_color}"/>{content}</svg>"##
        ))
    }

    fn visible_blocks(&self) -> Vec<super::native_blocks::NativeDocumentBlock> {
        let mut blocks = if self.blocks.is_empty() {
            vec![super::native_blocks::NativeDocumentBlock::Text(
                super::native_text::NativeTextLine::body(DOCUMENT_FALLBACK_TITLE.to_string()),
            )]
        } else {
            self.blocks.clone()
        };
        truncate_text_blocks(&mut blocks);
        blocks
    }

    fn text_element(&self, line: &super::native_text::NativeTextLine, y: u32) -> String {
        let text_color = self.style.text_color();
        let font_family = if line.is_code {
            CODE_FONT_FAMILY
        } else {
            super::native_text_runs::NativeTextRuns::font_family()
        };
        let font_weight = if line.bold { "bold" } else { "normal" };
        let font_size = line.font_size;

        if line.spans.is_empty() {
            let content = super::native_text_runs::NativeTextRuns::render(&line.text);
            format!(
                r##"<text x="{PAGE_MARGIN}" y="{y}" font-size="{font_size}" font-weight="{font_weight}" font-family="{font_family}" fill="{text_color}">{content}</text>"##,
            )
        } else {
            let spans_html: String = line
                .spans
                .iter()
                .map(|span| {
                    let [r, g, b] = span.color;
                    let color = format!("#{r:02x}{g:02x}{b:02x}");
                    let text = super::native_text_runs::NativeTextRuns::render(&span.text);
                    format!(r##"<tspan fill="{color}">{text}</tspan>"##)
                })
                .collect();
            format!(
                r##"<text x="{PAGE_MARGIN}" y="{y}" font-size="{font_size}" font-weight="{font_weight}" font-family="{font_family}">{spans_html}</text>"##,
            )
        }
    }

    fn svg_element(
        &self,
        svg: &super::native_blocks::NativeSvgBlock,
        y: u32,
        scale: f32,
    ) -> String {
        let scale = format!("{scale:.4}");
        format!(
            r#"<g transform="translate({PAGE_MARGIN} {y}) scale({scale})">{}</g>"#,
            svg.svg
        )
    }
}

impl NativeDocumentImage {
    pub(crate) fn save_png(&self, output: &std::path::Path) -> Result<(), MarkdownError> {
        let image = self.rgba_image()?;
        image
            .save_with_format(output, image::ImageFormat::Png)
            .map_err(|error| MarkdownError::ExportFailed(error.to_string()))
    }

    pub(crate) fn save_jpeg(&self, output: &std::path::Path) -> Result<(), MarkdownError> {
        let bytes = self.jpeg_bytes()?;
        std::fs::write(output, bytes)
            .map_err(|error| MarkdownError::ExportFailed(error.to_string()))
    }

    pub(crate) fn jpeg_bytes(&self) -> Result<Vec<u8>, MarkdownError> {
        let rgb = self.rgb_image()?;
        let mut bytes = Vec::new();
        let mut encoder =
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, JPEG_QUALITY_PERCENT);
        encoder
            .encode_image(&rgb)
            .map_err(|error| MarkdownError::ExportFailed(error.to_string()))?;
        Ok(bytes)
    }

    fn rgba_image(&self) -> Result<image::RgbaImage, MarkdownError> {
        image::RgbaImage::from_raw(self.width, self.height, self.rgba.clone()).ok_or_else(|| {
            MarkdownError::ExportFailed("native image buffer has invalid dimensions".to_string())
        })
    }

    fn rgb_image(&self) -> Result<image::RgbImage, MarkdownError> {
        let mut pixels =
            Vec::with_capacity((self.width * self.height * RGB_CHANNELS as u32) as usize);
        for chunk in self.rgba.chunks_exact(RGBA_BYTES) {
            let alpha = u16::from(chunk[ALPHA_CHANNEL_INDEX]);
            pixels.push(composite_over_white(chunk[0], alpha));
            pixels.push(composite_over_white(chunk[1], alpha));
            pixels.push(composite_over_white(chunk[2], alpha));
        }
        image::RgbImage::from_raw(self.width, self.height, pixels).ok_or_else(|| {
            MarkdownError::ExportFailed("native RGB buffer has invalid dimensions".to_string())
        })
    }
}

impl From<RasterizedSvg> for NativeDocumentImage {
    fn from(value: RasterizedSvg) -> Self {
        Self {
            width: value.width,
            height: value.height,
            rgba: value.rgba,
        }
    }
}

fn composite_over_white(value: u8, alpha: u16) -> u8 {
    (((u16::from(value) * alpha) + (MAX_ALPHA * (MAX_ALPHA - alpha))) / MAX_ALPHA) as u8
}

fn truncate_text_blocks(blocks: &mut Vec<super::native_blocks::NativeDocumentBlock>) {
    let mut text_count = 0;
    blocks.retain(|block| match block {
        super::native_blocks::NativeDocumentBlock::Text(_) => {
            text_count += 1;
            text_count <= MAX_TEXT_LINES
        }
        super::native_blocks::NativeDocumentBlock::Svg(_) => true,
    });
    if text_count > MAX_TEXT_LINES {
        blocks.push(super::native_blocks::NativeDocumentBlock::Text(
            super::native_text::NativeTextLine::body(TEXT_CAPTION_TRUNCATED.to_string()),
        ));
    }
}
