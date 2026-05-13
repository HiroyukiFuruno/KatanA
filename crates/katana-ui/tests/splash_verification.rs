use eframe::egui;

#[test]
fn test_splash_screen_centering() {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(800.0, 600.0))
        .build_ui(|ctx| {
            let overlay = katana_ui::views::splash::SplashOverlay::new(1.0, None, true);
            overlay.show(ctx);
        });
    harness.step();
    harness.snapshot("splash_screen_centering_800x600");
}

#[test]
fn test_splash_screen_centering_large() {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1920.0, 1080.0))
        .build_ui(|ctx| {
            let overlay = katana_ui::views::splash::SplashOverlay::new(1.0, None, true);
            overlay.show(ctx);
        });
    harness.step();
    harness.snapshot("splash_screen_centering_1920x1080");
}
