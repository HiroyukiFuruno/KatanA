use super::{local_image_frame_size, validate_open_file_waits};

#[test]
fn image_wait_defaults_off_and_rejects_other_wait_modes() {
    let default: crate::request::OpenFileStep =
        serde_json::from_str(r#"{"file_name":"a.png"}"#).unwrap();
    assert!(!default.wait_for_local_image_texture);
    for other in ["wait_for_html_frame", "wait_for_document_frame"] {
        let step: crate::request::OpenFileStep = serde_json::from_value(serde_json::json!({
            "file_name": "a.png", "wait_for_local_image_texture": true, (other): true
        }))
        .unwrap();
        assert!(validate_open_file_waits(&step).is_err());
    }
}

#[test]
fn local_image_requires_a_mesh_in_the_current_frame() {
    let ctx = egui::Context::default();
    let local = ctx.load_texture(
        "local_image_fixture",
        egui::ColorImage::filled([8, 12], egui::Color32::RED),
        egui::TextureOptions::LINEAR,
    );
    let other = ctx.load_texture(
        "other_fixture",
        egui::ColorImage::filled([4, 4], egui::Color32::BLUE),
        egui::TextureOptions::LINEAR,
    );
    let draw = |texture| {
        ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.image((texture, egui::vec2(30.0, 30.0)));
        })
    };
    let first = draw(local.id());
    assert_eq!(
        local_image_frame_size(&ctx, &first.shapes).unwrap(),
        Some([8, 12])
    );
    let next = draw(other.id());
    assert_eq!(local_image_frame_size(&ctx, &next.shapes).unwrap(), None);
    let blank = ctx.run_ui(egui::RawInput::default(), |_| {});
    assert_eq!(local_image_frame_size(&ctx, &blank.shapes).unwrap(), None);
}

#[test]
fn local_image_mesh_is_recognized_only_when_visible() {
    let ctx = egui::Context::default();
    let texture = ctx.load_texture(
        "local_image_mesh",
        egui::ColorImage::filled([5, 6], egui::Color32::RED),
        egui::TextureOptions::LINEAR,
    );
    let bounds = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(10.0, 10.0));
    let mut mesh = egui::Mesh::with_texture(texture.id());
    mesh.add_rect_with_uv(
        bounds,
        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
    let shape = egui::epaint::ClippedShape {
        clip_rect: bounds,
        shape: egui::Shape::mesh(mesh),
    };
    assert_eq!(
        local_image_frame_size(&ctx, &[shape.clone()]).unwrap(),
        Some([5, 6])
    );
    let hidden = egui::epaint::ClippedShape {
        clip_rect: bounds.translate(egui::vec2(20.0, 0.0)),
        ..shape
    };
    assert_eq!(local_image_frame_size(&ctx, &[hidden]).unwrap(), None);
}
