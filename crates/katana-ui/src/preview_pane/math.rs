use eframe::egui::{self};

#[derive(Clone)]
pub(crate) struct MathJaxCache(
    std::sync::Arc<egui::mutex::Mutex<std::collections::BTreeMap<String, String>>>,
);

impl Default for MathJaxCache {
    fn default() -> Self {
        Self(std::sync::Arc::new(egui::mutex::Mutex::new(
            Default::default(),
        )))
    }
}

/// Renders a LaTeX mathematical formula as an SVG image.
///
/// This function is registered as the `math_fn` callback in `egui_commonmark::CommonMarkViewer`.
pub(crate) fn render_math(ui: &mut egui::Ui, tex: &str, is_inline: bool) {
    /// Horizontal padding (left/right) inside the block math frame.
    const MATH_BLOCK_H_MARGIN: i8 = 8;
    /// Vertical padding (top/bottom) inside the block math frame.
    const MATH_BLOCK_V_MARGIN: i8 = 4;
    /// Corner radius for the block math frame.
    const MATH_BLOCK_CORNER_RADIUS: u8 = 4;
    /// Conversion ratio roughly equalizing 1 `ex` to pixel height in our font rendering context.
    const EX_TO_PX: f32 = 8.5;
    /// Negative top margin used to perfectly align inline math center with vertical text layout tops.
    const INLINE_MATH_MARGIN_TOP: i8 = -8;
    /// Negative top/bottom margin used to tighten the visual gap for block math spacing natively rendered.
    const BLOCK_MATH_MARGIN_VERTICAL: i8 = -10;
    let tex = tex.trim();
    if tex.is_empty() {
        return;
    }

    // MathJax paths must perfectly match the visual text colour (since math is inherently 'text')
    let text_color = ui.visuals().text_color();
    let hex_color = format!(
        "#{:02x}{:02x}{:02x}",
        text_color.r(),
        text_color.g(),
        text_color.b()
    );

    // Hash map key differentiates block/inline and the EXACT textual math colour requested,
    // to force re-rendering if the user changes the preview.text setting dynamically.
    let is_dark = ui.visuals().dark_mode;
    let cache_key = format!(
        "{}:{}:{}:{}",
        if is_dark { "dark" } else { "light" },
        hex_color,
        if is_inline { "inline" } else { "block" },
        tex
    );

    // Retrieve or create the cache in egui's temporary state storage.
    let cache = ui.ctx().memory_mut(|mem| {
        mem.data
            .get_temp_mut_or_default::<MathJaxCache>(egui::Id::new("katana_mathjax_cache"))
            .clone()
    });

    let uri = {
        let mut map = cache.0.lock();
        if let Some(cached_uri) = map.get(&cache_key) {
            cached_uri.clone()
        } else {
            // MathJax V8 initialization panics if called from multiple test threads concurrently.
            // In the actual app, egui rendering is single-threaded, but `cargo test` runs in parallel.
            static MATHJAX_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

            let svg_result = {
                let _lock = MATHJAX_LOCK.lock().unwrap();
                if is_inline {
                    mathjax_svg::convert_to_svg_inline(tex)
                } else {
                    mathjax_svg::convert_to_svg(tex)
                }
            };

            let data_uri = match svg_result {
                Ok(svg_string) => {
                    // `mathjax_svg` returns dimensions in `ex` units (e.g. width="8.6ex").
                    // `usvg` in KatanaSvgLoader does not support `.ex` parsing and will fail with a Bad px_scale_factor zero crash.
                    // We extract the `ex` value, scale it (1 ex ~ 8.5px aligns nicely with our font), and replace it with `px`.
                    let mut processed_svg = svg_string;
                    let width_re = regex::Regex::new(r#"width="([\d\.]+)ex""#).unwrap();
                    let height_re = regex::Regex::new(r#"height="([\d\.]+)ex""#).unwrap();

                    if let Some(caps) = width_re.captures(&processed_svg) {
                        if let Ok(w_ex) = caps.get(1).unwrap().as_str().parse::<f32>() {
                            let w_px = w_ex * EX_TO_PX;
                            processed_svg = width_re
                                .replace(&processed_svg, format!("width=\"{w_px}px\""))
                                .into_owned();
                        }
                    }
                    if let Some(caps) = height_re.captures(&processed_svg) {
                        if let Ok(h_ex) = caps.get(1).unwrap().as_str().parse::<f32>() {
                            let h_px = h_ex * EX_TO_PX;
                            processed_svg = height_re
                                .replace(&processed_svg, format!("height=\"{h_px}px\""))
                                .into_owned();
                        }
                    }

                    // `usvg` doesn't automatically inherit css `currentColor`.
                    // MathJax emits generic currentColor fields which we must explicitly colorize.
                    // (Already retrieved `hex_color` string earlier to form our cache key)
                    processed_svg = processed_svg.replace("currentColor", &hex_color);

                    use base64::{engine::general_purpose, Engine as _};
                    let b64 = general_purpose::STANDARD.encode(processed_svg.as_bytes());
                    format!("data:image/svg+xml;base64,{}", b64)
                }
                Err(e) => {
                    tracing::error!("MathJax rendering failed for {:?}: {}", tex, e);
                    // Return an empty string to trigger fallback
                    String::new()
                }
            };
            map.insert(cache_key.clone(), data_uri.clone());
            data_uri
        }
    };

    if uri.is_empty() {
        // Fallback display if rendering failed
        if is_inline {
            ui.label(
                egui::RichText::new(tex)
                    .monospace()
                    .color(ui.visuals().error_fg_color),
            );
        } else {
            egui::Frame::new()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(egui::Margin::symmetric(
                    MATH_BLOCK_H_MARGIN,
                    MATH_BLOCK_V_MARGIN,
                ))
                .corner_radius(egui::CornerRadius::same(MATH_BLOCK_CORNER_RADIUS))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(tex)
                            .monospace()
                            .color(ui.visuals().error_fg_color),
                    );
                });
        }
        return;
    }

    // Provide the original tex to AccessKit by putting a visually hidden label,
    // or by adding a tooltip (accessible name).
    let response = if is_inline {
        // To gracefully align the math graphics with the surrounding Japanese characters,
        // we use a negative top margin. Since egui horizontally aligns inline widgets
        // by their tops, this shrinks the bounding box at the top, physically shifting
        // the drawn image UP relative to the text by exactly the margin value.
        egui::Frame::new()
            .inner_margin(egui::Margin {
                left: 0,
                right: 0,
                top: INLINE_MATH_MARGIN_TOP,
                bottom: 0,
            })
            .show(ui, |ui| {
                ui.add(egui::Image::new(&uri).fit_to_original_size(1.0))
            })
            .inner
    } else {
        // Block math: MathJax display mode natively includes generous vertical spacing.
        // To tighten the flow without cropping the SVG, we use negative margins (-10px top/bottom).
        // This brings the surrounding text 10px closer to the integral/block equation.
        egui::Frame::new()
            .inner_margin(egui::Margin {
                left: 0,
                right: 0,
                top: BLOCK_MATH_MARGIN_VERTICAL,
                bottom: BLOCK_MATH_MARGIN_VERTICAL,
            })
            .show(ui, |ui| {
                ui.add(egui::Image::new(&uri).fit_to_original_size(1.0))
            })
            .inner
    };

    // To ensure UI integration tests (accesskit get_by_label) can find this math element,
    // add an invisible label right after the image if it wasn't intercepted by tooltip.
    response.on_hover_text(tex);

    // Add a tiny transparent label so `get_by_label` locates it correctly inside the dom tree.
    let mut rect = ui.cursor();
    rect.max = rect.min;
    ui.put(
        rect,
        egui::Label::new(
            egui::RichText::new(tex)
                .size(1.0)
                .color(crate::theme_bridge::TRANSPARENT),
        ),
    );
}
