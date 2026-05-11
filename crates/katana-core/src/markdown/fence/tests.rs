use super::*;
use crate::markdown::diagram::{DiagramBlock, DiagramRenderer, DiagramResult, NoOpRenderer};

#[test]
fn extract_fence_block_no_fence_prefix() {
    assert!(MarkdownFenceOps::extract_fence_block("not a fence").is_none());
}

#[test]
fn extract_fence_block_no_newline_after_info() {
    assert!(MarkdownFenceOps::extract_fence_block("```mermaid").is_none());
}

#[test]
fn extract_fence_block_no_closing_fence() {
    assert!(MarkdownFenceOps::extract_fence_block("```mermaid\ngraph TD; A-->B").is_none());
}

#[test]
fn extract_fence_block_valid() {
    let result = MarkdownFenceOps::extract_fence_block("```mermaid\ngraph TD; A-->B\n```\nrest");
    assert!(result.is_some());
    let (block, rest) = result.unwrap();
    assert_eq!(block.info, "mermaid");
    assert_eq!(block.content, "graph TD; A-->B");
    assert_eq!(rest, "rest");
}

#[test]
fn extract_fence_block_accepts_crlf_closing_line() {
    let result =
        MarkdownFenceOps::extract_fence_block("```mermaid\r\ngraph TD; A-->B\r\n```\r\nrest");
    assert!(result.is_some());
    let (block, rest) = result.unwrap();
    assert_eq!(block.info, "mermaid");
    assert!(block.content.contains("graph TD; A-->B"));
    assert_eq!(rest, "rest");
}

#[test]
fn transform_handles_fence_at_start_of_input() {
    let source = "```mermaid\ngraph TD; A-->B\n```\nAfter";
    let result = MarkdownFenceOps::transform_diagram_blocks(source, &NoOpRenderer);
    assert!(result.contains("After"));
}

struct PngTestRenderer;
impl DiagramRenderer for PngTestRenderer {
    fn render(&self, _block: &DiagramBlock) -> DiagramResult {
        DiagramResult::OkPng(vec![0x89, 0x50, 0x4E, 0x47])
    }
}

struct ErrorTestRenderer;
impl DiagramRenderer for ErrorTestRenderer {
    fn render(&self, block: &DiagramBlock) -> DiagramResult {
        DiagramResult::Err {
            source: block.source.clone(),
            error: "render failed".to_string(),
        }
    }
}

struct ZenumlDependencyMissingRenderer;
impl DiagramRenderer for ZenumlDependencyMissingRenderer {
    fn render(&self, block: &DiagramBlock) -> DiagramResult {
        DiagramResult::Err {
            source: block.source.clone(),
            error:
                "Failed to start ZenUML browser renderer: No such file or directory (os error 2)"
                    .to_string(),
        }
    }
}

struct CommandMissingTestRenderer;
impl DiagramRenderer for CommandMissingTestRenderer {
    fn render(&self, block: &DiagramBlock) -> DiagramResult {
        DiagramResult::CommandNotFound {
            tool_name: "tool".to_string(),
            install_hint: "install it".to_string(),
            source: block.source.clone(),
        }
    }
}

struct PanicRenderer;
impl DiagramRenderer for PanicRenderer {
    fn render(&self, _block: &DiagramBlock) -> DiagramResult {
        panic!("Panic is raised when an empty diagram block is passed to the renderer");
    }
}

#[test]
fn render_diagram_block_okpng_embeds_base64_img() {
    let block = FenceBlock {
        info: "mermaid".to_string(),
        content: "graph TD; A-->B".to_string(),
        raw: "```mermaid\ngraph TD; A-->B\n```".to_string(),
    };
    let result = MarkdownFenceOps::render_diagram_block(&block, &PngTestRenderer);
    let html = result.expect("mermaid blocks should produce Some");
    assert!(html.contains("data:image/png;base64,"));
    assert!(html.contains("<img"));
}

#[test]
fn render_diagram_block_error_uses_fallback_html() {
    let block = FenceBlock {
        info: "mermaid".to_string(),
        content: "<bad>".to_string(),
        raw: "```mermaid\n<bad>\n```".to_string(),
    };

    let html = MarkdownFenceOps::render_diagram_block(&block, &ErrorTestRenderer)
        .expect("mermaid blocks should produce Some");

    assert!(html.contains("Diagram render failed"));
    assert!(html.contains("&lt;bad&gt;"));
}

