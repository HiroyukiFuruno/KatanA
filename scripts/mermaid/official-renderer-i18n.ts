import os from "node:os";
import path from "node:path";

const I18N_RUNTIME_DIR = "crates/katana-core/src/markdown/mermaid_renderer/js_runtime";
const RUNTIME_SCRIPTS = [
  "browser_globals.js",
  "dom_node.js",
  "dom_style.js",
  "dom_html.js",
  "dom_node_html.js",
  "dom_selector.js",
  "dom_document.js",
  "dom_tree.js",
  "dom_layout.js",
  "dom_metrics.js",
  "dom_layout_prototype.js",
  "dom_css_style.js",
  "dom_computed_style.js",
  "dom_measure.js",
  "dom_measure_text_lines.js",
  "dom_text_metrics.js",
  "dom_text_measure.js",
  "svg_bounds_attrs.js",
  "svg_path_bounds.js",
  "svg_bounds.js",
  "svg_text_bounds.js",
  "svg_architecture_icons.js",
  "svg_group_utils.js",
  "svg_viewbox_fixes.js",
  "svg_er_layout.js",
  "svg_er_fixes.js",
  "svg_class_fixes.js",
  "svg_visual_fixes.js",
  "svg_review_fixes.js",
  "svg_review_ishikawa_fixes.js",
  "svg_review_venn_fixes.js",
  "svg_review_treemap_fixes.js",
  "svg_kanban_text.js",
  "svg_kanban_viewbox.js",
  "svg_kanban_label_layout.js",
  "svg_kanban_fixes.js",
  "svg_wardley_normalizer.js",
  "svg_diagram_normalizers.js",
  "svg_dimensions.js",
  "source_i18n_context.js",
  "source_i18n_normalize.js",
  "svg_normalize.js",
];

export class MermaidRuntimeScripts {
  static paths(): string[] {
    return RUNTIME_SCRIPTS.map((filename) => path.resolve(I18N_RUNTIME_DIR, filename));
  }
}

export function expandHome(value: string) {
  return value.startsWith("~/") ? path.join(os.homedir(), value.slice(2)) : value;
}
