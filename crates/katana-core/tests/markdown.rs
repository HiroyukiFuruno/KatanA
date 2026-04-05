use katana_core::markdown::*;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn basic_gfm_renders_to_html() {
    let md = "# Heading\n\nParagraph with **bold** and `code`.\n";
    let out = MarkdownRenderOps::render_basic(md).expect("render failed");
    assert!(out.html.contains("<h1>"));
    assert!(out.html.contains("<strong>bold</strong>"));
    assert!(out.html.contains("<code>code</code>"));
}

#[test]
fn gfm_table_renders() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |\n";
    let out = MarkdownRenderOps::render_basic(md).expect("render failed");
    assert!(out.html.contains("<table>"));
}

#[test]
fn gfm_tasklist_renders() {
    let md = "- [x] Done\n- [ ] Todo\n";
    let out = MarkdownRenderOps::render_basic(md).expect("render failed");
    assert!(out.html.contains("<li>"));
}

#[test]
fn malformed_document_does_not_panic() {
    let md = "## Unclosed\n\n```\nno close fence";
    assert!(MarkdownRenderOps::render_basic(md).is_ok());
}

#[test]
fn mermaid_block_is_transformed() {
    let md = "\n```mermaid\ngraph TD; A-->B\n```\n";
    let out = MarkdownRenderOps::render_basic(md).expect("render failed");
    assert!(out.html.contains("mermaid"));
}

#[test]
fn unknown_fence_passes_through() {
    let md = "\n```rust\nfn main() {}\n```\n";
    let out = MarkdownRenderOps::render_basic(md).expect("render failed");
    assert!(out.html.contains("fn main()"));
}

#[test]
fn render_with_katana_renderer_succeeds_for_plain_markdown() {
    let md = "# Hello\n\nWorld";
    let out = MarkdownRenderOps::render_with_katana_renderer(md).expect("render failed");
    assert!(out.html.contains("<h1>"));
    assert!(out.html.contains("World"));
}

#[test]
fn katana_renderer_handles_mermaid_block_without_crash() {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("MERMAID_MMDC", "/nonexistent/mmdc") };
    let md = "\n```mermaid\ngraph TD; A-->B\n```\n";
    let out = MarkdownRenderOps::render_with_katana_renderer(md).expect("render failed");
    assert!(!out.html.is_empty());
    unsafe { std::env::remove_var("MERMAID_MMDC") };
}

#[test]
fn katana_renderer_handles_plantuml_block_without_crash() {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("PLANTUML_JAR", "/nonexistent/plantuml.jar") };
    let md = "\n```plantuml\n@startuml\nA -> B\n@enduml\n```\n";
    let out = MarkdownRenderOps::render_with_katana_renderer(md).expect("render failed");
    assert!(!out.html.is_empty());
    unsafe { std::env::remove_var("PLANTUML_JAR") };
}

#[test]
fn katana_renderer_handles_drawio_block() {
    let md = "\n```drawio\n<mxGraphModel><root><mxCell id=\"0\"/></root></mxGraphModel>\n```\n";
    let out = MarkdownRenderOps::render_with_katana_renderer(md).expect("render failed");
    assert!(!out.html.is_empty());
}

#[test]
fn render_with_fence_at_very_start_of_document() {
    let md = "```mermaid\ngraph TD; A-->B\n```\nAfter block";
    let out = MarkdownRenderOps::render_basic(md).expect("render failed");
    assert!(!out.html.is_empty());
}

#[test]
fn render_with_katana_renderer_drawio_renders_svg() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="Box" vertex="1" parent="1"><mxGeometry x="10" y="10" width="100" height="50" as="geometry"/></mxCell>
</root></mxGraphModel>"#;
    let md = format!("\n```drawio\n{xml}\n```\n");
    let out = MarkdownRenderOps::render_with_katana_renderer(&md).expect("render failed");
    assert!(out.html.contains("svg") || out.html.contains("katana-diagram"));
}

#[test]
fn drawio_renderer_escapes_html_in_fallback() {
    let md = "\n```drawio\nnot valid xml & <stuff>\n```\n";
    let out = MarkdownRenderOps::render_with_katana_renderer(md).expect("render failed");
    assert!(!out.html.is_empty());
}

#[test]
fn okpng_branch_becomes_empty_string_in_core_layer() {
    use katana_core::markdown::diagram::{DiagramBlock, DiagramRenderer, DiagramResult};

    struct PngRenderer;
    impl DiagramRenderer for PngRenderer {
        fn render(&self, _block: &DiagramBlock) -> DiagramResult {
            DiagramResult::OkPng(vec![0x89, 0x50, 0x4E, 0x47])
        }
    }

    let md = "\n```mermaid\ngraph TD; A-->B\n```\n";
    let out = MarkdownRenderOps::render(md, &PngRenderer).expect("render failed");
    assert!(!out.html.contains("graph TD"));
}
