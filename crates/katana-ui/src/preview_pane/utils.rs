use super::types::*;
use eframe::egui;

impl PreviewPaneUtilsOps {
    pub fn open_tab(ctx: &egui::Context, url: &str) {
        ctx.open_url(egui::OpenUrl::new_tab(url));
    }

    pub fn with_preview_text_style<R>(
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        ui.scope(|ui| {
            let fonts_loaded = ui.ctx().data(|d| {
                d.get_temp::<bool>(egui::Id::new("katana_fonts_loaded"))
                    .unwrap_or(false)
            });
            if fonts_loaded {
                Self::set_preview_body_family(
                    ui,
                    egui::FontFamily::Name("MarkdownProportional".into()),
                );
            } else {
                Self::set_preview_body_family(ui, egui::FontFamily::Proportional);
            }

            if let Some(color) = ui.ctx().data(|d| {
                d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                    "katana_theme_colors",
                ))
            }) {
                ui.visuals_mut().override_text_color = Some(
                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(color.preview.text),
                );
                ui.visuals_mut().selection.bg_fill =
                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(color.preview.selection);
            }

            add_contents(ui)
        })
        .inner
    }

    pub fn set_preview_body_family(ui: &mut egui::Ui, family: egui::FontFamily) {
        let style = ui.style_mut();
        style.override_font_id = None;
        style.override_text_style = None;
        for text_style in [
            egui::TextStyle::Body,
            egui::TextStyle::Button,
            egui::TextStyle::Heading,
            egui::TextStyle::Small,
        ] {
            if let Some(font_id) = style.text_styles.get_mut(&text_style) {
                font_id.family = family.clone();
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn handle_local_image_section(
        path: &str,
        alt: &str,
        lines: usize,
        md_file_path: &std::path::Path,
        cache: std::sync::Arc<dyn katana_platform::CacheFacade>,
        force: bool,
        current_generation: u64,
        ordinal: usize,
        sections: &mut Vec<RenderedSection>,
        jobs: &mut Vec<RenderJob>,
        lifecycle: &mut Vec<SectionLifecycle>,
    ) {
        let path_buf = std::path::PathBuf::from(path.trim_start_matches("file://"));

        if path_buf.extension().and_then(|s| s.to_str()) == Some("drawio") {
            let Ok(xml) = std::fs::read_to_string(&path_buf) else {
                return Self::push_normal_image(&path_buf, alt, lines, sections, lifecycle);
            };
            sections.push(RenderedSection::Pending {
                kind: "Drawio".to_string(),
                source: xml.clone(),
                source_lines: lines,
            });
            jobs.push(RenderJob {
                kind: katana_core::markdown::DiagramKind::DrawIo,
                src: xml,
                path: md_file_path.to_path_buf(),
                cache,
                force,
                source_lines: lines,
                generation: current_generation,
                ordinal,
            });
            lifecycle.push(SectionLifecycle {
                is_loaded: false,
                is_drawn: false,
            });
            return;
        }

        Self::push_normal_image(&path_buf, alt, lines, sections, lifecycle);
    }

    fn push_normal_image(
        path_buf: &std::path::Path,
        alt: &str,
        lines: usize,
        sections: &mut Vec<RenderedSection>,
        lifecycle: &mut Vec<SectionLifecycle>,
    ) {
        sections.push(RenderedSection::LocalImage {
            path: path_buf.to_path_buf(),
            alt: alt.to_string(),
            source_lines: lines,
        });
        lifecycle.push(SectionLifecycle {
            is_loaded: true,
            is_drawn: false,
        });
    }
}
