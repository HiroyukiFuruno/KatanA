//! Pure egui UI rendering functions for the preview pane.
//!
//! This module contains code that depends entirely on the egui UI context (`egui::Ui`).
//! - Button click events (`button().clicked()`)
//! - Texture loading (`ui.ctx().load_texture()`)
//! - UI component rendering
//!
//! Since these cannot be executed without an egui frame context,
//! they are excluded from coverage measurement using `--ignore-filename-regex`.

use eframe::egui::{self, Vec2};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use katana_core::markdown::color_preset::DiagramColorPreset;
use katana_core::markdown::svg_rasterize::RasterizedSvg;

use crate::preview_pane::{DownloadRequest, RenderedSection};

/// Text color for the tool not installed warning (orange).
const WARNING_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 165, 0);

/// Vertical spacing (in points) added for empty lines in centered Markdown.
const CENTERED_BLANK_LINE_SPACING: f32 = 4.0;

/// Byte length of the badge image prefix `[![`.
const BADGE_PREFIX_LEN: usize = "[![".len();

/// Renders a single section.
/// Returns `Some(DownloadRequest)` if the download button is pressed.
pub(crate) fn show_section(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    section: &RenderedSection,
    id: usize,
) -> Option<DownloadRequest> {
    match section {
        RenderedSection::Markdown(md) => {
            let preset = DiagramColorPreset::current();
            // Boost the text color for dark-theme readability.
            let prev_override = ui.visuals().override_text_color;
            if let Some((r, g, b)) = DiagramColorPreset::parse_hex_rgb(preset.preview_text) {
                ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(r, g, b));
            }
            CommonMarkViewer::new()
                .syntax_theme_dark(preset.syntax_theme_dark)
                .syntax_theme_light(preset.syntax_theme_light)
                .show(ui, cache, md);
            ui.visuals_mut().override_text_color = prev_override;
            None
        }
        RenderedSection::CenteredMarkdown(md) => {
            let preset = DiagramColorPreset::current();
            let text_color = DiagramColorPreset::parse_hex_rgb(preset.preview_text)
                .map(|(r, g, b)| egui::Color32::from_rgb(r, g, b));
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                for line in md.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        ui.add_space(CENTERED_BLANK_LINE_SPACING);
                    } else if let Some(heading) = trimmed.strip_prefix("# ") {
                        let mut rt = egui::RichText::new(heading).heading();
                        if let Some(c) = text_color {
                            rt = rt.color(c);
                        }
                        ui.label(rt);
                    } else {
                        render_centered_line(ui, trimmed, text_color);
                    }
                }
            });
            None
        }
        RenderedSection::Image { svg_data, alt } => {
            show_rasterized(ui, svg_data, alt, id);
            None
        }
        RenderedSection::Error {
            kind,
            _source: _,
            message,
        } => {
            ui.label(
                egui::RichText::new(crate::i18n::tf(
                    "render_error",
                    &[("kind", kind), ("message", message)],
                ))
                .color(egui::Color32::YELLOW)
                .small(),
            );
            None
        }
        RenderedSection::CommandNotFound {
            tool_name,
            install_hint,
            _source: _,
        } => {
            let msg = crate::i18n::t("missing_dependency")
                .replace("{tool_name}", tool_name)
                .replace("{install_hint}", install_hint);
            ui.label(
                egui::RichText::new(msg)
                    .color(egui::Color32::YELLOW)
                    .small(),
            );
            None
        }
        RenderedSection::NotInstalled {
            kind,
            download_url,
            install_path,
        } => show_not_installed(ui, kind, download_url, install_path),
        RenderedSection::Pending { kind } => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    egui::RichText::new(crate::i18n::tf("rendering", &[("kind", kind)])).weak(),
                );
            });
            None
        }
    }
}

/// Download button UI for uninstalled tools.
pub(crate) fn show_not_installed(
    ui: &mut egui::Ui,
    kind: &str,
    download_url: &str,
    install_path: &std::path::Path,
) -> Option<DownloadRequest> {
    let mut request = None;
    ui.group(|ui| {
        ui.label(
            egui::RichText::new(crate::i18n::tf("tool_not_installed", &[("tool", kind)]))
                .color(WARNING_TEXT_COLOR),
        );
        ui.label(
            egui::RichText::new(crate::i18n::tf(
                "tool_install_path",
                &[("path", &install_path.display().to_string())],
            ))
            .small()
            .weak(),
        );
        if ui
            .button(crate::i18n::tf("tool_download", &[("tool", kind)]))
            .clicked()
        {
            request = Some(DownloadRequest {
                url: download_url.to_string(),
                dest: install_path.to_path_buf(),
            });
        }
    });
    request
}

