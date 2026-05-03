use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::mermaid_renderer;

#[test]
fn rust_managed_js_runtime_keeps_beta_diagram_geometry_visible() {
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        eprintln!("mermaid.min.js is not installed; skipping layout regression");
        return;
    }

    let tree_view = render_fixture("22-01-tree-view-simple.md");
    assert!(tag_with_class(&tree_view, "treeView-node-label").contains("fill="));
    assert!(tag_with_class(&tree_view, "treeView-node-line").contains("stroke="));

    let ishikawa = render_fixture("13-02-ishikawa-diagram-4-categories.md");
    let spine = tag_with_class(&ishikawa, "ishikawa-spine");
    let x1 = attr_number(spine, "x1");
    let x2 = attr_number(spine, "x2");
    assert!((x2 - x1).abs() > 100.0, "Ishikawa spine collapsed: {spine}");
    assert!(read_view_box(&ishikawa)[2] <= 500.0);
    assert!(attr_number(&ishikawa, "height") <= 480.0);
    assert!(!ishikawa.contains(r#"height="556""#));

    let treemap = render_fixture("23-01-treemap-flat.md");
    assert!(!treemap.contains("NaN"));
    assert!(treemap.contains("Mermaid"));
    assert!(treemap.contains("Rasterize"));

    let kanban = render_fixture("14-01-kanban-simple.md");
    assert!(!kanban.contains(r#"height="336""#));
    assert!(read_view_box(&kanban)[3] <= 130.0);
    assert!(kanban.contains(r#"height="39""#));
    assert!(!kanban.contains(r#"height="81.6""#));

    let sankey = render_fixture("20-01-sankey-simple.md");
    assert!(
        sankey.contains("<linearGradient"),
        "Sankey gradients must preserve SVG tag casing"
    );
    assert!(
        sankey.contains("mix-blend-mode: multiply"),
        "Sankey links must preserve browser SVG blend styling"
    );

    let gantt = render_fixture("11-01-gantt-chart-status-colors.md");
    assert!(gantt.contains("Mermaid renderer schedule"));
    let domain = tag_with_class(&gantt, "domain");
    assert!(
        !domain.contains("H-"),
        "Gantt date axis collapsed into negative width: {domain}"
    );
}

#[test]
fn rust_managed_js_runtime_keeps_core_diagram_labels_in_bounds() {
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        eprintln!("mermaid.min.js is not installed; skipping layout regression");
        return;
    }

    for fixture in [
        "03-02-class-diagram-inheritance.md",
        "05-01-er-diagram-simple.md",
        "19-01-requirement-diagram-single.md",
    ] {
        let svg = render_fixture(fixture);
        assert!(!svg.contains("NaN"), "{fixture} contains NaN");
        assert!(svg.contains("viewBox="), "{fixture} lost viewBox");
        let view_box = read_view_box(&svg);
        assert!(
            view_box[2] >= 300.0 || view_box[3] >= 300.0,
            "{fixture} viewBox is too small: {view_box:?}"
        );
    }
}

#[test]
fn rust_managed_js_runtime_keeps_review_feedback_visual_details() {
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        eprintln!("mermaid.min.js is not installed; skipping layout regression");
        return;
    }

    let er = render_fixture("05-01-er-diagram-simple.md");
    assert!(er.contains("row-rect-odd"));
    assert!(er.contains("row-rect-even"));
    assert!(er.contains(">string</tspan>"));
    assert!(er.contains(">path</tspan>"));
    assert!(
        er.matches(r#"class="label name" transform="translate(0, "#).count() >= 2
    );
    assert!(er.contains(r#"<text y="-10.1" style="" text-anchor="middle">"#));
    assert!(er.contains(r#"<text y="-10.1" style="" text-anchor="middle"><tspan class="text-outer-tspan row" x="0" y="-0.1em" dy="1.1em"><tspan font-style="normal" class="text-inner-tspan" font-weight="normal">DIAGRAM"#));
    assert!(
        index_of(&er, "row-rect-odd") < index_of(&er, "label attribute-type"),
        "ER row backgrounds must stay behind attribute labels"
    );

    let gantt = render_fixture("11-01-gantt-chart-status-colors.md");
    assert!(gantt.contains(r##"fill="#1e1e1e"></rect>"##));

    let venn = render_fixture("25-02-venn-diagram-3-sets-with-styles.md");
    assert!(venn.contains("AB"));
    assert!(venn.contains("BC"));
    assert!(venn.contains("AC"));
    assert!(venn.contains("ABC"));

    let class = render_fixture("03-02-class-diagram-inheritance.md");
    assert!(read_view_box(&class)[3] <= 410.0);
}

#[test]
fn rust_managed_js_runtime_aligns_er_attribute_rows() {
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        eprintln!("mermaid.min.js is not installed; skipping layout regression");
        return;
    }

    let er = render_source(
        "complex ER",
        "erDiagram\n    CUSTOMER ||--o{ ORDER : places\n    ORDER ||--|{ ORDER_ITEM : contains\n    PRODUCT ||--o{ ORDER_ITEM : includes\n    CUSTOMER {\n        string id\n        string name\n        string email\n    }\n    ORDER {\n        string id\n        date orderDate\n        string status\n    }\n    PRODUCT {\n        string id\n        string name\n        float price\n    }\n    ORDER_ITEM {\n        int quantity\n        float price\n    }",
    );

    assert!(er.contains(r#"class="label name" transform="translate(0, "#));
    assert!(er.contains(r#"class="label attribute-type" transform="translate("#));
    assert!(er.contains(r#"class="label attribute-name" transform="translate("#));
    assert!(er.contains(r#"class="label attribute-type" transform="translate("#));
    assert!(er.contains(r#"class="label attribute-name" transform="translate("#));
    assert!(er.contains(r#"class="label attribute-type" transform="translate("#));
    assert!(er.contains(r#"<tspan class="text-outer-tspan row" x="0" y="-0.1em" dy="1.1em">"#));
}

fn render_fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/mermaid_parts/en")
        .join(name);
    let markdown = std::fs::read_to_string(path).unwrap();
    let source = extract_mermaid_block(&markdown);
    render_source(name, &source)
}

fn render_source(name: &str, source: &str) -> String {
    let block = DiagramBlock {
        kind: DiagramKind::Mermaid,
        source: source.to_string(),
    };
    match mermaid_renderer::MermaidRenderOps::render_mermaid(&block) {
        DiagramResult::Ok(svg) => svg,
        other => panic!("{name} did not render: {other:?}"),
    }
}

fn extract_mermaid_block(markdown: &str) -> String {
    let mut lines = Vec::new();
    let mut in_block = false;
    for line in markdown.lines() {
        if matches!(line.trim(), "```mermaid" | "~~~mermaid") {
            in_block = true;
            continue;
        }
        if in_block && matches!(line.trim(), "```" | "~~~") {
            return lines.join("\n");
        }
        if in_block {
            lines.push(line);
        }
    }
    panic!("Mermaid block not found");
}

fn index_of(text: &str, needle: &str) -> usize {
    text.find(needle)
        .unwrap_or_else(|| panic!("{needle} not found"))
}

fn tag_with_class<'a>(svg: &'a str, class_name: &str) -> &'a str {
    svg.split('<')
        .find(|tag| tag.contains(&format!(r#"class="{class_name}""#)))
        .unwrap_or_else(|| panic!("{class_name} tag not found"))
}

fn attr_number(tag: &str, name: &str) -> f64 {
    let pattern = format!(r#"{name}=""#);
    let start = tag
        .find(&pattern)
        .unwrap_or_else(|| panic!("{name} not found"))
        + pattern.len();
    let rest = &tag[start..];
    let end = rest.find('"').unwrap();
    rest[..end].parse::<f64>().unwrap()
}

fn read_view_box(svg: &str) -> [f64; 4] {
    let start = svg.find(r#"viewBox=""#).unwrap() + r#"viewBox=""#.len();
    let rest = &svg[start..];
    let end = rest.find('"').unwrap();
    let values = rest[..end]
        .split_whitespace()
        .map(|it| it.parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    [values[0], values[1], values[2], values[3]]
}
