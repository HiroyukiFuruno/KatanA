use super::{navigation_command, navigation_response};
use eframe::egui;
const POINTER_CENTER: f32 = 50.0;
const PAGE_COUNT: usize = 3;
const FOCUS_TARGET_SIZE: egui::Vec2 = egui::vec2(100.0, 20.0);

use katana_document_viewer::{
    DocumentFrame, DocumentSurfaceFrame, DocumentViewerCommand, DocumentViewerState,
    PdfRenderedPage, ViewerCapabilities, ViewerDocumentFormat, ViewerImageSurface,
};

pub(super) fn frame(
    format: ViewerDocumentFormat,
    active_index: usize,
    item_count: usize,
) -> DocumentFrame {
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
    let mut text_buffer = String::new();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        if let Some(widget) = focused_widget {
            focus_widget(ui, widget, &mut text_buffer, true);
        } else {
            navigation_response(ui, 0).request_focus();
        }
    });
    output.textures_delta.clear();

    let mut command = None;
    let mut events = vec![egui::Event::ModifiersChanged(modifiers)];
    events.push(egui::Event::PointerMoved(egui::pos2(
        POINTER_CENTER,
        POINTER_CENTER,
    )));
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
        focused: true,
        events,
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ui| {
        if let Some(widget) = focused_widget {
            focus_widget(ui, widget, &mut text_buffer, false);
        }
        let response = navigation_response(ui, 0);
        command = navigation_command(ui, &response, frame);
    });
    output.textures_delta.clear();
    command
}

fn focus_widget(ui: &mut egui::Ui, widget: &str, text_buffer: &mut String, request_focus: bool) {
    if matches!(widget, "markdown-editor" | "document-search") {
        let response = ui.add(egui::TextEdit::singleline(text_buffer).id(egui::Id::new(widget)));
        if request_focus {
            response.request_focus();
        }
    } else {
        let response = ui.interact(
            egui::Rect::from_min_size(egui::Pos2::ZERO, FOCUS_TARGET_SIZE),
            egui::Id::new(widget),
            egui::Sense::focusable_noninteractive(),
        );
        if request_focus {
            response.request_focus();
        }
    }
}

#[test]
fn focused_document_routes_page_arrows_for_pdf_and_pptx() {
    assert_eq!(
        dispatch_key(
            egui::Key::ArrowRight,
            egui::Modifiers::default(),
            None,
            &frame(ViewerDocumentFormat::Pdf, 0, PAGE_COUNT)
        ),
        Some(DocumentViewerCommand::Next)
    );
    assert_eq!(
        dispatch_key(
            egui::Key::ArrowLeft,
            egui::Modifiers::default(),
            None,
            &frame(ViewerDocumentFormat::Pptx, 1, PAGE_COUNT)
        ),
        Some(DocumentViewerCommand::Previous)
    );
}

#[test]
fn page_navigation_ignores_unfocused_widgets_other_formats_modifiers_and_boundaries() {
    let pdf = frame(ViewerDocumentFormat::Pdf, 0, PAGE_COUNT);
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
            &frame(ViewerDocumentFormat::Docx, 0, PAGE_COUNT)
        ),
        None
    );
    assert_eq!(
        dispatch_key(
            egui::Key::ArrowLeft,
            egui::Modifiers::default(),
            None,
            &frame(ViewerDocumentFormat::Pdf, 0, PAGE_COUNT)
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
