mod ops;
mod render;
mod types;

use crate::shell_ui::{
    TOC_HEADING_VISIBILITY_THRESHOLD, TOC_PANEL_DEFAULT_WIDTH, TOC_PANEL_MARGIN,
};
pub(crate) use types::*;

use eframe::egui;

#[cfg(test)]
mod tests;

const TOC_MIN_WIDTH: f32 = 100.0;
const TOC_MAX_WIDTH: f32 = 500.0;

impl<'a> TocPanel<'a> {
    pub fn show(mut self, ui: &mut egui::Ui) -> (Option<usize>, Option<usize>, egui::Rect) {
        use katana_platform::settings::TocPosition;
        let position = self.state.config.settings.settings().layout.toc_position;
        let mut clicked_line = None;
        let mut active_index_out = None;

        let width = TOC_PANEL_DEFAULT_WIDTH;
        let panel = match position {
            TocPosition::Left => egui::SidePanel::left("toc_panel"),
            TocPosition::Right => egui::SidePanel::right("toc_panel"),
        };

        let frame =
            egui::Frame::side_top_panel(&ui.ctx().global_style()).inner_margin(TOC_PANEL_MARGIN);

        let panel_resp = panel
            .frame(frame)
            .default_width(width)
            .width_range(TOC_MIN_WIDTH..=TOC_MAX_WIDTH)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    self.render_header(ui);
                    ui.separator();

                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let active_index;
                            let is_code_only = self.state.active_view_mode()
                                == crate::app_state::ViewMode::CodeOnly;
                            if !is_code_only && let Some(visible_rect) = self.preview.visible_rect {
                                let threshold =
                                    visible_rect.min.y + TOC_HEADING_VISIBILITY_THRESHOLD;
                                active_index = Self::find_active_toc_index_preview(
                                    &self.preview.outline_items,
                                    &self.preview.anchor_map,
                                    threshold,
                                );
                            } else {
                                let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
                                let editor_y = self.state.scroll.editor_y;
                                let current_line = editor_y / row_height;
                                let logical_threshold = current_line + 1.0;
                                active_index = Self::find_active_toc_index_editor(
                                    &self.preview.outline_items,
                                    &self.preview.anchor_map,
                                    logical_threshold,
                                );
                            }
                            active_index_out = Some(active_index);

                            let ctx = TocRenderContext {
                                items: &self.preview.outline_items,
                                active_index,
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
            });

        (clicked_line, active_index_out, panel_resp.response.rect)
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
                    .add(crate::Icon::ExpandAll.button(ui, crate::icon::IconSize::Small))
                    .on_hover_text(expand_label)
                    .clicked()
                {
                    self.state.layout.toc_force_open = Some(true);
                }

                if ui
                    .add(crate::Icon::CollapseAll.button(ui, crate::icon::IconSize::Small))
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
}
