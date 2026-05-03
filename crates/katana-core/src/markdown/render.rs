use comrak::{Options, markdown_to_html};

use super::drawio_renderer;
use super::fence::MarkdownFenceOps;
use super::mermaid_renderer;
use super::plantuml_renderer;
use super::types::*;

impl DiagramRenderer for KatanaRenderer {
    fn render(&self, block: &DiagramBlock) -> DiagramResult {
        match block.kind {
            DiagramKind::Mermaid => mermaid_renderer::MermaidRenderOps::render_mermaid(block),
            DiagramKind::PlantUml => plantuml_renderer::PlantUmlRendererOps::render_plantuml(block),
            DiagramKind::DrawIo => drawio_renderer::DrawioRendererOps::render_drawio(block),
        }
    }
}

impl MarkdownRenderOps {
    pub fn gfm_options() -> Options<'static> {
        let mut opts = Options::default();
        opts.extension.strikethrough = true;
        opts.extension.table = true;
        opts.extension.autolink = true;
        opts.extension.tasklist = true;
        opts.extension.footnotes = true;
        /* WHY: GFM Alerts ([!NOTE], [!TIP], etc.) must be enabled
        to match the preview pane's rendering of alert blocks. */
        opts.extension.alerts = true;
        /* WHY: Math support ($inline$ and $$block$$) to match preview. */
        opts.extension.math_dollars = true;
        opts.extension.math_code = true;
        /* WHY: Generate heading IDs so exported HTML anchors work. */
        opts.extension.header_id_prefix = Some(String::new());
        opts.extension.description_lists = true;
        opts.render.r#unsafe = true;
        opts
    }

    pub fn render_basic(source: &str) -> Result<RenderOutput, MarkdownError> {
        Self::render(source, &NoOpRenderer)
    }

    pub fn render_with_katana_renderer(source: &str) -> Result<RenderOutput, MarkdownError> {
        Self::render(source, &KatanaRenderer)
    }

    pub fn render<R: DiagramRenderer>(
        source: &str,
        renderer: &R,
    ) -> Result<RenderOutput, MarkdownError> {
        let transformed = MarkdownFenceOps::transform_diagram_blocks(source, renderer);
        let protected = DiagramHtmlProtector::protect(&transformed);
        let math_repaired = Self::repair_inline_math_spaces(&protected.markdown);
        let html = markdown_to_html(&math_repaired, &Self::gfm_options());
        Ok(RenderOutput {
            html: protected.restore(&html),
        })
    }

    /// Repairs `$ lenient $` math to `$strict$` so that comrak recognizes it.
    fn repair_inline_math_spaces(source: &str) -> String {
        use std::sync::LazyLock;
        static MATH_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
            /* WHY: Match single `$` delimiters, capturing inner contents without `$` characters */
            regex::Regex::new(r"(?s)\$([ \t]*)([^$]+?)([ \t]*)\$").unwrap()
        });

        MATH_REGEX
            .replace_all(source, |caps: &regex::Captures| {
                /* WHY: caps[2] is the inner content stripped of boundary spaces */
                format!("${}$", &caps[2])
            })
            .to_string()
    }

    pub fn transform_only<R: DiagramRenderer>(
        source: &str,
        renderer: &R,
    ) -> Result<RenderOutput, MarkdownError> {
        let html = MarkdownFenceOps::transform_diagram_blocks(source, renderer);
        Ok(RenderOutput { html })
    }
}

struct DiagramHtmlProtector {
    markdown: String,
    fragments: Vec<DiagramHtmlFragment>,
}

struct DiagramHtmlFragment {
    placeholder: String,
    html: String,
}

impl DiagramHtmlProtector {
    fn protect(source: &str) -> Self {
        let mut markdown = String::with_capacity(source.len());
        let mut fragments = Vec::new();
        let mut position = 0;
        while let Some(range) = DiagramHtmlRange::next(source, position) {
            markdown.push_str(&source[position..range.start]);
            let placeholder = format!("<!--KATANA_DIAGRAM_HTML_PLACEHOLDER_{}-->", fragments.len());
            markdown.push_str(&placeholder);
            fragments.push(DiagramHtmlFragment {
                placeholder,
                html: source[range.start..range.end].to_string(),
            });
            position = range.end;
        }
        markdown.push_str(&source[position..]);
        Self {
            markdown,
            fragments,
        }
    }

    fn restore(&self, html: &str) -> String {
        self.fragments
            .iter()
            .fold(html.to_string(), |current, fragment| {
                current.replace(&fragment.placeholder, &fragment.html)
            })
    }
}

struct DiagramHtmlRange {
    start: usize,
    end: usize,
}

impl DiagramHtmlRange {
    fn next(source: &str, offset: usize) -> Option<Self> {
        let svg = Self::find(source, offset, "<svg", "</svg>");
        let div = Self::find(source, offset, r#"<div class="katana-diagram"#, "</div>");
        [svg, div].into_iter().flatten().min_by_key(|it| it.start)
    }

    fn find(source: &str, offset: usize, start_marker: &str, end_marker: &str) -> Option<Self> {
        let start = offset + source[offset..].find(start_marker)?;
        let after_start = start + start_marker.len();
        let end = after_start + source[after_start..].find(end_marker)? + end_marker.len();
        Some(Self { start, end })
    }
}
