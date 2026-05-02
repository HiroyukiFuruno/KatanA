use crate::markdown::diagram_js_runtime::DiagramRuntimeScript;

pub(super) struct DrawioRuntimeScripts;

impl DrawioRuntimeScripts {
    pub(super) fn build<'a>(bundle: &'a str, request_json: &str) -> Vec<DiagramRuntimeScript<'a>> {
        vec![
            DiagramRuntimeScript::borrowed("browser-globals.js", BROWSER_GLOBALS),
            DiagramRuntimeScript::borrowed("dom-node.js", DOM_NODE),
            DiagramRuntimeScript::borrowed("dom-style.js", DOM_STYLE),
            DiagramRuntimeScript::borrowed("dom-html.js", DOM_HTML),
            DiagramRuntimeScript::borrowed("dom-node-html.js", DOM_NODE_HTML),
            DiagramRuntimeScript::borrowed("dom-selector.js", DOM_SELECTOR),
            DiagramRuntimeScript::borrowed("dom-document.js", DOM_DOCUMENT),
            DiagramRuntimeScript::borrowed("drawio-globals.js", DRAWIO_GLOBALS),
            DiagramRuntimeScript::borrowed("dom-tree.js", DOM_TREE),
            DiagramRuntimeScript::borrowed("dom-layout.js", DOM_LAYOUT),
            DiagramRuntimeScript::borrowed("dom-metrics.js", DOM_METRICS),
            DiagramRuntimeScript::borrowed("dom-layout-prototype.js", DOM_LAYOUT_PROTOTYPE),
            DiagramRuntimeScript::borrowed("dom-css-style.js", DOM_CSS_STYLE),
            DiagramRuntimeScript::borrowed("dom-computed-style.js", DOM_COMPUTED_STYLE),
            DiagramRuntimeScript::borrowed("dom-measure.js", DOM_MEASURE),
            DiagramRuntimeScript::borrowed("dom-measure-text-lines.js", DOM_MEASURE_TEXT_LINES),
            DiagramRuntimeScript::borrowed("dom-text-metrics.js", DOM_TEXT_METRICS),
            DiagramRuntimeScript::borrowed("dom-text-measure.js", DOM_TEXT_MEASURE),
            DiagramRuntimeScript::owned("drawio-request.js", request_script(request_json)),
            DiagramRuntimeScript::borrowed("drawio.min.js", bundle),
            DiagramRuntimeScript::borrowed("drawio-resources.js", DRAWIO_RESOURCES),
            DiagramRuntimeScript::borrowed("drawio-stencils.js", DRAWIO_STENCILS),
            DiagramRuntimeScript::borrowed("drawio-xml-document.js", DRAWIO_XML_DOCUMENT),
            DiagramRuntimeScript::borrowed("drawio-mx-adapter.js", DRAWIO_MX_ADAPTER),
            DiagramRuntimeScript::borrowed("drawio-runtime-adapter.js", DRAWIO_RUNTIME_ADAPTER),
            DiagramRuntimeScript::borrowed("drawio-svg-source-crop.js", DRAWIO_SVG_SOURCE_CROP),
            DiagramRuntimeScript::borrowed("drawio-svg-crop.js", DRAWIO_SVG_CROP),
            DiagramRuntimeScript::borrowed("drawio-svg-source-styles.js", DRAWIO_SVG_SOURCE_STYLES),
            DiagramRuntimeScript::borrowed("drawio-svg-light-dark.js", DRAWIO_SVG_LIGHT_DARK),
            DiagramRuntimeScript::borrowed("drawio-svg-paint.js", DRAWIO_SVG_PAINT),
            DiagramRuntimeScript::borrowed("drawio-svg-color-context.js", DRAWIO_SVG_COLOR_CONTEXT),
            DiagramRuntimeScript::borrowed("drawio-svg-azure-colors.js", DRAWIO_SVG_AZURE_COLORS),
            DiagramRuntimeScript::borrowed("drawio-svg-colors.js", DRAWIO_SVG_COLORS),
            DiagramRuntimeScript::borrowed("drawio-svg-gcp-colors.js", DRAWIO_SVG_GCP_COLORS),
            DiagramRuntimeScript::borrowed("drawio-svg-shape-colors.js", DRAWIO_SVG_SHAPE_COLORS),
            DiagramRuntimeScript::borrowed(
                "drawio-svg-style-color-tables.js",
                DRAWIO_SVG_STYLE_COLOR_TABLES,
            ),
            DiagramRuntimeScript::borrowed(
                "drawio-svg-style-color-parse.js",
                DRAWIO_SVG_STYLE_COLOR_PARSE,
            ),
            DiagramRuntimeScript::borrowed("drawio-svg-style-colors.js", DRAWIO_SVG_STYLE_COLORS),
            DiagramRuntimeScript::borrowed("drawio-svg-shadow.js", DRAWIO_SVG_SHADOW),
            DiagramRuntimeScript::borrowed("drawio-svg-labels.js", DRAWIO_SVG_LABELS),
            DiagramRuntimeScript::borrowed(
                "drawio-svg-rich-text-labels.js",
                DRAWIO_SVG_RICH_TEXT_LABELS,
            ),
            DiagramRuntimeScript::borrowed("drawio-svg-label-clip.js", DRAWIO_SVG_LABEL_CLIP),
            DiagramRuntimeScript::borrowed("drawio-svg-label-wrap.js", DRAWIO_SVG_LABEL_WRAP),
            DiagramRuntimeScript::borrowed(
                "drawio-svg-html-text-styles.js",
                DRAWIO_SVG_HTML_TEXT_STYLES,
            ),
            DiagramRuntimeScript::borrowed(
                "drawio-svg-html-text-labels.js",
                DRAWIO_SVG_HTML_TEXT_LABELS,
            ),
            DiagramRuntimeScript::borrowed("drawio-svg-html-tables.js", DRAWIO_SVG_HTML_TABLES),
            DiagramRuntimeScript::borrowed("drawio-svg-text-baseline.js", DRAWIO_SVG_TEXT_BASELINE),
            DiagramRuntimeScript::borrowed(
                "drawio-svg-foreign-object.js",
                DRAWIO_SVG_FOREIGN_OBJECT,
            ),
            DiagramRuntimeScript::borrowed("drawio-svg-geometry.js", DRAWIO_SVG_GEOMETRY),
            DiagramRuntimeScript::borrowed("drawio-svg-postprocess.js", DRAWIO_SVG_POSTPROCESS),
            DiagramRuntimeScript::borrowed("render-drawio.js", RENDER_DRAWIO),
        ]
    }
}