/// Displays rasterized SVG as an egui texture.
pub(crate) fn show_rasterized(ui: &mut egui::Ui, img: &RasterizedSvg, _alt: &str, id: usize) {
    let color_img = egui::ColorImage::from_rgba_unmultiplied(
        [img.width as usize, img.height as usize],
        &img.rgba,
    );
    let texture = ui.ctx().load_texture(
        format!("diagram_{id}"),
        color_img,
        egui::TextureOptions::LINEAR,
    );
    let max_w = ui.available_width();
    let scale = (max_w / img.width as f32).min(1.0);
    let size = Vec2::new(img.width as f32 * scale, img.height as f32 * scale);
    ui.add(egui::Image::new((texture.id(), size)));
}

/// Renders the section list sequentially and returns a download request if any.
pub(crate) fn render_sections(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    sections: &[RenderedSection],
) -> Option<DownloadRequest> {
    let mut request: Option<DownloadRequest> = None;
    for (i, section) in sections.iter().enumerate() {
        ui.push_id(format!("section_{i}"), |ui| {
            if let Some(req) = show_section(ui, cache, section, i) {
                request = Some(req);
            }
            ui.separator();
        });
    }
    if sections.is_empty() {
        ui.label(egui::RichText::new(crate::i18n::t("no_preview")).weak());
    }
    request
}

/// Parsed Markdown image reference.
struct MdImage {
    src: String,
    alt: String,
    /// Number of characters consumed from the input string.
    consumed: usize,
}

/// Renders a single line of centered content, handling inline images and text.
fn render_centered_line(ui: &mut egui::Ui, line: &str, text_color: Option<egui::Color32>) {
    let mut remaining = line;

    while !remaining.is_empty() {
        // Find next image pattern
        if let Some(offset) = find_next_image(remaining) {
            // Render text before the image
            let before = remaining[..offset].trim();
            if !before.is_empty() {
                let mut rt = egui::RichText::new(before);
                if let Some(c) = text_color {
                    rt = rt.color(c);
                }
                ui.label(rt);
            }

            remaining = &remaining[offset..];
            if let Some(img) = parse_md_image(remaining) {
                let consumed = img.consumed;
                if img.src.starts_with("http://") || img.src.starts_with("https://") {
                    // HTTP images not supported — show alt text as badge label
                    if !img.alt.is_empty() {
                        ui.label(
                            egui::RichText::new(&img.alt)
                                .small()
                                .italics()
                                .color(egui::Color32::GRAY),
                        );
                    }
                } else {
                    let image = egui::Image::new(img.src).fit_to_original_size(1.0);
                    ui.add(image);
                }
                remaining = &remaining[consumed..];
            } else {
                // False start — skip one character and retry
                remaining = &remaining[1..];
            }
        } else {
            // No more images — render remaining text
            let text = remaining.trim();
            if !text.is_empty() {
                let mut rt = egui::RichText::new(text);
                if let Some(c) = text_color {
                    rt = rt.color(c);
                }
                ui.label(rt);
            }
            break;
        }
    }
}

/// Finds the byte offset of the next image pattern (`![` or `[![`).
/// For badge patterns `[![`, returns the position of the opening `[`.
fn find_next_image(s: &str) -> Option<usize> {
    // Check for badge pattern `[![` first — it starts one char before `![`
    let badge_pos = s.find("[![");
    let simple_pos = s.find("![");

    match (badge_pos, simple_pos) {
        (Some(b), Some(s)) => Some(b.min(s)),
        (Some(b), None) => Some(b),
        (None, Some(s)) => Some(s),
        (None, None) => None,
    }
}

/// Parses a Markdown image at the start of the given string.
/// Returns `None` if the string doesn't start with a recognized image pattern.
fn parse_md_image(s: &str) -> Option<MdImage> {
    // Badge pattern: [![alt](img_url)](link_url)
    if let Some(rest) = s.strip_prefix("[![") {
        let alt_end = rest.find(']')?;
        let alt = &rest[..alt_end];
        let after_alt = &rest[alt_end + 1..];
        let inner_src = after_alt.strip_prefix('(')?;
        let src_end = inner_src.find(')')?;
        let src = &inner_src[..src_end];
        // After inner: "](link_url)"
        let after_inner = &inner_src[src_end + 1..];
        let after_bracket = after_inner.strip_prefix("](")?;
        let link_end = after_bracket.find(')')?;
        let total = BADGE_PREFIX_LEN + alt_end + 1 + 1 + src_end + 1 + 2 + link_end + 1;
        return Some(MdImage {
            src: src.to_string(),
            alt: alt.to_string(),
            consumed: total,
        });
    }

    // Simple image: ![alt](src)
    let rest = s.strip_prefix("![")?;
    let close_bracket = rest.find("](")?;
    let alt = &rest[..close_bracket];
    let after = &rest[close_bracket + 2..];
    let close_paren = after.find(')')?;
    let src = &after[..close_paren];
    if src.is_empty() {
        return None;
    }
    let total = 2 + close_bracket + 2 + close_paren + 1;
    Some(MdImage {
        src: src.to_string(),
        alt: alt.to_string(),
        consumed: total,
    })
}
