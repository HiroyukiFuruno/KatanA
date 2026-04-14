use eframe::egui::{self};

pub use super::types::HtmlLogicOps;

impl HtmlLogicOps {
    pub(crate) fn render_html_block(
        ui: &mut egui::Ui,
        html: &str,
        text_color: Option<egui::Color32>,
        md_file_path: &std::path::Path,
    ) {
        let clip_rect = ui.clip_rect();
        let ctx = ui.ctx().clone();
        let block_rect = egui::Rect::from_min_size(
            egui::pos2(ui.max_rect().left(), ui.next_widget_position().y),
            egui::vec2(ui.max_rect().width(), ui.available_height()),
        );

        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(block_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |block_ui| {
                block_ui.set_clip_rect(clip_rect);

                const HTML_BLOCK_MARGIN_TOP_ADJUST: f32 = -7.0;
                block_ui.add_space(HTML_BLOCK_MARGIN_TOP_ADJUST);

                let resolved_html = katana_core::preview::ImagePreviewOps::resolve_html_image_paths(
                    html,
                    md_file_path,
                );
                let base_dir = md_file_path.parent().unwrap_or(std::path::Path::new("."));
                let parser = katana_core::html::HtmlParser::new(base_dir);
                let nodes = parser.parse(&resolved_html);
                let mut renderer = crate::html_renderer::HtmlRenderer::new(block_ui, base_dir);
                if let Some(c) = text_color {
                    renderer = renderer.text_color(c);
                }
                if let Some(action) = renderer.render(&nodes) {
                    match action {
                        katana_core::html::LinkAction::OpenInBrowser(url) => {
                            super::types::PreviewPaneUtilsOps::open_tab(&ctx, &url);
                        }
                        katana_core::html::LinkAction::NavigateCurrentTab(path) => {
                            super::types::PreviewPaneUtilsOps::open_tab(
                                &ctx,
                                &path.to_string_lossy(),
                            );
                        }
                    }
                }

                const HTML_BLOCK_MARGIN_BOTTOM_ADJUST: f32 = -3.0;
                block_ui.add_space(HTML_BLOCK_MARGIN_BOTTOM_ADJUST);
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::path::Path;
    use std::rc::Rc;

    use eframe::egui;
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };

    use super::HtmlLogicOps;
    const HTML_BADGE_HEIGHT_MIN: f32 = 28.0;

    #[test]
    fn html_block_badge_advances_cursor_before_following_text() {
        let after_html_y = Rc::new(Cell::new(0.0_f32));
        let after_html_y_capture = Rc::clone(&after_html_y);
        let html = concat!(
            "<p align=\"center\">",
            "<a href=\"https://github.com/sponsors/HiroyukiFuruno\">",
            "<img src=\"https://img.shields.io/badge/Sponsor-❤️-ea4aaa?style=for-the-badge&logo=github-sponsors\" alt=\"Sponsor\">",
            "</a>",
            "</p>",
        );

        let mut harness = Harness::builder()
            .with_size(egui::vec2(800.0, 240.0))
            .build_ui(move |ui| {
                HtmlLogicOps::render_html_block(ui, html, None, Path::new("/tmp/README.md"));
                after_html_y_capture.set(ui.next_widget_position().y);
                ui.label("Support helps cover:");
            });
        harness.step();
        harness.run();

        let label = harness.get_by_label("Support helps cover:");
        let bounds = label
            .accesskit_node()
            .raw_bounds()
            .expect("following label should have bounds");
        let gap_from_top = bounds.y0 as f32;

        assert!(
            after_html_y.get() >= HTML_BADGE_HEIGHT_MIN,
            "HTML badge block must advance cursor by a meaningful height, got {:.1}",
            after_html_y.get()
        );
        assert!(
            gap_from_top >= HTML_BADGE_HEIGHT_MIN,
            "Following text must render below the badge row, got Y={gap_from_top:.1}"
        );
    }
}
