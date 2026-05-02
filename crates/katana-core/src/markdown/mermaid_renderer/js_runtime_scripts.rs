use crate::markdown::diagram_js_runtime::DiagramRuntimeScript;

pub(super) struct MermaidRuntimeScripts;

impl MermaidRuntimeScripts {
    pub(super) fn build<'a>(bundle: &'a str, request_json: &str) -> Vec<DiagramRuntimeScript<'a>> {
        vec![
            DiagramRuntimeScript::borrowed("browser-globals.js", BROWSER_GLOBALS),
            DiagramRuntimeScript::borrowed("dom-node.js", DOM_NODE),
            DiagramRuntimeScript::borrowed("dom-style.js", DOM_STYLE),
            DiagramRuntimeScript::borrowed("dom-html.js", DOM_HTML),
            DiagramRuntimeScript::borrowed("dom-node-html.js", DOM_NODE_HTML),
            DiagramRuntimeScript::borrowed("dom-selector.js", DOM_SELECTOR),
            DiagramRuntimeScript::borrowed("dom-document.js", DOM_DOCUMENT),
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
            DiagramRuntimeScript::borrowed("svg-bounds-attrs.js", SVG_BOUNDS_ATTRS),
            DiagramRuntimeScript::borrowed("svg-path-bounds.js", SVG_PATH_BOUNDS),
            DiagramRuntimeScript::borrowed("svg-bounds.js", SVG_BOUNDS),
            DiagramRuntimeScript::borrowed("svg-text-bounds.js", SVG_TEXT_BOUNDS),
            DiagramRuntimeScript::borrowed("svg-architecture-icons.js", SVG_ARCHITECTURE_ICONS),
            DiagramRuntimeScript::borrowed("svg-group-utils.js", SVG_GROUP_UTILS),
            DiagramRuntimeScript::borrowed("svg-viewbox-fixes.js", SVG_VIEWBOX_FIXES),
            DiagramRuntimeScript::borrowed("svg-er-layout.js", SVG_ER_LAYOUT),
            DiagramRuntimeScript::borrowed("svg-er-fixes.js", SVG_ER_FIXES),
            DiagramRuntimeScript::borrowed("svg-class-fixes.js", SVG_CLASS_FIXES),
            DiagramRuntimeScript::borrowed("svg-visual-fixes.js", SVG_VISUAL_FIXES),
            DiagramRuntimeScript::borrowed("svg-review-fixes.js", SVG_REVIEW_FIXES),
            DiagramRuntimeScript::borrowed(
                "svg-review-ishikawa-fixes.js",
                SVG_REVIEW_ISHIKAWA_FIXES,
            ),
            DiagramRuntimeScript::borrowed("svg-review-venn-fixes.js", SVG_REVIEW_VENN_FIXES),
            DiagramRuntimeScript::borrowed("svg-review-treemap-fixes.js", SVG_REVIEW_TREEMAP_FIXES),
            DiagramRuntimeScript::borrowed("svg-kanban-text.js", SVG_KANBAN_TEXT),
            DiagramRuntimeScript::borrowed("svg-kanban-viewbox.js", SVG_KANBAN_VIEWBOX),
            DiagramRuntimeScript::borrowed("svg-kanban-label-layout.js", SVG_KANBAN_LABEL_LAYOUT),
            DiagramRuntimeScript::borrowed("svg-kanban-fixes.js", SVG_KANBAN_FIXES),
            DiagramRuntimeScript::borrowed("svg-wardley-normalizer.js", SVG_WARDLEY_NORMALIZER),
            DiagramRuntimeScript::borrowed("svg-diagram-normalizers.js", SVG_DIAGRAM_NORMALIZERS),
            DiagramRuntimeScript::borrowed("svg-dimensions.js", SVG_DIMENSIONS),
            DiagramRuntimeScript::borrowed("source-i18n-context.js", SOURCE_I18N_CONTEXT),
            DiagramRuntimeScript::borrowed("source-i18n-normalize.js", SOURCE_I18N_NORMALIZE),
            DiagramRuntimeScript::borrowed("svg-normalize.js", SVG_NORMALIZE),
            DiagramRuntimeScript::borrowed("mermaid.min.js", bundle),
            DiagramRuntimeScript::owned("render-mermaid.js", render_script(request_json)),
        ]
    }
}

fn render_script(request_json: &str) -> String {
    format!(
        "globalThis.__katanaMermaidRequest = {request_json};\n{}",
        RENDER_MERMAID
    )
}

