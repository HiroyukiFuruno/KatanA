use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::mermaid_renderer;
use katana_core::markdown::svg_rasterize::SvgRasterizeOps;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());
const MAX_VISIBLE_EDGE: u32 = 1200;

#[test]
#[ignore = "spike: run explicitly when evaluating Rust-managed Mermaid.js runtime"]
fn rust_managed_js_runtime_renders_representative_mermaid_svg() {
    let _guard = ENV_LOCK.lock().unwrap();
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        eprintln!("mermaid.min.js is not installed; skipping spike");
        return;
    }

    for case in representative_cases() {
        let svg = render_svg(case.source);
        assert!(svg.contains("<svg"), "{} did not return SVG", case.name);
        for label in case.labels {
            assert!(svg.contains(label), "{} lost label {label}", case.name);
        }
        let image = SvgRasterizeOps::rasterize_svg(&svg, 1.0)
            .unwrap_or_else(|err| panic!("{} SVG failed to rasterize: {err}", case.name));
        assert!(image.width > 0, "{} rasterized width is empty", case.name);
        assert!(image.height > 0, "{} rasterized height is empty", case.name);
    }
}

#[test]
#[ignore = "spike: run explicitly when evaluating the all-pattern Mermaid fixture"]
fn rust_managed_js_runtime_evaluates_all_fixture_mermaid_blocks() {
    let _guard = ENV_LOCK.lock().unwrap();
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        eprintln!("mermaid.min.js is not installed; skipping spike");
        return;
    }

    let fixture = include_str!("../../../assets/fixtures/sample_mermaid_all.md");
    let sources = extract_mermaid_blocks(fixture);
    assert!(
        sources.len() >= 20,
        "fixture block count = {}",
        sources.len()
    );

    let mut success_count = 0;
    let mut failures = Vec::new();
    for (index, source) in sources.iter().enumerate() {
        match try_render_svg(source) {
            Ok(svg) => match SvgRasterizeOps::rasterize_svg(&svg, 1.0) {
                Ok(image)
                    if image.width > 0
                        && image.height > 0
                        && image.width <= MAX_VISIBLE_EDGE
                        && image.height <= MAX_VISIBLE_EDGE
                        && expected_labels(index, &svg) =>
                {
                    success_count += 1;
                }
                Ok(image) => failures.push(format!(
                    "block {index}: invalid rasterized image or labels {}x{}",
                    image.width, image.height
                )),
                Err(err) => failures.push(format!("block {index}: rasterize failed: {err}")),
            },
            Err(error) => failures.push(format!("block {index}: render failed: {error}")),
        }
    }
    eprintln!(
        "all-pattern fixture result: {success_count}/{} blocks rendered",
        sources.len()
    );
    for failure in &failures {
        eprintln!("{failure}");
    }
    assert!(failures.is_empty(), "success_count = {success_count}");
}

struct MermaidCase {
    name: &'static str,
    source: &'static str,
    labels: &'static [&'static str],
}

fn representative_cases() -> Vec<MermaidCase> {
    vec![
        MermaidCase {
            name: "flowchart",
            source: "graph TD; A-->B",
            labels: &["A", "B"],
        },
        MermaidCase {
            name: "sequence",
            source: "sequenceDiagram\n  participant User\n  participant KatanA\n  User->>KatanA: Open",
            labels: &["User", "KatanA", "Open"],
        },
        MermaidCase {
            name: "class",
            source: "classDiagram\n  class PreviewPane\n  PreviewPane --> RenderedSection",
            labels: &["PreviewPane", "RenderedSection"],
        },
        MermaidCase {
            name: "state",
            source: "stateDiagram-v2\n  [*] --> Pending\n  Pending --> Image : success\n  Image --> [*]",
            labels: &["Pending", "Image", "success"],
        },
        MermaidCase {
            name: "gantt",
            source: "gantt\n  title KatanA Schedule\n  dateFormat YYYY-MM-DD\n  section Core\n  Rendering :done, 2026-01-01, 30d",
            labels: &["KatanA", "Rendering"],
        },
        MermaidCase {
            name: "pie",
            source: "pie title Rendering Engine Distribution\n  \"DrawIo\" : 1\n  \"Mermaid\" : 1",
            labels: &["DrawIo", "Mermaid"],
        },
    ]
}

fn render_svg(source: &str) -> String {
    let block = DiagramBlock {
        kind: DiagramKind::Mermaid,
        source: source.to_string(),
    };
    match mermaid_renderer::MermaidRenderOps::render_mermaid(&block) {
        DiagramResult::Ok(svg) => svg,
        other => panic!("Expected Rust-managed Mermaid SVG, got {other:?}"),
    }
}

fn try_render_svg(source: &str) -> Result<String, String> {
    let block = DiagramBlock {
        kind: DiagramKind::Mermaid,
        source: source.to_string(),
    };
    match mermaid_renderer::MermaidRenderOps::render_mermaid(&block) {
        DiagramResult::Ok(svg) => Ok(svg),
        other => Err(format!("{other:?}")),
    }
}

fn extract_mermaid_blocks(markdown: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut in_mermaid = false;
    for line in markdown.lines() {
        if matches!(line.trim(), "~~~mermaid" | "```mermaid") {
            in_mermaid = true;
            current.clear();
            continue;
        }
        if in_mermaid && matches!(line.trim(), "~~~" | "```") {
            in_mermaid = false;
            blocks.push(current.join("\n"));
            current.clear();
            continue;
        }
        if in_mermaid {
            current.push(line);
        }
    }
    blocks
}

fn expected_labels(index: usize, svg: &str) -> bool {
    let text = normalized_svg_text(svg);
    let labels: &[&str] = match index {
        7 => &["Rust-managed JS", "SVG rasterize", "Export runtime"],
        8 => &[
            "OS independent runtime",
            "Fast accurate rendering",
            "R1",
            "R2",
        ],
        9 => &["base", "feature", "rust-js"],
        11 => &["Mermaid", "Runtime", "DOM shim", "Rasterize"],
        12 => &[
            "Mermaid runtime adoption",
            "Preview path",
            "Fixture coverage",
        ],
        15 => &["Markdown", "Parser", "Preview"],
        16 => &["Markdown", "Parser", "Renderer"],
        18 => &["export runtime", "Rust-managed Mermaid", "OS Chrome"],
        19 => &["Markdown", "Renderer", "SVG cache"],
        21 => &["Root", "Runtime", "DOM shim"],
        22 => &["Diagram quality", "Text measurement", "ViewBox"],
        24 => &["Runtime cost", "Mermaid", "Cache"],
        25 => &["Renderer adoption", "Preview", "DOMShim"],
        _ => &[],
    };
    labels.iter().all(|label| text.contains(label))
}

fn normalized_svg_text(svg: &str) -> String {
    let without_tags = regex::Regex::new(r"<[^>]+>").unwrap().replace_all(svg, " ");
    without_tags
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
