use eframe::egui::{self};

pub use super::types::HtmlLogicOps;

impl HtmlLogicOps {
    pub(crate) fn render_html_block(
        ui: &mut egui::Ui,
        html: &str,
        text_color: Option<egui::Color32>,
        md_file_path: &std::path::Path,
    ) {
        let ctx = ui.ctx().clone();
        trace_html_layout(ui, "callback_entry", html);

        /* WHY: Using ui.vertical ensures that the vertical space used by the HTML block */
        /* WHY: is correctly allocated in the parent UI, preventing overlap with subsequent elements. */
        let response = ui.vertical(|ui| {
            trace_html_layout(ui, "vertical_entry", html);
            const HTML_BLOCK_MARGIN_TOP_ADJUST: f32 = -7.0;
            ui.add_space(HTML_BLOCK_MARGIN_TOP_ADJUST);
            trace_html_layout(ui, "after_top_adjust", html);

            let resolved_html =
                katana_core::preview::ImagePreviewOps::resolve_html_image_paths(html, md_file_path);
            let base_dir = md_file_path.parent().unwrap_or(std::path::Path::new("."));
            let parser = katana_core::html::HtmlParser::new(base_dir);
            let nodes = parser.parse(&resolved_html);
            if debug_enabled() {
                ui.ctx().data_mut(|data| {
                    data.insert_temp(
                        egui::Id::new("katana.html_layout_trace.active"),
                        is_trace_target(html),
                    );
                });
            }
            trace_html_layout(ui, "before_renderer", html);
            let mut renderer = crate::html_renderer::HtmlRenderer::new(ui, base_dir);
            if let Some(c) = text_color {
                renderer = renderer.text_color(c);
            }
            if let Some(action) = renderer.render(&nodes) {
                match action {
                    katana_core::html::LinkAction::OpenInBrowser(url) => {
                        super::types::PreviewPaneUtilsOps::open_tab(&ctx, &url);
                    }
                    katana_core::html::LinkAction::NavigateCurrentTab(path) => {
                        super::types::PreviewPaneUtilsOps::open_tab(&ctx, &path.to_string_lossy());
                    }
                }
            }
            if debug_enabled() {
                ui.ctx().data_mut(|data| {
                    data.remove::<bool>(egui::Id::new("katana.html_layout_trace.active"));
                });
            }
            trace_html_layout(ui, "after_renderer", html);

            const HTML_BLOCK_MARGIN_BOTTOM_ADJUST: f32 = -3.0;
            ui.add_space(HTML_BLOCK_MARGIN_BOTTOM_ADJUST);
            trace_html_layout(ui, "after_bottom_adjust", html);
        });
        if debug_enabled() && is_trace_target(html) {
            crate::debug_log::DebugLog::write(
                "html_layout_trace",
                format_args!(
                    "stage=vertical_response frame={} html={} response=({:.1},{:.1},{:.1},{:.1}) parent_next_y={:.1} parent_min=({:.1},{:.1},{:.1},{:.1})",
                    ui.ctx().cumulative_frame_nr(),
                    summarize_html(html),
                    response.response.rect.min.x,
                    response.response.rect.min.y,
                    response.response.rect.max.x,
                    response.response.rect.max.y,
                    ui.next_widget_position().y,
                    ui.min_rect().min.x,
                    ui.min_rect().min.y,
                    ui.min_rect().max.x,
                    ui.min_rect().max.y,
                ),
            );
        }
    }

    pub(crate) fn layout_trace_active(ctx: &egui::Context) -> bool {
        ctx.data(|data| {
            data.get_temp::<bool>(egui::Id::new("katana.html_layout_trace.active"))
                .unwrap_or(false)
        })
    }
}

/// Records the height contributed by the HTML callback to the parent Markdown layout only when DEBUG=true.
/// Includes a short HTML sample so layout compensation can be traced within the same UI frame.
fn trace_html_layout(ui: &egui::Ui, stage: &str, html: &str) {
    if !debug_enabled() || !is_trace_target(html) {
        return;
    }
    let next = ui.next_widget_position();
    let min = ui.min_rect();
    crate::debug_log::DebugLog::write(
        "html_layout_trace",
        format_args!(
            "stage={stage} frame={} html={} body_height={:.1} cursor_height={:.1} item_spacing_y={:.1} next=({:.1},{:.1}) min=({:.1},{:.1},{:.1},{:.1}) available=({:.1},{:.1},{:.1},{:.1})",
            ui.ctx().cumulative_frame_nr(),
            summarize_html(html),
            ui.text_style_height(&egui::TextStyle::Body),
            ui.cursor().height(),
            ui.spacing().item_spacing.y,
            next.x,
            next.y,
            min.min.x,
            min.min.y,
            min.max.x,
            min.max.y,
            ui.available_rect_before_wrap().min.x,
            ui.available_rect_before_wrap().min.y,
            ui.available_rect_before_wrap().max.x,
            ui.available_rect_before_wrap().max.y,
        ),
    );
}

fn debug_enabled() -> bool {
    std::env::var("DEBUG").ok().as_deref() == Some("true")
}

fn is_trace_target(html: &str) -> bool {
    html.contains("KatanA Desktop")
        || html.contains("A fast, lightweight Markdown workspace for macOS")
        || html.contains("License-MIT-blue.svg")
        || html.contains("English |")
        || html.contains("data:image/svg+xml")
}

fn summarize_html(html: &str) -> String {
    const MAX_CHARS: usize = 80;
    let compact = html.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut summary = compact.chars().take(MAX_CHARS).collect::<String>();
    if compact.chars().nth(MAX_CHARS).is_some() {
        summary.push_str("...");
    }
    summary.replace(' ', "_")
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
                ui.vertical(|ui| {
                    HtmlLogicOps::render_html_block(ui, html, None, Path::new("/tmp/README.md"));
                    after_html_y_capture.set(ui.next_widget_position().y);
                    ui.label("Support helps cover:");
                });
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
