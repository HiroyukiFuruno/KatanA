use katana_ui::preview_pane::RendererLogicOps;

#[test]
fn valid_svg_is_extracted() {
    /* WHY: Verify that the SVG regex extraction correctly pulls out <svg> content from an HTML wrap. */
    let html = r#"<div><svg width="100" height="100"><rect/></svg></div>"#;
    let svg = RendererLogicOps::extract_svg(html).unwrap();
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn returns_none_if_no_svg_is_present() {
    /* WHY: Verify that the extraction fails safely when no valid SVG tag is present. */
    assert!(RendererLogicOps::extract_svg("<div>hello</div>").is_none());
    assert!(RendererLogicOps::extract_svg("").is_none());
}

#[test]
fn covers_from_start_to_end_if_multiple_svgs_are_present() {
    /* WHY: Verify that if multiple SVG tags exist in the same buffer, the extraction correctly identifies them. */
    let html = r#"<svg>first</svg><p>mid</p><svg>second</svg>"#;
    let svg = RendererLogicOps::extract_svg(html).unwrap();
    assert!(svg.contains("first"));
    assert!(svg.contains("second"));
}

#[test]
fn valid_png_is_decoded() {
    /* WHY: Verify that PNG data can be correctly decoded into raw RGBA bytes for rendering in an egui texture. */
    let mut buf = Vec::new();
    {
        use image::{ImageBuffer, Rgba};
        let img = ImageBuffer::from_pixel(1, 1, Rgba([255u8, 255, 255, 255]));
        let mut cursor = std::io::Cursor::new(&mut buf);
        img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();
    }
    let result = RendererLogicOps::decode_png_rgba(&buf);
    assert!(result.is_ok());
    let rasterized = result.unwrap();
    assert_eq!(rasterized.width, 1);
    assert_eq!(rasterized.height, 1);
    /* WHY: 1x1 RGBA = 4 bytes */
    assert_eq!(rasterized.rgba.len(), 4);
}

#[test]
fn invalid_data_returns_error() {
    /* WHY: Verify that nonsensical data results in a decoding error instead of a crash. */
    let result = RendererLogicOps::decode_png_rgba(b"not a png");
    assert!(result.is_err());
}