fn request_script(request_json: &str) -> String {
    format!("globalThis.__katanaDrawioRequest = {request_json};")
}

const BROWSER_GLOBALS: &str = include_str!("../mermaid_renderer/js_runtime/browser_globals.js");
const DOM_NODE: &str = include_str!("../mermaid_renderer/js_runtime/dom_node.js");
const DOM_STYLE: &str = include_str!("../mermaid_renderer/js_runtime/dom_style.js");
const DOM_HTML: &str = include_str!("../mermaid_renderer/js_runtime/dom_html.js");
const DOM_NODE_HTML: &str = include_str!("../mermaid_renderer/js_runtime/dom_node_html.js");
const DOM_SELECTOR: &str = include_str!("../mermaid_renderer/js_runtime/dom_selector.js");
const DOM_DOCUMENT: &str = include_str!("../mermaid_renderer/js_runtime/dom_document.js");
const DOM_TREE: &str = include_str!("../mermaid_renderer/js_runtime/dom_tree.js");
const DOM_LAYOUT: &str = include_str!("../mermaid_renderer/js_runtime/dom_layout.js");
const DOM_METRICS: &str = include_str!("../mermaid_renderer/js_runtime/dom_metrics.js");
const DOM_LAYOUT_PROTOTYPE: &str =
    include_str!("../mermaid_renderer/js_runtime/dom_layout_prototype.js");
const DOM_CSS_STYLE: &str = include_str!("../mermaid_renderer/js_runtime/dom_css_style.js");
const DOM_COMPUTED_STYLE: &str =
    include_str!("../mermaid_renderer/js_runtime/dom_computed_style.js");
const DOM_MEASURE: &str = include_str!("../mermaid_renderer/js_runtime/dom_measure.js");
const DOM_MEASURE_TEXT_LINES: &str =
    include_str!("../mermaid_renderer/js_runtime/dom_measure_text_lines.js");
