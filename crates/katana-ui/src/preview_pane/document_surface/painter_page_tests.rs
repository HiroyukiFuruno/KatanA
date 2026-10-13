use eframe::egui;
use katana_document_viewer::DocumentPageSurfaceFrame;

const DISPLAY_WIDTH_MILLI: u32 = 40_000;
const DISPLAY_HEIGHT_MILLI: u32 = 30_000;
const PIXEL_CHANNEL: u8 = 255;
const RGBA_LENGTH: usize = 16;
const OVERFLOW_WIDTH: f32 = 10_000.0;
const DISPLAY_WIDTH: f32 = 40.0;
const DISPLAY_HEIGHT: f32 = 30.0;

fn page() -> DocumentPageSurfaceFrame {
    DocumentPageSurfaceFrame {
        fingerprint: "page-fixture".to_owned(),
        width: 2,
        height: 2,
        display_width_milli: DISPLAY_WIDTH_MILLI,
        display_height_milli: DISPLAY_HEIGHT_MILLI,
        content_scale: 1,
        accessibility_label: "page".to_owned(),
        rgba: vec![PIXEL_CHANNEL; RGBA_LENGTH],
    }
}

#[test]
fn texture_update_reuses_fingerprint_and_page_can_overflow_viewport() {
    let context = crate::test_ui::Context::default();
    let mut owner = super::DocumentFramePainter::default();
    let page = page();
    context.run_ui(egui::RawInput::default(), |ui| {
        super::update_texture(&mut owner, ui, &page);
        let texture = owner.texture.as_ref().expect("texture");
        super::paint_page(ui, texture, &page, 7);
        super::paint_centered_image(ui, texture, egui::vec2(OVERFLOW_WIDTH, DISPLAY_HEIGHT));
    });
    let texture = owner.texture.as_ref().expect("texture").id();
    context.run_ui(egui::RawInput::default(), |ui| {
        super::update_texture(&mut owner, ui, &page);
    });
    assert_eq!(owner.texture.as_ref().expect("texture").id(), texture);
    assert_eq!(
        super::display_size(&page),
        egui::vec2(DISPLAY_WIDTH, DISPLAY_HEIGHT)
    );
}
