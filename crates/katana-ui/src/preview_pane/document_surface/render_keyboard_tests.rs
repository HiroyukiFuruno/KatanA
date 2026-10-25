use super::navigation_command;
use eframe::egui;
use katana_document_viewer::{
    DocumentFrame, DocumentSurfaceFrame, DocumentViewerCommand, DocumentViewerState,
    PdfRenderedPage, ViewerCapabilities, ViewerDocumentFormat, ViewerImageSurface,
};

fn frame(format: ViewerDocumentFormat, active_index: usize, item_count: usize) -> DocumentFrame {
    let surface = DocumentSurfaceFrame::from_rendered_page(
        "keyboard navigation test",
        PdfRenderedPage {
            page_index: active_index,
            scale: 1.0,
            surface: ViewerImageSurface {
                fingerprint: format!("keyboard-page-{active_index}"),
                width: 1,
                height: 1,
                display_width: 1.0,
                display_height: 1.0,
                content_scale: 100,
                rgba: vec![0, 0, 0, 255],
            },
        },
    )
    .expect("test page surface");
    DocumentFrame {
        surface,
        state: DocumentViewerState {
            active_index,
            ..DocumentViewerState::new(item_count)
        },
        capabilities: ViewerCapabilities::static_page(),
        diagnostics: Vec::new(),
        format,
    }
}

fn dispatch_key(
    key: egui::Key,
    modifiers: egui::Modifiers,
    focused_widget: Option<&str>,
    frame: &DocumentFrame,
) -> Option<DocumentViewerCommand> {
    let ctx = egui::Context::default();
    let doc_id = egui::Id::new("document-page-navigation-test");
    if let Some(widget) = focused_widget {
        let focus_id = egui::Id::new(widget);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.interact(
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(100.0, 100.0)),
                focus_id,
                egui::Sense::focusable_noninteractive(),
            )
            .request_focus();
        });
        output.textures_delta.clear();
    } else {
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.interact(
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(100.0, 100.0)),
                doc_id,
                egui::Sense::focusable_noninteractive(),
            )
            .request_focus();
        });
        output.textures_delta.clear();
    }

    let mut command = None;
    let mut events = vec![egui::Event::ModifiersChanged(modifiers)];
    events.push(egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    });
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(100.0, 100.0),
        )),
        events,
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ui| {
        let response = ui.interact(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(100.0, 100.0)),
            doc_id,
            egui::Sense::focusable_noninteractive(),
        );
        command = navigation_command(ui, &response, frame);
    });
    output.textures_delta.clear();
    command
}

#[test]
fn focused_page_arrow_events_dispatch_viewer_commands_for_pdf_and_pptx() {
    assert_eq!(
        dispatch_key(
            egui::Key::ArrowRight,
            egui::Modifiers::default(),
            None,
            &frame(ViewerDocumentFormat::Pdf, 0, 3)
        ),
        Some(DocumentViewerCommand::Next)
    );
    assert_eq!(
        dispatch_key(
            egui::Key::ArrowLeft,
            egui::Modifiers::default(),
            None,
            &frame(ViewerDocumentFormat::Pptx, 1, 3)
        ),
        Some(DocumentViewerCommand::Previous)
    );
}

#[test]
fn page_navigation_ignores_unfocused_widgets_other_formats_modifiers_and_boundaries() {
    let pdf = frame(ViewerDocumentFormat::Pdf, 0, 3);
    for focused_widget in [
        "markdown-editor",
        "document-search",
        "toolbar-popup",
        "explorer-tree",
    ] {
        assert_eq!(
            dispatch_key(
                egui::Key::ArrowRight,
                egui::Modifiers::default(),
                Some(focused_widget),
                &pdf
            ),
            None,
            "arrow key should remain owned by {focused_widget}"
        );
    }
    assert_eq!(
        dispatch_key(egui::Key::ArrowRight, egui::Modifiers::SHIFT, None, &pdf),
        None
    );
    assert_eq!(
        dispatch_key(
            egui::Key::ArrowRight,
            egui::Modifiers::default(),
            None,
            &frame(ViewerDocumentFormat::Docx, 0, 3)
        ),
        None
    );
    assert_eq!(
        dispatch_key(
            egui::Key::ArrowLeft,
            egui::Modifiers::default(),
            None,
            &frame(ViewerDocumentFormat::Pdf, 0, 3)
        ),
        None
    );
    assert_eq!(
        dispatch_key(
            egui::Key::ArrowRight,
            egui::Modifiers::default(),
            None,
            &frame(ViewerDocumentFormat::Pptx, 2, 3)
        ),
        None
    );
}
