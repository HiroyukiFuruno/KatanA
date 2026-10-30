use katana_platform::SplitDirection;

use crate::app::preview::PreviewOps;
use crate::shell::KatanaApp;
use crate::theme_bridge;
use crate::views::panels::preview::PreviewContent;
use katana_platform::PaneOrder;
pub(crate) struct SplitMode<'a> {
    pub _ctx: &'a egui::Context,
    pub app: &'a mut KatanaApp,
    pub split_dir: SplitDirection,
    pub pane_order: PaneOrder,
}
impl<'a> SplitMode<'a> {
    pub fn new(
        _ctx: &'a egui::Context,
        app: &'a mut KatanaApp,
        split_dir: SplitDirection,
        pane_order: PaneOrder,
    ) -> Self {
        Self {
            _ctx,
            app,
            split_dir,
            pane_order,
        }
    }
    pub fn show(self, ui: &mut egui::Ui) {
        let app = self.app;
        let split_dir = self.split_dir;
        let pane_order = self.pane_order;
        let ctx = ui.ctx().clone();
        let is_markdown = app
            .state
            .active_document()
            .is_some_and(|document| is_markdown_path(&document.path));
        paint_split_background(ui, is_markdown);
        match split_dir {
            SplitDirection::Horizontal => HorizontalSplit::new(&ctx, app, pane_order).show(ui),
            SplitDirection::Vertical => VerticalSplit::new(&ctx, app, pane_order).show(ui),
        };
    }
}

fn is_markdown_path(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
        })
}

fn paint_split_background(ui: &egui::Ui, is_markdown: bool) {
    if is_markdown {
        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, ui.visuals().window_fill());
    }
}

pub(crate) use super::split_horizontal::HorizontalSplit;

pub(crate) use super::split_vertical::VerticalSplit;

pub(crate) struct PreviewOnly<'a> {
    pub ui: &'a mut egui::Ui,
    pub app: &'a mut KatanaApp,
}
impl<'a> PreviewOnly<'a> {
    pub fn new(ui: &'a mut egui::Ui, app: &'a mut KatanaApp) -> Self {
        Self { ui, app }
    }
    pub fn show(self) {
        let ui = self.ui;
        let app = self.app;
        ui.painter().rect_filled(
            ui.max_rect(),
            0.0,
            theme_bridge::ThemeBridgeOps::rgb_to_color32(
                app.state
                    .config
                    .settings
                    .settings()
                    .effective_theme_colors()
                    .preview
                    .background,
            ),
        );
        let active_path = app.state.active_document().map(|d| d.path.clone());
        if let Some(path) = active_path {
            let pane = crate::shell::KatanaApp::get_preview_pane(&mut app.tab_previews, path);
            PreviewContent::new(
                pane,
                app.state.document.active_document(),
                &mut app.state.scroll,
                &mut app.pending_action,
                false,
                Some(app.state.search.doc_search.query.clone()),
                Some(app.state.search.doc_search_active_index),
            )
            .show(ui);

            /* WHY: In PreviewOnly mode, there is no editor to consume the scroll_to_line request. */
            /* WHY: We consume it here right after PreviewContent has processed it. */
            app.state.scroll.scroll_to_line = None;
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(
                    crate::i18n::I18nOps::get()
                        .workspace
                        .no_document_selected
                        .clone(),
                );
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_markdown_path, paint_split_background};
    use eframe::egui;

    #[test]
    fn split_background_uses_explorer_window_role_only_for_markdown() {
        assert!(is_markdown_path(std::path::Path::new("notes.MD")));
        assert!(is_markdown_path(std::path::Path::new("notes.markdown")));
        assert!(!is_markdown_path(std::path::Path::new("notes.html")));

        let ctx = egui::Context::default();
        for preset in katana_platform::theme::ThemePreset::builtins() {
            let colors = preset.colors();
            let visuals = crate::theme_bridge::ThemeBridgeOps::visuals_from_theme(&colors);
            let mut style = (*ctx.global_style()).clone();
            style.visuals = visuals.clone();

            /* WHY: 実際のExplorerPanelと同じ背景色の役割を全テーマで維持する。 */
            let explorer_fill =
                crate::views::panels::explorer::panel::ExplorerPanel::frame(&style).fill;
            assert_eq!(
                explorer_fill,
                visuals.window_fill,
                "Explorer window role changed for {}",
                preset.display_name()
            );

            ctx.set_global_style(style);
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                paint_split_background(ui, true);
            });
            assert!(
                output.shapes.iter().any(|shape| matches!(
                    &shape.shape,
                    egui::Shape::Rect(rect) if rect.fill == explorer_fill
                )),
                "Markdown split did not paint the Explorer fill for {}",
                preset.display_name()
            );
            if visuals.window_fill != visuals.panel_fill {
                assert!(
                    !output.shapes.iter().any(|shape| matches!(
                        &shape.shape,
                        egui::Shape::Rect(rect) if rect.fill == visuals.panel_fill
                    )),
                    "Markdown split used panel_fill for {}",
                    preset.display_name()
                );
            }
            output.textures_delta.clear();

            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                paint_split_background(ui, false);
            });
            assert!(
                !output.shapes.iter().any(|shape| matches!(
                    &shape.shape,
                    egui::Shape::Rect(rect) if rect.fill == explorer_fill
                )),
                "non-Markdown split unexpectedly painted Explorer fill for {}",
                preset.display_name()
            );
            output.textures_delta.clear();
        }
    }
}
