use std::path::PathBuf;

use super::*;
use crate::markdown::{DiagramResult, color_preset::DiagramColorPreset};

fn input_with_options(options: DiagramRenderOptions) -> DiagramBackendInput {
    DiagramBackendInput {
        language: DiagramBackendLanguage::Mermaid,
        source: "graph TD; A-->B".to_string(),
        options,
        theme: DiagramThemeSnapshot::from_preset("dark", true, DiagramColorPreset::dark()),
        document: DiagramDocumentContext::Detached {
            display_name: "scratch.md".to_string(),
        },
    }
}

#[test]
fn cache_key_changes_when_backend_id_changes() {
    let input = input_with_options(DiagramRenderOptions::default());
    let version = DiagramBackendVersion::new("1.0.0");
    let cli = DiagramBackendCacheKey::new(
        DiagramBackendId::new(DiagramBackendLanguage::Mermaid, "mmdc"),
        version.clone(),
        &input,
    );
    let native = DiagramBackendCacheKey::new(
        DiagramBackendId::new(DiagramBackendLanguage::Mermaid, "native"),
        version,
        &input,
    );

    assert_ne!(cli, native);
}

#[test]
fn cache_key_changes_when_backend_version_changes() {
    let input = input_with_options(DiagramRenderOptions::default());
    let old = DiagramBackendCacheKey::new(
        DiagramBackendId::new(DiagramBackendLanguage::Mermaid, "mmdc"),
        DiagramBackendVersion::new("1.0.0"),
        &input,
    );
    let new = DiagramBackendCacheKey::new(
        DiagramBackendId::new(DiagramBackendLanguage::Mermaid, "mmdc"),
        DiagramBackendVersion::new("2.0.0"),
        &input,
    );

    assert_ne!(old, new);
}

#[test]
fn cache_key_changes_when_render_options_change() {
    let fast = input_with_options(DiagramRenderOptions {
        timeout_millis: 1_000,
        ..DiagramRenderOptions::default()
    });
    let slow = input_with_options(DiagramRenderOptions {
        timeout_millis: 10_000,
        ..DiagramRenderOptions::default()
    });
    let backend_id = DiagramBackendId::new(DiagramBackendLanguage::Mermaid, "mmdc");
    let version = DiagramBackendVersion::new("1.0.0");

    assert_ne!(
        DiagramBackendCacheKey::new(backend_id.clone(), version.clone(), &fast),
        DiagramBackendCacheKey::new(backend_id, version, &slow)
    );
}

#[test]
fn backend_output_converts_to_existing_diagram_result() {
    match DiagramBackendOutput::HtmlFragment("<svg></svg>".to_string()).into_diagram_result() {
        DiagramResult::Ok(html) => assert_eq!(html, "<svg></svg>"),
        other => panic!("unexpected result: {other:?}"),
    }
    match DiagramBackendOutput::Png(vec![1, 2, 3]).into_diagram_result() {
        DiagramResult::OkPng(bytes) => assert_eq!(bytes, vec![1, 2, 3]),
        other => panic!("unexpected result: {other:?}"),
    }
}

#[test]
fn backend_error_converts_to_existing_diagram_result() {
    match (DiagramBackendError::RenderFailed {
        message: "render failed".to_string(),
    })
    .into_diagram_result("graph TD; A-->B")
    {
        DiagramResult::Err { source, error } => {
            assert_eq!(source, "graph TD; A-->B");
            assert_eq!(error, "render failed");
        }
        other => panic!("unexpected result: {other:?}"),
    }

    match (DiagramBackendError::CommandNotFound {
        tool_name: "mmdc".to_string(),
        install_hint: "npm install".to_string(),
    })
    .into_diagram_result("graph TD; A-->B")
    {
        DiagramResult::CommandNotFound {
            tool_name,
            install_hint,
            source,
        } => {
            assert_eq!(tool_name, "mmdc");
            assert_eq!(install_hint, "npm install");
            assert_eq!(source, "graph TD; A-->B");
        }
        other => panic!("unexpected result: {other:?}"),
    }

    match (DiagramBackendError::NotInstalled {
        kind: "PlantUML".to_string(),
        download_url: "https://example.com/plantuml.jar".to_string(),
        install_path: PathBuf::from("plantuml.jar"),
    })
    .into_diagram_result("ignored")
    {
        DiagramResult::NotInstalled {
            kind,
            download_url,
            install_path,
        } => {
            assert_eq!(kind, "PlantUML");
            assert_eq!(download_url, "https://example.com/plantuml.jar");
            assert_eq!(install_path, PathBuf::from("plantuml.jar"));
        }
        other => panic!("unexpected result: {other:?}"),
    }
}