const DOM_TEXT_METRICS: &str = include_str!("../mermaid_renderer/js_runtime/dom_text_metrics.js");
const DOM_TEXT_MEASURE: &str = include_str!("../mermaid_renderer/js_runtime/dom_text_measure.js");
const DRAWIO_GLOBALS: &str = include_str!("js_runtime/drawio_globals.js");
const DRAWIO_RESOURCES: &str = include_str!("js_runtime/drawio_resources.js");
const DRAWIO_STENCILS: &str = include_str!("js_runtime/drawio_stencils.js");
const DRAWIO_XML_DOCUMENT: &str = include_str!("js_runtime/drawio_xml_document.js");
const DRAWIO_MX_ADAPTER: &str = include_str!("js_runtime/drawio_mx_adapter.js");
const DRAWIO_RUNTIME_ADAPTER: &str = include_str!("js_runtime/drawio_runtime_adapter.js");
const DRAWIO_SVG_SOURCE_CROP: &str = include_str!("js_runtime/drawio_svg_source_crop.js");
const DRAWIO_SVG_CROP: &str = include_str!("js_runtime/drawio_svg_crop.js");
const DRAWIO_SVG_SOURCE_STYLES: &str = include_str!("js_runtime/drawio_svg_source_styles.js");
const DRAWIO_SVG_LIGHT_DARK: &str = include_str!("js_runtime/drawio_svg_light_dark.js");
const DRAWIO_SVG_PAINT: &str = include_str!("js_runtime/drawio_svg_paint.js");
const DRAWIO_SVG_COLOR_CONTEXT: &str = include_str!("js_runtime/drawio_svg_color_context.js");
const DRAWIO_SVG_AZURE_COLORS: &str = include_str!("js_runtime/drawio_svg_azure_colors.js");
const DRAWIO_SVG_COLORS: &str = include_str!("js_runtime/drawio_svg_colors.js");
const DRAWIO_SVG_GCP_COLORS: &str = include_str!("js_runtime/drawio_svg_gcp_colors.js");
const DRAWIO_SVG_SHAPE_COLORS: &str = include_str!("js_runtime/drawio_svg_shape_colors.js");
const DRAWIO_SVG_STYLE_COLOR_TABLES: &str =
    include_str!("js_runtime/drawio_svg_style_color_tables.js");
const DRAWIO_SVG_STYLE_COLOR_PARSE: &str =
    include_str!("js_runtime/drawio_svg_style_color_parse.js");
const DRAWIO_SVG_STYLE_COLORS: &str = include_str!("js_runtime/drawio_svg_style_colors.js");
const DRAWIO_SVG_SHADOW: &str = include_str!("js_runtime/drawio_svg_shadow.js");
const DRAWIO_SVG_LABELS: &str = include_str!("js_runtime/drawio_svg_labels.js");
const DRAWIO_SVG_RICH_TEXT_LABELS: &str = include_str!("js_runtime/drawio_svg_rich_text_labels.js");
const DRAWIO_SVG_LABEL_CLIP: &str = include_str!("js_runtime/drawio_svg_label_clip.js");
const DRAWIO_SVG_LABEL_WRAP: &str = include_str!("js_runtime/drawio_svg_label_wrap.js");
const DRAWIO_SVG_HTML_TEXT_STYLES: &str = include_str!("js_runtime/drawio_svg_html_text_styles.js");
const DRAWIO_SVG_HTML_TEXT_LABELS: &str = include_str!("js_runtime/drawio_svg_html_text_labels.js");
const DRAWIO_SVG_HTML_TABLES: &str = include_str!("js_runtime/drawio_svg_html_tables.js");
const DRAWIO_SVG_TEXT_BASELINE: &str = include_str!("js_runtime/drawio_svg_text_baseline.js");
const DRAWIO_SVG_FOREIGN_OBJECT: &str = include_str!("js_runtime/drawio_svg_foreign_object.js");
const DRAWIO_SVG_GEOMETRY: &str = include_str!("js_runtime/drawio_svg_geometry.js");
const DRAWIO_SVG_POSTPROCESS: &str = include_str!("js_runtime/drawio_svg_postprocess.js");
const RENDER_DRAWIO: &str = include_str!("js_runtime/render_drawio.js");
