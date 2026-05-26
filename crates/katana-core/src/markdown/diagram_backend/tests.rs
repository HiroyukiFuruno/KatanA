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

fn input_with_theme(theme: DiagramThemeSnapshot) -> DiagramBackendInput {
    DiagramBackendInput {
        theme,
        ..input_with_options(DiagramRenderOptions::default())
    }
}

#[test]
fn cache_key_changes_when_backend_id_changes() {
    let input = input_with_options(DiagramRenderOptions::default());
    let version = DiagramBackendVersion::new("1.0.0");
    let cli = DiagramBackendCacheKey::new(
        DiagramBackendId::new(DiagramBackendLanguage::Mermaid, "mermaid-js"),
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
        DiagramBackendId::new(DiagramBackendLanguage::Mermaid, "mermaid-js"),
        DiagramBackendVersion::new("1.0.0"),
        &input,
    );
    let new = DiagramBackendCacheKey::new(
        DiagramBackendId::new(DiagramBackendLanguage::Mermaid, "mermaid-js"),
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
    let backend_id = DiagramBackendId::new(DiagramBackendLanguage::Mermaid, "mermaid-js");
    let version = DiagramBackendVersion::new("1.0.0");

    assert_ne!(
        DiagramBackendCacheKey::new(backend_id.clone(), version.clone(), &fast),
        DiagramBackendCacheKey::new(backend_id, version, &slow)
    );
}

#[test]
fn cache_key_changes_when_theme_changes() {
    let dark = input_with_theme(DiagramThemeSnapshot::from_preset(
        "dark",
        true,
        DiagramColorPreset::dark(),
    ));
    let light = input_with_theme(DiagramThemeSnapshot::from_preset(
        "light",
        false,
        DiagramColorPreset::light(),
    ));
    let backend_id = DiagramBackendId::new(DiagramBackendLanguage::Mermaid, "krr-mermaid");
    let version = DiagramBackendVersion::new("runtime");

    assert_ne!(
        DiagramBackendCacheKey::new(backend_id.clone(), version.clone(), &dark),
        DiagramBackendCacheKey::new(backend_id, version, &light)
    );
}

#[test]
fn cache_key_changes_when_runtime_profile_changes() {
    let input = input_with_options(DiagramRenderOptions::default());
    let backend_id = DiagramBackendId::new(DiagramBackendLanguage::Mermaid, "krr-mermaid");
    let old = DiagramBackendVersion::from_krr(
        "0.3.3",
        "Mermaid",
        "11.10.0",
        "old-checksum",
        "katana-mermaid",
    );
    let new = DiagramBackendVersion::from_krr(
        "0.3.3",
        "Mermaid",
        "11.10.0",
        "new-checksum",
        "katana-mermaid",
    );

    assert_ne!(
        DiagramBackendCacheKey::new(backend_id.clone(), old, &input),
        DiagramBackendCacheKey::new(backend_id, new, &input)
    );
}

#[test]
fn cache_key_changes_when_kdv_or_krr_version_changes() {
    let input = input_with_options(DiagramRenderOptions::default());
    let backend_id = DiagramBackendId::new(DiagramBackendLanguage::Mermaid, "kdv-krr-mermaid");
    let old = DiagramBackendVersion::from_kdv_krr(
        "0.1.0",
        "0.3.3",
        "Mermaid",
        "11.10.0",
        "checksum",
        "katana-mermaid",
    );
    let new = DiagramBackendVersion::from_kdv_krr(
        "0.1.1",
        "0.3.4",
        "Mermaid",
        "11.10.0",
        "checksum",
        "katana-mermaid",
    );

    assert_ne!(
        DiagramBackendCacheKey::new(backend_id.clone(), old, &input),
        DiagramBackendCacheKey::new(backend_id, new, &input)
    );
}

#[test]
fn current_theme_snapshot_uses_ui_theme_override() {
    DiagramThemeSnapshot::set_current_override(DiagramThemeOverride {
        name: "custom-light".to_string(),
        is_dark: false,
        background: "#fff4c2".to_string(),
        text: "#332900".to_string(),
        preview_text: "#332900".to_string(),
    });

    let snapshot = DiagramThemeSnapshot::current();

    assert_eq!(snapshot.name, "custom-light");
    assert_eq!(snapshot.background, "#fff4c2");
    assert_eq!(snapshot.text, "#332900");
    assert_eq!(snapshot.preview_text, "#332900");

    DiagramThemeSnapshot::clear_current_override();
}

#[test]
fn workspace_file_document_cache_id_includes_root_and_path() {
    let context = DiagramDocumentContext::WorkspaceFile {
        workspace_root: PathBuf::from("/workspace"),
        document_path: PathBuf::from("docs/diagram.md"),
    };

    assert_eq!(context.cache_id(), "/workspace:docs/diagram.md");
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
        tool_name: "renderer".to_string(),
        install_hint: "install renderer".to_string(),
    })
    .into_diagram_result("graph TD; A-->B")
    {
        DiagramResult::CommandNotFound {
            tool_name,
            install_hint,
            source,
        } => {
            assert_eq!(tool_name, "renderer");
            assert_eq!(install_hint, "install renderer");
            assert_eq!(source, "graph TD; A-->B");
        }
        other => panic!("unexpected result: {other:?}"),
    }

    match (DiagramBackendError::NotInstalled {
        kind: "PlantUML".to_string(),
        message: "runtime unavailable".to_string(),
    })
    .into_diagram_result("ignored")
    {
        DiagramResult::NotInstalled { kind, message } => {
            assert_eq!(kind, "PlantUML");
            assert_eq!(message, "runtime unavailable");
        }
        other => panic!("unexpected result: {other:?}"),
    }
}