#[test]
fn render_diagram_block_command_missing_uses_install_hint() {
    let block = FenceBlock {
        info: "mermaid".to_string(),
        content: "graph TD; A-->B".to_string(),
        raw: "```mermaid\ngraph TD; A-->B\n```".to_string(),
    };

    let html = MarkdownFenceOps::render_diagram_block(&block, &CommandMissingTestRenderer)
        .expect("mermaid blocks should produce Some");

    assert!(html.contains("tool not found. install it"));
}

#[test]
fn render_diagram_block_skips_empty_mermaid_source() {
    let block = FenceBlock {
        info: "mermaid".to_string(),
        content: "   \n\t".to_string(),
        raw: "```mermaid\n   \n\t\n```".to_string(),
    };

    assert!(MarkdownFenceOps::render_diagram_block(&block, &PanicRenderer).is_none());
}

#[test]
fn transform_keeps_empty_mermaid_fence_as_markdown() {
    let source = "before\n```mermaid\n   \n```\nafter";
    let result = MarkdownFenceOps::transform_diagram_blocks(source, &PanicRenderer);

    assert_eq!(result, source);
}

#[test]
fn transform_keeps_empty_tilde_mermaid_fence_as_markdown() {
    let source = "before\n~~~mermaid\n   \n~~~\nafter";
    let result = MarkdownFenceOps::transform_diagram_blocks(source, &PanicRenderer);

    assert_eq!(result, source);
}

#[test]
fn render_diagram_block_passes_zenuml_mermaid_source_to_renderer() {
    let block = FenceBlock {
        info: "mermaid".to_string(),
        content: "zenuml\n    title Order Service".to_string(),
        raw: "```mermaid\nzenuml\n    title Order Service\n```".to_string(),
    };

    let html = MarkdownFenceOps::render_diagram_block(&block, &PngTestRenderer)
        .expect("ZenUML Mermaid blocks should be rendered by the Mermaid renderer");
    assert!(html.contains("data:image/png;base64,"));
}

#[test]
fn transform_renders_zenuml_mermaid_fence() {
    let source = "before\n```mermaid\nzenuml\n    title Order Service\n```\nafter";
    let result = MarkdownFenceOps::transform_diagram_blocks(source, &PngTestRenderer);

    assert!(result.contains("before"));
    assert!(result.contains("after"));
    assert!(result.contains("data:image/png;base64,"));
    assert!(!result.contains("```mermaid"));
}

#[test]
fn transform_preserves_zenuml_mermaid_fence_when_dependency_is_missing() {
    let source = "before\n```mermaid\nzenuml\n    title Order Service\n```\nafter";
    let result =
        MarkdownFenceOps::transform_diagram_blocks(source, &ZenumlDependencyMissingRenderer);

    assert_eq!(result, source);
}

#[test]
fn transform_with_png_renderer_embeds_base64_in_output() {
    let source = "# Hello\n\n```mermaid\ngraph TD; A-->B\n```\n\nAfter";
    let result = MarkdownFenceOps::transform_diagram_blocks(source, &PngTestRenderer);
    assert!(result.contains("data:image/png;base64,"));
    assert!(result.contains("After"));
}

#[test]
fn transform_unrecognized_fence_remains_unchanged() {
    /* WHY: Info string "unknown_lang" maps to DiagramKind::Unknown, skipping it. */
    let source = "```unknown_lang\ncontent\n```";
    let result = MarkdownFenceOps::transform_diagram_blocks(source, &NoOpRenderer);
    assert_eq!(result, source);
}

#[test]
fn transform_handles_drawio_at_start() {
    let source = "<mxGraphModel><root></root></mxGraphModel>After";
    let result = MarkdownFenceOps::transform_diagram_blocks(source, &NoOpRenderer);
    /* WHY: Trigger logic for known diagram. Since NoOp doesn't add formatting that breaks parsing,
    it'll output wrapped blocks. We just care that After remains. */
    assert!(result.contains("After"));
}

#[test]
fn transform_handles_drawio_unclosed() {
    let source = "Before\n<mxGraphModel><root>";
    let result = MarkdownFenceOps::transform_diagram_blocks(source, &NoOpRenderer);
    /* WHY: Should append the start tag and move on because end tag is missing */
    assert_eq!(result, source);
}

#[test]
fn transform_handles_plantuml_at_start() {
    let source = "@startuml\nA->B\n@enduml After";
    let result = MarkdownFenceOps::transform_diagram_blocks(source, &NoOpRenderer);
    assert!(result.contains("After"));
}
