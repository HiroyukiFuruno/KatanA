mod anchor_lookup_ops;
mod anchor_ops;
mod ops;
mod render;
mod types;

use crate::shell_ui::TOC_PANEL_DEFAULT_WIDTH;
pub(crate) use types::*;

use eframe::egui;

#[cfg(test)]
mod tests;

const TOC_MIN_WIDTH: f32 = 100.0;
const TOC_MAX_WIDTH: f32 = 500.0;

impl<'a> TocPanel<'a> {
    pub(crate) fn panel_frame(style: &egui::Style) -> egui::Frame {
        egui::Frame::window(style).inner_margin(egui::Margin::ZERO)
    }

    /// Render the TOC as a `SidePanel` (used only when the panel is pinned in a split layout).
    ///
    /// WHY: Kept for callers that still need a SidePanel (e.g. split-view without overlay).
    #[allow(dead_code)]
    pub fn show_as_panel(
        mut self,
        ui: &mut egui::Ui,
    ) -> (Option<usize>, Option<usize>, egui::Rect) {
        use katana_platform::settings::TocPosition;
        let position = self.state.config.settings.settings().layout.toc_position;

        let width = TOC_PANEL_DEFAULT_WIDTH;
        let panel = match position {
            TocPosition::Left => egui::SidePanel::left("toc_panel"),
            TocPosition::Right => egui::SidePanel::right("toc_panel"),
        };

        let mut output = (None, None);
        let panel_resp = panel
            .frame(Self::panel_frame(&ui.ctx().global_style()))
            .default_width(width)
            .width_range(TOC_MIN_WIDTH..=TOC_MAX_WIDTH)
            .show_inside(ui, |ui| {
                output = self.show_toc_content(ui);
            });

        (output.0, output.1, panel_resp.response.rect)
    }

    /// Render the TOC content directly into the given `ui` without a wrapping SidePanel.
    ///
    /// WHY: Used by the overlay popup in side_panel_toc.rs where the panel is shown as a
    /// Foreground Area and must not create an additional SidePanel inside it.
    pub fn show(mut self, ui: &mut egui::Ui) -> (Option<usize>, Option<usize>, egui::Rect) {
        let (clicked_line, active_index_out) = self.show_toc_content(ui);
        let rect = ui.min_rect();
        (clicked_line, active_index_out, rect)
    }

    fn show_toc_content(&mut self, ui: &mut egui::Ui) -> (Option<usize>, Option<usize>) {
        let mut clicked_line = None;
        let mut active_index_out = None;

        ui.vertical(|ui| {
            self.render_header(ui);
            ui.separator();
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
                    let now = ui.input(|i| i.time);
                    let active_index = self.active_toc_index_from_anchor_state(row_height, now);
                    active_index_out = Some(active_index);
                    let auto_scroll_active_item = self.state.toc.should_auto_scroll();
                    if auto_scroll_active_item {
                        self.state.toc.consume_auto_scroll();
                    }

                    let ctx = TocRenderContext {
                        items: &self.preview.outline_items,
                        active_index,
                        auto_scroll_active_item,
                        show_vertical_lines: self
                            .state
                            .config
                            .settings
                            .settings()
                            .layout
                            .accordion_vertical_line,
                        force_open: self.state.layout.toc_force_open,
                    };

                    let mut idx = 0;
                    let mut next_scroll = None;
                    while idx < ctx.items.len() {
                        if let Some(scroll) = self.render_toc_item(ui, &ctx, &mut idx) {
                            next_scroll = Some(scroll);
                        }
                    }

                    /* WHY: Reset force_open after one frame of application */
                    if self.state.layout.toc_force_open.is_some() {
                        self.state.layout.toc_force_open = None;
                    }

                    if let Some(index) = next_scroll {
                        self.preview.scroll_request = Some(index);
                        Self::record_toc_click_anchor(
                            &mut self.state.toc,
                            &self.preview.anchor_map,
                            index,
                        );
                        active_index_out = Some(index);
                        if let Some(item) =
                            self.preview.outline_items.iter().find(|i| i.index == index)
                        {
                            clicked_line = Some(item.line_start);
                        }
                    }
                });

            /* WHY: Consume force_open after the frame */
            self.state.layout.toc_force_open = None;
        });

        (clicked_line, active_index_out)
    }

    fn render_header(&mut self, ui: &mut egui::Ui) {
        let icon_btn_size =
            crate::icon::IconSize::Small.to_vec2() + ui.spacing().button_padding * 2.0;
        let square_size = icon_btn_size.max_elem();
        let icon_min_size = egui::vec2(square_size, square_size);
        const HEADER_SPACING: f32 = 4.0;

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), icon_min_size.y),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let expand_label = crate::i18n::I18nOps::get().action.expand_all.clone();
                let collapse_label = crate::i18n::I18nOps::get().action.collapse_all.clone();

                if ui
                    .add(Self::panel_icon_button(ui, crate::Icon::ExpandAll))
                    .on_hover_text(expand_label)
                    .clicked()
                {
                    self.state.layout.toc_force_open = Some(true);
                }

                if ui
                    .add(Self::panel_icon_button(ui, crate::Icon::CollapseAll))
                    .on_hover_text(collapse_label)
                    .clicked()
                {
                    self.state.layout.toc_force_open = Some(false);
                }

                ui.add_space(HEADER_SPACING);
                ui.heading(crate::i18n::I18nOps::get().toc.title.clone());
            },
        );
    }

    fn panel_icon_button(ui: &egui::Ui, icon: crate::Icon) -> egui::Button<'static> {
        icon.button_on_fill(ui, crate::icon::IconSize::Small, ui.visuals().window_fill())
    }
}
