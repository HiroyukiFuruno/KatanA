use eframe::egui;
use katana_document_viewer::{
    DocumentPageSurfaceFrame, DocumentSurfaceCommand, DocumentSurfaceFrame, DocumentViewport,
};

use super::painter::DocumentFramePainter;

const MILLI_SCALE: f32 = 1_000.0;
const CENTER_RATIO: f32 = 0.5;
const PAGE_BORDER_WIDTH: f32 = 1.0;

pub(super) fn paint(
    painter: &mut DocumentFramePainter,
    ui: &mut egui::Ui,
    frame: &DocumentSurfaceFrame,
    surface_id: u64,
) -> Vec<DocumentSurfaceCommand> {
    let viewport = ui.available_size().max(egui::vec2(1.0, 1.0));
    let command =
        DocumentSurfaceCommand::Resize(DocumentViewport::new(viewport.x as u32, viewport.y as u32));
    let Some(page) = frame.page() else {
        return vec![command];
    };
    update_texture(painter, ui, page);
    if let Some(texture) = &painter.texture {
        paint_page(ui, texture, page, surface_id);
    }
    vec![command]
}

fn update_texture(
    painter: &mut DocumentFramePainter,
    ui: &egui::Ui,
    page: &DocumentPageSurfaceFrame,
) {
    if painter.texture_fingerprint.as_deref() == Some(&page.fingerprint) {
        return;
    }
    let image = egui::ColorImage::from_rgba_unmultiplied(
        [page.width as usize, page.height as usize],
        &page.rgba,
    );
    painter.texture = Some(ui.ctx().load_texture(
        format!("katana-document:{}", page.fingerprint),
        image,
        egui::TextureOptions::LINEAR,
    ));
    painter.texture_fingerprint = Some(page.fingerprint.clone());
}

fn paint_page(
    ui: &mut egui::Ui,
    texture: &egui::TextureHandle,
    page: &DocumentPageSurfaceFrame,
    surface_id: u64,
) {
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .id_salt(("katana_document_page", surface_id))
        .show(ui, |ui| {
            paint_centered_image(ui, texture, display_size(page))
        });
}

fn display_size(page: &DocumentPageSurfaceFrame) -> egui::Vec2 {
    egui::vec2(
        page.display_width_milli as f32 / MILLI_SCALE,
        page.display_height_milli as f32 / MILLI_SCALE,
    )
}

fn paint_centered_image(ui: &mut egui::Ui, texture: &egui::TextureHandle, size: egui::Vec2) {
    let available = ui.available_width();
    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        if size.x < available {
            ui.add_space((available - size.x) * CENTER_RATIO);
        }
        let response = ui.add(egui::Image::new(texture).fit_to_exact_size(size));
        let color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        let stroke = egui::Stroke::new(PAGE_BORDER_WIDTH, color);
        ui.painter().add(egui::Shape::rect_stroke(
            response.rect.shrink(PAGE_BORDER_WIDTH),
            0.0,
            stroke,
            egui::StrokeKind::Inside,
        ));
    });
}

#[cfg(test)]
#[path = "painter_page_tests.rs"]
mod tests;
