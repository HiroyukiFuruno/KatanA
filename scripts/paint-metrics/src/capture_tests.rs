use super::capture;

#[test]
fn captures_real_multiline_text_mesh_and_origin() {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(300.0, 200.0),
        )),
        ..Default::default()
    };
    let mut first_output = ctx.run_ui(input.clone(), |ui| {
        egui::Area::new("offset".into())
            .fixed_pos(egui::pos2(17.0, 23.0))
            .show(ui.ctx(), |ui| {
                ui.label("alpha\nbeta");
                ui.label("a b");
            });
    });
    first_output.textures_delta.clear();
    // A second real frame ensures deferred font/layout state is materialized.
    let mut output = ctx.run_ui(input, |ui| {
        egui::Area::new("offset".into())
            .fixed_pos(egui::pos2(17.0, 23.0))
            .show(ui.ctx(), |ui| {
                ui.label("alpha\nbeta");
                ui.label("a b");
            });
    });
    let value = capture(&output);
    output.textures_delta.clear();
    let shapes = value["text_shapes"].as_array().expect("text shapes");
    let shape = shapes
        .iter()
        .find(|shape| shape["text"] == "alpha\nbeta")
        .expect("label");
    assert_eq!(shape["origin"]["x"], 17.0);
    assert!(shape["rows"].as_array().expect("rows").len() >= 2);
    let rows = shape["rows"].as_array().expect("rows");
    let first_baseline_y = rows[0]["glyphs"][0]["baseline"]["y"]
        .as_f64()
        .expect("first baseline");
    let second_baseline_y = rows[1]["glyphs"][0]["baseline"]["y"]
        .as_f64()
        .expect("second baseline");
    assert!(first_baseline_y > 23.0);
    assert!(second_baseline_y > first_baseline_y);
    assert!(shape["mesh_bounds"].is_object());
    assert!(shape["rows"][0]["glyphs"][0]["glyph_band"].is_object());
    let spaced = shapes
        .iter()
        .find(|shape| shape["text"] == "a b")
        .expect("spaced label");
    assert_eq!(spaced["rows"][0]["glyphs"][1]["char"], " ");
    assert!(spaced["rows"][0]["glyphs"][1]["glyph_band"].is_null());
    assert!(
        capture(&egui::FullOutput::default())["text_shapes"]
            .as_array()
            .expect("text shapes")
            .is_empty()
    );
}

#[test]
fn rotated_text_fails_closed() {
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(200.0, 100.0),
            )),
            ..Default::default()
        },
        |ui| {
            let galley = ui.ctx().fonts_mut(|fonts| {
                fonts.layout_no_wrap(
                    "rotated".to_owned(),
                    egui::FontId::proportional(14.0),
                    egui::Color32::WHITE,
                )
            });
            ui.painter().add(
                egui::epaint::TextShape::new(egui::pos2(5.0, 7.0), galley, egui::Color32::WHITE)
                    .with_angle(0.25),
            );
        },
    );
    let value = capture(&output);
    output.textures_delta.clear();
    let shape = &value["text_shapes"][0];
    assert_eq!(shape["coordinate_status"], "unsupported_rotation");
    assert!(shape.get("origin").is_none());
}
