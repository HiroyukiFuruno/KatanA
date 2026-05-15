use crate::markdown::DiagramResult;
use crate::markdown::diagram_backend::result::{
    DiagramBackendError, DiagramBackendOutput, DiagramBackendRenderResult,
};
use katana_diagram_renderer::RenderError;

pub(super) fn diagram_result_to_backend(result: DiagramResult) -> DiagramBackendRenderResult {
    match result {
        DiagramResult::Ok(html) => Ok(DiagramBackendOutput::HtmlFragment(html)),
        DiagramResult::OkPng(bytes) => Ok(DiagramBackendOutput::Png(bytes)),
        DiagramResult::Err { error, .. } => {
            Err(DiagramBackendError::RenderFailed { message: error })
        }
        DiagramResult::CommandNotFound {
            tool_name,
            install_hint,
            ..
        } => Err(DiagramBackendError::CommandNotFound {
            tool_name,
            install_hint,
        }),
        DiagramResult::NotInstalled {
            kind,
            download_url,
            install_path,
        } => Err(DiagramBackendError::NotInstalled {
            kind,
            download_url,
            install_path,
        }),
    }
}

pub(super) fn kdr_error_to_backend(error: RenderError) -> DiagramBackendError {
    match error {
        RenderError::NotInstalled {
            kind,
            download_url,
            install_path,
        } => DiagramBackendError::NotInstalled {
            kind,
            download_url,
            install_path,
        },
        RenderError::InvalidInput(message)
        | RenderError::Runtime(message)
        | RenderError::RuntimeResolution(message) => DiagramBackendError::RenderFailed { message },
        RenderError::UnsupportedKind => DiagramBackendError::RenderFailed {
            message: "unsupported diagram kind".to_string(),
        },
    }
}