const BROWSER_GLOBALS: &str = include_str!("js_runtime/browser_globals.js");
const DOM_NODE: &str = include_str!("js_runtime/dom_node.js");
const DOM_STYLE: &str = include_str!("js_runtime/dom_style.js");
const DOM_HTML: &str = include_str!("js_runtime/dom_html.js");
const DOM_NODE_HTML: &str = include_str!("js_runtime/dom_node_html.js");
const DOM_SELECTOR: &str = include_str!("js_runtime/dom_selector.js");
const DOM_DOCUMENT: &str = include_str!("js_runtime/dom_document.js");
const DOM_TREE: &str = include_str!("js_runtime/dom_tree.js");
const DOM_LAYOUT: &str = include_str!("js_runtime/dom_layout.js");
const DOM_METRICS: &str = include_str!("js_runtime/dom_metrics.js");
const DOM_LAYOUT_PROTOTYPE: &str = include_str!("js_runtime/dom_layout_prototype.js");
const DOM_CSS_STYLE: &str = include_str!("js_runtime/dom_css_style.js");
const DOM_COMPUTED_STYLE: &str = include_str!("js_runtime/dom_computed_style.js");
const DOM_MEASURE: &str = include_str!("js_runtime/dom_measure.js");
const DOM_MEASURE_TEXT_LINES: &str = include_str!("js_runtime/dom_measure_text_lines.js");
const DOM_TEXT_METRICS: &str = include_str!("js_runtime/dom_text_metrics.js");
const DOM_TEXT_MEASURE: &str = include_str!("js_runtime/dom_text_measure.js");
const SVG_BOUNDS_ATTRS: &str = include_str!("js_runtime/svg_bounds_attrs.js");
const SVG_PATH_BOUNDS: &str = include_str!("js_runtime/svg_path_bounds.js");
const SVG_BOUNDS: &str = include_str!("js_runtime/svg_bounds.js");
const SVG_TEXT_BOUNDS: &str = include_str!("js_runtime/svg_text_bounds.js");
const SVG_ARCHITECTURE_ICONS: &str = include_str!("js_runtime/svg_architecture_icons.js");
const SVG_GROUP_UTILS: &str = include_str!("js_runtime/svg_group_utils.js");
const SVG_VIEWBOX_FIXES: &str = include_str!("js_runtime/svg_viewbox_fixes.js");
const SVG_ER_LAYOUT: &str = include_str!("js_runtime/svg_er_layout.js");
const SVG_ER_FIXES: &str = include_str!("js_runtime/svg_er_fixes.js");
const SVG_CLASS_FIXES: &str = include_str!("js_runtime/svg_class_fixes.js");
const SVG_VISUAL_FIXES: &str = include_str!("js_runtime/svg_visual_fixes.js");
const SVG_REVIEW_FIXES: &str = include_str!("js_runtime/svg_review_fixes.js");
const SVG_REVIEW_ISHIKAWA_FIXES: &str = include_str!("js_runtime/svg_review_ishikawa_fixes.js");
const SVG_REVIEW_VENN_FIXES: &str = include_str!("js_runtime/svg_review_venn_fixes.js");
const SVG_REVIEW_TREEMAP_FIXES: &str = include_str!("js_runtime/svg_review_treemap_fixes.js");
const SVG_KANBAN_TEXT: &str = include_str!("js_runtime/svg_kanban_text.js");
const SVG_KANBAN_VIEWBOX: &str = include_str!("js_runtime/svg_kanban_viewbox.js");
const SVG_KANBAN_LABEL_LAYOUT: &str = include_str!("js_runtime/svg_kanban_label_layout.js");
const SVG_KANBAN_FIXES: &str = include_str!("js_runtime/svg_kanban_fixes.js");
const SVG_WARDLEY_NORMALIZER: &str = include_str!("js_runtime/svg_wardley_normalizer.js");
const SVG_DIAGRAM_NORMALIZERS: &str = include_str!("js_runtime/svg_diagram_normalizers.js");
const SVG_DIMENSIONS: &str = include_str!("js_runtime/svg_dimensions.js");
const SOURCE_I18N_CONTEXT: &str = include_str!("js_runtime/source_i18n_context.js");
const SOURCE_I18N_NORMALIZE: &str = include_str!("js_runtime/source_i18n_normalize.js");
const SVG_NORMALIZE: &str = include_str!("js_runtime/svg_normalize.js");
const RENDER_MERMAID: &str = include_str!("js_runtime/render_mermaid.js");
