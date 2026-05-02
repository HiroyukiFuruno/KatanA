use katana_core::markdown::mermaid_renderer;
use katana_core::markdown::{
    color_preset::DiagramColorPreset,
    diagram::{DiagramBlock, DiagramKind, DiagramResult},
};
use regex::Regex;
use std::sync::Mutex;

static THEME_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn rust_managed_js_runtime_keeps_mermaid_md_feedback_diagrams_readable() {
    let _theme_guard = THEME_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let _dark_mode_guard = DarkModeGuard::set(true);
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        eprintln!("mermaid.min.js is not installed; skipping layout regression");
        return;
    }

    let flowchart = render_named_block("1. Flowchart");
    let iphone_width = flowchart_node_width(&flowchart, "E");
    assert!(
        (iphone_width - 107.53125).abs() < 0.2,
        "flowchart iPhone node width drifted from browser text measurement: {iphone_width}"
    );

    let er = render_named_block("5. Entity Relationship Diagram");
    assert!(er.contains(r#"class="label name" transform="translate(0,"#));
    assert!(!er.contains(r#"class="label name" transform="translate(-"#));
    let single_node_er = render_all_fixture("05-01-er-diagram-simple.md");
    assert!(
        regex::Regex::new(
            r#"<g class="label" style="" transform="translate\(0, -9\.5\)">[\s\S]*?<text y="-10\.1" style="" text-anchor="middle">[\s\S]*?>DIAGRAM"#
        )
        .unwrap()
        .is_match(&single_node_er)
    );
    assert!(
        !single_node_er.contains(r#"<text y="-10.1" style="" transform="translate(-32.3125, 0)">"#)
    );

    let state = render_named_block("6. State Diagram");
    assert!(state.contains(r#"class="label" style="" transform="translate(0, -9.5)""#));
    assert!(state.contains(r#"<text y="-10.1" style="" text-anchor="middle">"#));
    let state_view_box = read_view_box(&state);
    assert!(
        state_view_box[1] >= 0.0,
        "state diagram viewBox keeps unexpected top padding: {state_view_box:?}"
    );

    let mindmap = render_named_block("7. Mindmap");
    let mindmap_view_box = read_view_box(&mindmap);
    assert!(
        mindmap_view_box[2] <= 1400.0 && mindmap_view_box[3] <= 900.0,
        "mindmap viewBox is too sparse: {mindmap_view_box:?}"
    );

    let block = render_named_block("10. Block Diagram");
    assert!(!block.contains("&nbsp;"));
    assert!(!block.contains("&amp;nbsp;"));
    let block_view_box = read_view_box(&block);
    assert!(
        block_view_box[2] > 300.0,
        "block diagram width collapsed: {block_view_box:?}"
    );

    let gitgraph = render_named_block("12. Git Graph");
    let gitgraph_view_box = read_view_box(&gitgraph);
    assert!(
        gitgraph_view_box[3] <= 280.0,
        "gitgraph viewBox kept excessive bottom padding: {gitgraph_view_box:?}"
    );
    assert!(
        (gitgraph_commit_label_background_height(&gitgraph) - 15.0).abs() < 0.1,
        "gitgraph commit labels should use class-based 10px text measurement"
    );

    let ishikawa = render_all_fixture("13-02-ishikawa-diagram-4-categories.md");
    assert!(ishikawa.contains(">Blurry"));
    assert!(ishikawa.contains(">Photo"));
    assert!(!ishikawa.contains(r#"Q 148.79999999999998 0 0 -39.2 Z"#));
    assert!(
        (ishikawa_head_half_height(&ishikawa) - 52.8).abs() < 0.1,
        "standard ishikawa head height should match the two-line Mermaid layout"
    );
    assert!(ishikawa_head_width(&ishikawa) >= 144.0);
    assert!(
        ishikawa_bottom_category_box_fits(&ishikawa),
        "ishikawa bottom category boxes are clipped"
    );
    let wide_ishikawa = render_source(
        "wide ishikawa",
        "ishikawa-beta\n    Blurry Photoabcdefghijk\n    User\n        Shaky hands",
    );
    assert!(ishikawa_head_width(&wide_ishikawa) > ishikawa_head_width(&ishikawa));

    let kanban = render_all_fixture("14-02-kanban-full.md");
    let kanban_heights = kanban_section_heights(&kanban);
    let long_card = kanban_card_metrics(&kanban, "id6");
    let short_card = kanban_card_metrics(&kanban, "id5");
    assert!(
        kanban_heights.iter().any(|height| *height > 150.0),
        "kanban sections do not grow for wrapped cards: {kanban_heights:?}"
    );
    assert!(
        long_card.0 > 140.0 && short_card.0 <= 45.0,
        "kanban card heights are not content-based: long={long_card:?}, short={short_card:?}"
    );
    assert!(
        kanban_label_is_top_aligned(long_card) && kanban_label_is_top_aligned(short_card),
        "kanban card labels are not top-aligned: long={long_card:?}, short={short_card:?}"
    );

    let pie = render_named_block("16. Pie Chart");
    let pie_view_box = read_view_box(&pie);
    assert!(
        pie_view_box[2] >= 620.0,
        "pie viewBox should include the legend with browser-equivalent padding: {pie_view_box:?}"
    );

    let venn = render_named_block("25. Venn Diagram");
    assert!(!venn.contains(".venn-set-0 path{fill:rgb(122,122,122)"));
    eprintln!("venn svg: {venn}");
    assert!(
        ["skyblue", "orange", "lightgreen", "white"]
            .iter()
            .all(|color| venn.contains(color))
    );
    assert!(venn.contains("fill: rgb(122, 122, 122); stroke: rgb(122, 122, 122);"));
    assert!(venn.contains("fill: rgb(164, 0, 0); stroke: rgb(164, 0, 0);"));
    assert!(venn.contains("fill: rgb(204, 42, 145); stroke: rgb(204, 42, 145);"));

    let wardley = render_named_block("26. Wardley Map");
    assert!(
        wardley.contains(r##"class="wardley-background""##)
            && wardley_background_fill(&wardley).as_deref() == Some("#333333")
    );

    let xychart = render_all_fixture("27-02-xy-chart-bar-line.md");
    assert!(xychart.contains(r#"transform="translate(350, 21.5) rotate(0)">Sales Revenue</text>"#));
    assert!(xychart.contains(r#"<path d="M 81.710625,43 L 81.710625,467 " fill="none""#));
    assert!(xychart.contains(r#"transform="translate(70.710625, 51) rotate(0)">11000</text>"#));
    assert!(xychart.contains(r#"transform="translate(5, 255) rotate(270)">Revenue (in $)</text>"#));
}

#[test]
fn rust_managed_js_runtime_keeps_light_theme_feedback_diagrams_readable() {
    let _theme_guard = THEME_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        eprintln!("mermaid.min.js is not installed; skipping light theme regression");
        return;
    }

    let _guard = DarkModeGuard::set(false);
    let kanban = render_all_fixture("14-01-kanban-simple.md");
    assert!(kanban.contains(".cluster-label text"));
    assert!(kanban.contains(".cluster-label tspan"));
    assert!(kanban.contains(".label tspan"));
    assert!(kanban.contains(".kanban-ticket-link tspan"));
    assert!(kanban.contains("fill:#333333!important"));

    let wardley = render_named_block("26. Wardley Map");
    assert!(wardley.contains(r##"class="wardley-map""##));
    assert!(wardley_background_fill(&wardley).as_deref() == Some("transparent"));
}

struct DarkModeGuard {
    previous: bool,
}

impl DarkModeGuard {
    fn set(is_dark: bool) -> Self {
        let previous = DiagramColorPreset::is_dark_mode();
        DiagramColorPreset::set_dark_mode(is_dark);
        Self { previous }
    }
}

impl Drop for DarkModeGuard {
    fn drop(&mut self) {
        DiagramColorPreset::set_dark_mode(self.previous);
    }
}

fn render_named_block(heading: &str) -> String {
    render_source(heading, &source_after_heading(heading))
}

fn render_all_fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/mermaid_parts/en")
        .join(name);
    render_source(
        name,
        &extract_mermaid_block(&std::fs::read_to_string(path).unwrap()),
    )
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

fn source_after_heading(heading: &str) -> String {
    let marker = format!("## {heading}");
    let markdown = include_str!("../../../assets/fixtures/sample_mermaid.md");
    let section = markdown
        .split(&marker)
        .nth(1)
        .unwrap_or_else(|| panic!("{heading} section not found"));
    extract_mermaid_block(section)
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

fn kanban_section_heights(svg: &str) -> Vec<f64> {
    let section_pattern = regex::Regex::new(
        r#"<rect style="" rx="5" ry="5" x="[^"]+" y="[^"]+" width="200" height="([^"]+)">"#,
    )
    .unwrap();
    section_pattern
        .captures_iter(svg)
        .map(|capture| capture[1].parse::<f64>().unwrap())
        .collect()
}

fn kanban_card_metrics(svg: &str, id: &str) -> (f64, f64) {
    let pattern = regex::Regex::new(&format!(
        r#"id="katana-mermaid-svg-[0-9a-f]{{16}}(?:-[0-9a-f]{{16}})?-{id}"[\s\S]*?<rect class="basic label-container[^"]*"[^>]*\sheight="([^"]+)"[\s\S]*?<g class="label" style="text-align:left !important" transform="translate\([^,]+, ([^)]+)\)""#
    ))
    .unwrap();
    let captures = pattern
        .captures(svg)
        .unwrap_or_else(|| panic!("{id} kanban card not found"));
    (captures[1].parse().unwrap(), captures[2].parse().unwrap())
}

fn kanban_label_is_top_aligned(metrics: (f64, f64)) -> bool {
    (metrics.1 - (-metrics.0 / 2.0 + 10.0)).abs() < 0.1
}

fn ishikawa_head_width(svg: &str) -> f64 {
    let pattern = regex::Regex::new(r#"class="ishikawa-head" d="M 0 -[^"]+ Q ([\d.]+) 0"#).unwrap();
    pattern.captures(svg).unwrap()[1].parse::<f64>().unwrap()
}

fn ishikawa_head_half_height(svg: &str) -> f64 {
    let pattern = regex::Regex::new(r#"class="ishikawa-head" d="M 0 -([\d.]+) L 0"#).unwrap();
    pattern.captures(svg).unwrap()[1].parse::<f64>().unwrap()
}

fn ishikawa_bottom_category_box_fits(svg: &str) -> bool {
    let view_box = read_view_box(svg);
    let view_box_bottom = view_box[1] + view_box[3];
    let label_box_pattern = regex::Regex::new(
        r#"<rect class="ishikawa-label-box" x="[^"]+" y="([^"]+)" width="[^"]+" height="([^"]+)""#,
    )
    .unwrap();
    label_box_pattern
        .captures_iter(svg)
        .map(|capture| capture[1].parse::<f64>().unwrap() + capture[2].parse::<f64>().unwrap())
        .all(|bottom| bottom <= view_box_bottom)
}

fn flowchart_node_width(svg: &str, node_id: &str) -> f64 {
    let pattern = regex::Regex::new(&format!(
        r#"id="katana-mermaid-svg-[0-9a-f]{{16}}(?:-[0-9a-f]{{16}})?-flowchart-{node_id}-\d+"[\s\S]*?<rect class="basic label-container"[^>]*\swidth="([^"]+)""#
    ))
    .unwrap();
    pattern.captures(svg).unwrap()[1].parse::<f64>().unwrap()
}

fn gitgraph_commit_label_background_height(svg: &str) -> f64 {
    let pattern =
        regex::Regex::new(r#"<rect class="commit-label-bkg"[^>]*\sheight="([^"]+)""#).unwrap();
    pattern.captures(svg).unwrap()[1].parse::<f64>().unwrap()
}

fn wardley_background_fill(svg: &str) -> Option<String> {
    let pattern = Regex::new(r#"<rect class=\"wardley-background\"[^>]* fill=\"([^\"]+)\""#)
        .unwrap();
    pattern
        .captures(svg)
        .and_then(|capture| capture.get(1).map(|it| it.as_str().to_string()))
}
