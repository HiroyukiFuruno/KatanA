const INITIAL_WIDTH: f32 = 1280.0;
const INITIAL_HEIGHT: f32 = 800.0;
const MIN_WIDTH: f32 = 800.0;
const MIN_HEIGHT: f32 = 500.0;

pub(super) fn initial_window_size() -> egui::Vec2 {
    egui::vec2(INITIAL_WIDTH, INITIAL_HEIGHT)
}

pub(super) fn min_window_size() -> egui::Vec2 {
    egui::vec2(MIN_WIDTH, MIN_HEIGHT)
}

pub(super) fn load_icon() -> std::sync::Arc<egui::IconData> {
    let icon_bytes = include_bytes!("../../../assets/icon.iconset/icon_512x512.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon byte map")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    std::sync::Arc::new(egui::IconData {
        rgba,
        width,
        height,
    })
}
