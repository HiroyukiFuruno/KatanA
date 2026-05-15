use super::super::{diagram_result_to_backend, kdr_error_to_backend};
use crate::markdown::DiagramResult;
use crate::markdown::diagram_backend::result::{DiagramBackendError, DiagramBackendOutput};
use katana_diagram_renderer::RenderError;

#[test]
fn ok_png_result_maps_to_png_output() {
    let bytes = vec![1u8, 2, 3];
    let result = diagram_result_to_backend(DiagramResult::OkPng(bytes.clone()));
    assert_eq!(result, Ok(DiagramBackendOutput::Png(bytes)));
}

#[test]
fn ok_html_result_maps_to_html_fragment_output() {
    let result = diagram_result_to_backend(DiagramResult::Ok("<svg></svg>".to_string()));
    assert_eq!(
        result,
        Ok(DiagramBackendOutput::HtmlFragment(
            "<svg></svg>".to_string()
        ))
    );
}

#[test]
fn err_result_maps_to_render_failed() {
    let result = diagram_result_to_backend(DiagramResult::Err {
        source: "src".to_string(),
        error: "oops".to_string(),
    });
    assert_eq!(
        result,
        Err(DiagramBackendError::RenderFailed {
            message: "oops".to_string()
        })
    );
}

#[test]
fn command_not_found_result_maps_to_backend_command_not_found() {
    let result = diagram_result_to_backend(DiagramResult::CommandNotFound {
        tool_name: "plantuml".to_string(),
        install_hint: "brew install plantuml".to_string(),
        source: "src".to_string(),
    });
    assert_eq!(
        result,
        Err(DiagramBackendError::CommandNotFound {
            tool_name: "plantuml".to_string(),
            install_hint: "brew install plantuml".to_string(),
        })
    );
}

#[test]
fn not_installed_result_maps_to_backend_not_installed() {
    let result = diagram_result_to_backend(DiagramResult::NotInstalled {
        kind: "PlantUML".to_string(),
        download_url: "https://example.com/plantuml.jar".to_string(),
        install_path: std::path::PathBuf::from("plantuml.jar"),
    });
    assert_eq!(
        result,
        Err(DiagramBackendError::NotInstalled {
            kind: "PlantUML".to_string(),
            download_url: "https://example.com/plantuml.jar".to_string(),
            install_path: std::path::PathBuf::from("plantuml.jar"),
        })
    );
}

#[test]
fn kdr_not_installed_error_maps_to_backend_not_installed() {
    let result = kdr_error_to_backend(RenderError::NotInstalled {
        kind: "Draw.io".to_string(),
        download_url: "https://example.com/drawio.zip".to_string(),
        install_path: std::path::PathBuf::from("drawio.zip"),
    });
    assert_eq!(
        result,
        DiagramBackendError::NotInstalled {
            kind: "Draw.io".to_string(),
            download_url: "https://example.com/drawio.zip".to_string(),
            install_path: std::path::PathBuf::from("drawio.zip"),
        }
    );
}

#[test]
fn kdr_invalid_input_error_maps_to_render_failed() {
    let result = kdr_error_to_backend(RenderError::InvalidInput("bad diagram".to_string()));
    assert_eq!(
        result,
        DiagramBackendError::RenderFailed {
            message: "bad diagram".to_string(),
        }
    );
}

#[test]
fn kdr_runtime_error_maps_to_render_failed() {
    let result = kdr_error_to_backend(RenderError::Runtime("runtime failed".to_string()));
    assert_eq!(
        result,
        DiagramBackendError::RenderFailed {
            message: "runtime failed".to_string(),
        }
    );
}

#[test]
fn kdr_runtime_resolution_error_maps_to_render_failed() {
    let result = kdr_error_to_backend(RenderError::RuntimeResolution(
        "runtime missing".to_string(),
    ));
    assert_eq!(
        result,
        DiagramBackendError::RenderFailed {
            message: "runtime missing".to_string(),
        }
    );
}

#[test]
fn kdr_unsupported_kind_error_maps_to_render_failed() {
    let result = kdr_error_to_backend(RenderError::UnsupportedKind);
    assert_eq!(
        result,
        DiagramBackendError::RenderFailed {
            message: "unsupported diagram kind".to_string(),
        }
    );
}
