use crate::shell_ui::{
    TOC_HEADING_VISIBILITY_THRESHOLD, TOC_INDENT_PER_LEVEL, TOC_PANEL_DEFAULT_WIDTH,
    TOC_PANEL_MARGIN,
};
use eframe::egui;

fn find_active_toc_index_preview(
    heading_anchors: &[(std::ops::Range<usize>, egui::Rect)],
    threshold: f32,
) -> usize {
    let mut active = 0;
    for (i, (_, rect)) in heading_anchors.iter().enumerate() {
        if rect.min.y <= threshold {
            active = i;
        } else {
            break;
        }
    }
    active
}

fn find_active_toc_index_editor(
    anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
    current_line: f32,
) -> usize {
    let mut active = 0;
    let logical_threshold = current_line + 1.0;
    for item in anchor_map {
        if matches!(item.kind, katana_core::markdown::outline::AnchorKind::Heading) 
            && let Some(idx) = item.index {
            if (item.line_span.start as f32) > logical_threshold {
                break;
            }
            active = idx;
        }
    }
    active
}

pub(crate) struct TocPanel<'a> {
    pub preview: &'a mut crate::preview_pane::PreviewPane,
    pub state: &'a crate::app_state::AppState,
}

impl<'a> TocPanel<'a> {
    pub fn new(
        preview: &'a mut crate::preview_pane::PreviewPane,
        state: &'a crate::app_state::AppState,
    ) -> Self {
        Self { preview, state }
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<usize> {
        let preview = self.preview;
        let state = self.state;
        use katana_platform::settings::TocPosition;
        let position = state.config.settings.settings().layout.toc_position;
        let mut clicked_line = None;

        let panel = match position {
            TocPosition::Left => egui::Panel::left("toc_panel"),
            TocPosition::Right => egui::Panel::right("toc_panel"),
        };

        let frame =
            egui::Frame::side_top_panel(&ui.ctx().global_style()).inner_margin(TOC_PANEL_MARGIN);

        panel
            .frame(frame)
            .resizable(true)
            .default_size(TOC_PANEL_DEFAULT_WIDTH)
            .show_inside(ui, |ui| {
                ui.heading(crate::i18n::I18nOps::get().toc.title.clone());
                ui.separator();

                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        if preview.outline_items.is_empty() {
                            ui.label(
                                egui::RichText::new(crate::i18n::I18nOps::get().toc.empty.clone())
                                    .weak()
                                    .italics(),
                            );
                        } else {
                            let active_index = if let Some(visible_rect) = preview.visible_rect {
                                let threshold = visible_rect.min.y + TOC_HEADING_VISIBILITY_THRESHOLD;
                                find_active_toc_index_preview(&preview.heading_anchors, threshold)
                            } else {
                                let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
                                let editor_y = state
                                    .scroll
                                    .mapper
                                    .logical_to_editor(state.scroll.logical_position);
                                let current_line = editor_y / row_height;
                                find_active_toc_index_editor(&preview.anchor_map, current_line)
                            };

                            let mut next_scroll = None;
                            for (i, item) in preview.outline_items.iter().enumerate() {
                                let indent = (item.level as f32 - 1.0) * TOC_INDENT_PER_LEVEL;
                                crate::widgets::AlignCenter::new()
                                    .shrink_to_fit(true)
                                    .content(|ui| {
                                        ui.add_space(indent);
                                        let is_active = i == active_index;
                                        let mut text = egui::RichText::new(&item.text);
                                        if is_active {
                                            text = text
                                                .strong()
                                                .color(ui.visuals().widgets.active.text_color());
                                        }
                                        if ui
                                            .add(
                                                egui::Button::selectable(is_active, text)
                                                    .frame_when_inactive(true),
                                            )
                                            .clicked()
                                        {
                                            next_scroll = Some(item.index);
                                        }
                                    })
                                    .show(ui);
                            }
                            if let Some(index) = next_scroll {
                                preview.scroll_request = Some(index);
                                if let Some(item) =
                                    preview.outline_items.iter().find(|i| i.index == index)
                                {
                                    clicked_line = Some(item.line_start);
                                }
                            }
                        }
                    });
            });

        clicked_line
    }
}
