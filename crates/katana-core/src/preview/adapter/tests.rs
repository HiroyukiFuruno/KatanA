use std::any::type_name;
use std::path::PathBuf;

use super::*;

const SAMPLE_HEADING_START: usize = 0;
const SAMPLE_HEADING_END: usize = 8;
const SAMPLE_BLOCK_START: usize = 9;
const SAMPLE_BLOCK_END: usize = 21;
const SAMPLE_HEADING_LEVEL: u8 = 1;

struct ContractOnlyAdapter;

impl PreviewAdapter for ContractOnlyAdapter {
    fn render(&self, input: &PreviewInput) -> PreviewAdapterResult {
        Ok(PreviewOutput {
            html: format!("<p>{}</p>", input.source),
            metadata: sample_metadata(),
            diagnostics: Vec::new(),
        })
    }
}

#[test]
fn adapter_trait_is_implementable_with_contract_types_only() {
    let adapter = ContractOnlyAdapter;
    let input = sample_input();
    let output = adapter
        .render(&input)
        .expect("contract adapter should render");

    assert_eq!(output.html, "<p># Title</p>");
    assert_eq!(output.metadata.headings.len(), 1);
}

#[test]
fn metadata_keeps_heading_and_block_anchors() {
    let metadata = sample_metadata();

    assert_eq!(metadata.headings[0].id, "heading-title");
    assert_eq!(metadata.headings[0].level, SAMPLE_HEADING_LEVEL);
    assert_eq!(
        metadata.headings[0].source_range,
        PreviewSourceRange::new(SAMPLE_HEADING_START, SAMPLE_HEADING_END)
    );
    assert_eq!(metadata.blocks[0].kind, PreviewBlockKind::Paragraph);
}

#[test]
fn action_hook_carries_renderer_neutral_source_range() {
    let metadata = sample_metadata();

    let action = &metadata.actions[0];
    assert_eq!(action.id, "copy-code");
    assert_eq!(
        action.source_range,
        Some(PreviewSourceRange::new(
            SAMPLE_BLOCK_START,
            SAMPLE_BLOCK_END
        ))
    );
    assert!(matches!(
        action.action,
        PreviewAction::CopyCode { ref code } if code == "println!()"
    ));
}

#[test]
fn workspace_context_records_document_path() {
    let input = sample_input();

    match input.workspace {
        PreviewWorkspaceContext::WorkspaceFile {
            workspace_root,
            document_path,
        } => {
            assert_eq!(workspace_root, PathBuf::from("/workspace"));
            assert_eq!(document_path, PathBuf::from("/workspace/doc.md"));
        }
        PreviewWorkspaceContext::Detached { display_name } => {
            panic!("unexpected detached context: {display_name}");
        }
    }
}

#[test]
fn public_contract_names_do_not_expose_renderer_internals() {
    let type_names = [
        type_name::<PreviewInput>(),
        type_name::<PreviewOutput>(),
        type_name::<PreviewRenderMetadata>(),
        type_name::<PreviewAction>(),
        type_name::<PreviewAdapterError>(),
    ]
    .join(" ");

    for forbidden in ["egui", "comrak", "pulldown", "commonmark"] {
        assert!(!type_names.contains(forbidden), "{forbidden} leaked");
    }
}

fn sample_input() -> PreviewInput {
    PreviewInput {
        source: "# Title".to_string(),
        options: PreviewRenderOptions {
            enabled_extensions: vec![PreviewExtension::GfmTable, PreviewExtension::Diagram],
            ..PreviewRenderOptions::default()
        },
        theme: PreviewThemeSnapshot {
            name: "dark".to_string(),
            is_dark: true,
            background: "#111111".to_string(),
            text: "#f0f0f0".to_string(),
            link: "#66aaff".to_string(),
            code_background: "#222222".to_string(),
            border: "#333333".to_string(),
        },
        workspace: PreviewWorkspaceContext::WorkspaceFile {
            workspace_root: PathBuf::from("/workspace"),
            document_path: PathBuf::from("/workspace/doc.md"),
        },
    }
}

fn sample_metadata() -> PreviewRenderMetadata {
    PreviewRenderMetadata {
        headings: vec![PreviewHeadingAnchor {
            id: "heading-title".to_string(),
            level: SAMPLE_HEADING_LEVEL,
            title: "Title".to_string(),
            source_range: PreviewSourceRange::new(SAMPLE_HEADING_START, SAMPLE_HEADING_END),
            rendered_id: PreviewRenderedId::new("heading-1"),
        }],
        blocks: vec![PreviewBlockAnchor {
            id: "paragraph-1".to_string(),
            kind: PreviewBlockKind::Paragraph,
            source_range: PreviewSourceRange::new(SAMPLE_BLOCK_START, SAMPLE_BLOCK_END),
            rendered_id: PreviewRenderedId::new("block-1"),
        }],
        actions: vec![PreviewActionHook {
            id: "copy-code".to_string(),
            source_range: Some(PreviewSourceRange::new(
                SAMPLE_BLOCK_START,
                SAMPLE_BLOCK_END,
            )),
            action: PreviewAction::CopyCode {
                code: "println!()".to_string(),
            },
        }],
        attributes: Default::default(),
    }
}
