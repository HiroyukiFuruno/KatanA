#![cfg(all(test, feature = "external-fixture-acceptance"))]

mod assertions;
mod expected;
mod fixture;
mod rasterizer;
mod theme_state;

#[test]
fn current_custom_and_builtin_themes_reach_real_pdf_pixels() {
    let _render_environment = crate::test_render_env::RenderEnvLock::lock();
    let _restore_theme = theme_state::ThemeStateRestore::capture();
    let rasterizer = rasterizer::PdfRasterizer::from_required_environment();
    let artifacts = fixture::PdfThemeFixture::create_artifact_directory();

    for case in fixture::PdfThemeFixture::theme_cases() {
        let (pdf_path, theme) = fixture::PdfThemeFixture::export_pdf(&case, &artifacts);
        let png_path = rasterizer.rasterize_first_page(&pdf_path, &artifacts.join(case.name));
        let image = image::open(&png_path)
            .expect("Poppler output must be a readable PNG")
            .to_rgb8();

        assertions::PdfThemeAssertions::assert_theme_pixels(&image, &theme, case.name);
        assertions::PdfThemeAssertions::assert_table_colors_reject_background_substitution(
            &image, &theme, case.name,
        );
    }
}
