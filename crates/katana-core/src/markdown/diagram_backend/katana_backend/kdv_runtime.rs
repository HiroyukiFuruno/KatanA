use super::super::{
    DiagramBackendError, DiagramBackendInput, DiagramBackendOutput, DiagramBackendRenderResult,
};
use crate::markdown::kdv_theme_adapter::KdvThemeAdapter;
use katana_document_viewer::{
    DiagramRenderEngine, DiagramRenderRequest, KrrDiagramRenderEngine, RenderedDiagram,
};
use katana_markdown_model::DiagramKind as KdvDiagramKind;

const RUNTIME_MISSING_MARKER: &str = "runtime is not installed:";

pub(super) fn render(
    kind: KdvDiagramKind,
    input: &DiagramBackendInput,
) -> DiagramBackendRenderResult {
    let output = render_diagram(kind.clone(), input)?;
    kdv_diagram_output(&kind, output)
}

pub(super) fn kdv_diagram_output(
    kind: &KdvDiagramKind,
    output: RenderedDiagram,
) -> DiagramBackendRenderResult {
    if output.svg.trim_start().starts_with("<svg") {
        return Ok(DiagramBackendOutput::HtmlFragment(output.svg));
    }
    if matches!(kind, KdvDiagramKind::PlantUml) {
        return Err(DiagramBackendError::NotInstalled {
            kind: diagram_kind_label(kind).to_string(),
            message: "KDV/KRR returned non-SVG PlantUML output.".to_string(),
        });
    }
    Err(DiagramBackendError::RenderFailed {
        message: "KDV diagram renderer returned non-SVG output".to_string(),
    })
}

fn render_diagram(
    kind: KdvDiagramKind,
    input: &DiagramBackendInput,
) -> Result<RenderedDiagram, DiagramBackendError> {
    let document_id = input.document.cache_id();
    let theme = KdvThemeAdapter::from_diagram_theme_for_kind(&input.theme, &kind);
    let error_kind = kind.clone();
    KrrDiagramRenderEngine
        .render(DiagramRenderRequest {
            node_id: &document_id,
            document_id: &document_id,
            kind,
            source: input.source.clone(),
            theme: &theme,
        })
        .map_err(|message| kdv_error_to_backend(&error_kind, message))
}

#[cfg(test)]
pub(super) fn kdv_render_request_theme(
    kind: KdvDiagramKind,
    input: &DiagramBackendInput,
) -> katana_document_viewer::KdvThemeSnapshot {
    let document_id = input.document.cache_id();
    let theme = KdvThemeAdapter::from_diagram_theme_for_kind(&input.theme, &kind);
    let _request = DiagramRenderRequest {
        node_id: &document_id,
        document_id: &document_id,
        kind,
        source: input.source.clone(),
        theme: &theme,
    };
    theme
}

fn kdv_error_to_backend(kind: &KdvDiagramKind, message: String) -> DiagramBackendError {
    if runtime_missing_path(&message).is_some() {
        return DiagramBackendError::NotInstalled {
            kind: diagram_kind_label(kind).to_string(),
            message,
        };
    }
    DiagramBackendError::RenderFailed { message }
}

fn runtime_missing_path(message: &str) -> Option<&str> {
    let (_, path) = message.split_once(RUNTIME_MISSING_MARKER)?;
    Some(path.trim())
}

fn diagram_kind_label(kind: &KdvDiagramKind) -> &'static str {
    match kind {
        KdvDiagramKind::Mermaid => "Mermaid",
        KdvDiagramKind::DrawIo => "Draw.io",
        KdvDiagramKind::PlantUml => "PlantUML",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_error_without_missing_marker_maps_to_render_failed() {
        assert!(matches!(
            kdv_error_to_backend(&KdvDiagramKind::Mermaid, "syntax error".to_string()),
            DiagramBackendError::RenderFailed { message } if message == "syntax error"
        ));
    }
}
