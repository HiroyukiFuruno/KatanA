use super::types::*;
use crate::app_state::{AppAction, ScrollSource};
use crate::preview_pane::{DownloadRequest, PreviewPane};
use crate::shell_ui::{LIGHT_MODE_ICON_ACTIVE_BG, LIGHT_MODE_ICON_BG, PREVIEW_CONTENT_PADDING};
use eframe::egui;

impl PreviewLogicOps {
    pub fn preview_panel_id(path: Option<&std::path::Path>, base: &'static str) -> egui::Id {
        match path {
            Some(path) => egui::Id::new((base, path)),
            None => egui::Id::new(base),
        }
    }

    pub fn invalidate_preview_image_cache(ctx: &egui::Context, action: &AppAction) {
        if matches!(action, AppAction::RefreshDiagrams) {
            crate::icon::IconRegistry::install(ctx);
        }
    }
}

impl<'a> PreviewContent<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        preview: &'a mut PreviewPane,
        document: Option<&'a katana_core::document::Document>,
        scroll: &'a mut crate::app_state::ScrollState,
        toc_visible: bool,
        show_toc: bool,
        action: &'a mut AppAction,
        scroll_sync: bool,
        search_query: Option<String>,
    ) -> Self {
        Self {
            preview,
            document,
            scroll,
            toc_visible,
            show_toc,
            action,
            scroll_sync,
            search_query,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        let preview = self.preview;
        let document = self.document;
        let scroll = self.scroll;
        let toc_visible = self.toc_visible;
        let show_toc = self.show_toc;
        let action = self.action;
        let scroll_sync = self.scroll_sync;
        let search_query = self.search_query.clone();
        let mut download_req = None;
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        let outer_rect = ui.available_rect_before_wrap();
        ui.allocate_rect(outer_rect, egui::Sense::hover());

        let mut scroll_area = egui::ScrollArea::vertical()
            .id_salt("preview_scroll")
            .auto_shrink(std::array::from_fn(|_| false));

        let mut forced_offset = None;
        let consuming_editor = scroll_sync && scroll.source == ScrollSource::Editor;
        if consuming_editor {
            forced_offset = Some(scroll.mapper.logical_to_preview(scroll.logical_position));
        } else if let Some(target_line) = scroll.scroll_to_line
            && !scroll.preview_search_scroll_pending
        {
            if !scroll_sync {
                let mut found_offset = None;
                for (span, rect) in &preview.heading_anchors {
                    if span.contains(&target_line) || span.start >= target_line {
                        found_offset = Some((rect.min.y - preview.content_top_y).max(0.0));
                        break;
                    }
                }
                if let Some(off) = found_offset {
                    forced_offset = Some(off);
                } else if let Some((_, rect)) = preview.heading_anchors.last() {
                    forced_offset = Some((rect.min.y - preview.content_top_y).max(0.0));
                } else {
                    forced_offset = Some(0.0);
                }
            } else {
                let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
                let editor_y = target_line as f32 * row_height;
                scroll.logical_position = scroll.mapper.editor_to_logical(editor_y);
                forced_offset = Some(scroll.mapper.logical_to_preview(scroll.logical_position));
            }
        }

        if let Some(target_scroll_offset) = forced_offset {
            scroll.preview_echo.record(target_scroll_offset);
            scroll_area = scroll_area.vertical_scroll_offset(target_scroll_offset);
        }

        let mut content_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(outer_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        content_ui.set_clip_rect(outer_rect);

        let output = scroll_area.show(&mut content_ui, |ui| {
            egui::Frame::NONE
                .inner_margin(egui::Margin::symmetric(
                    PREVIEW_CONTENT_PADDING,
                    PREVIEW_CONTENT_PADDING,
                ))
                .show(ui, |ui| {
                    let content_width = ui.available_width();
                    let child_rect = egui::Rect::from_min_size(
                        ui.next_widget_position(),
                        egui::vec2(content_width, 0.0),
                    );
                    ui.scope_builder(
                        egui::UiBuilder::new()
                            .max_rect(child_rect)
                            .layout(egui::Layout::top_down(egui::Align::Min)),
                        |ui| {
                            const PREVIEW_PANE_TOP_BOTTOM_PADDING: f32 = 4.0; // WHY: 0.25rem padding
                            ui.add_space(PREVIEW_PANE_TOP_BOTTOM_PADDING);
                            let mut hovered_lines = Vec::new();
                            let (req, actions) = preview.show_content(
                                ui,
                                scroll.active_editor_line,
                                Some(&mut hovered_lines),
                                search_query.clone(),
                                scroll.preview_search_scroll_pending,
                            );
                            if scroll.preview_search_scroll_pending {
                                scroll.preview_search_scroll_pending = false;
                            }
                            if scroll_sync && scroll.source != ScrollSource::Preview {
                                scroll.hovered_preview_lines = hovered_lines.clone();
                            }

                            if ui.rect_contains_pointer(ui.min_rect())
                                && ui.input(|i| i.pointer.primary_clicked())
                                && let Some(hovered) = hovered_lines.first()
                            {
                                scroll.scroll_to_line = Some(hovered.start);
                            }
                            download_req = req;
                            if let Some((global_index, new_state)) = actions.into_iter().next() {
                                *action = AppAction::ToggleTaskList {
                                    global_index,
                                    new_state,
                                };
                            }
                            ui.add_space(PREVIEW_PANE_TOP_BOTTOM_PADDING);
                        },
                    );
                });
        });

        if scroll_sync {
            let max_scroll = (output.content_size.y - output.inner_rect.height()).max(0.0);
            scroll.preview_max = max_scroll;

            let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
            let mut computed_anchors = Vec::with_capacity(preview.heading_anchors.len());
            for (span, rect) in &preview.heading_anchors {
                let p_y = (rect.min.y - preview.content_top_y).max(0.0);
                computed_anchors.push((span.clone(), p_y));
            }

            scroll.mapper = crate::state::scroll_sync::ScrollMapper::build(
                scroll.editor_max,
                scroll.preview_max,
                row_height,
                &computed_anchors,
            );

            if consuming_editor {
                scroll.source = ScrollSource::Neither;
            } else if max_scroll > 0.0 {
                let preview_y = output.state.offset.y;
                if !scroll.preview_echo.is_echo(preview_y) {
                    let next_logical = scroll.mapper.preview_to_logical(preview_y);
                    if next_logical != scroll.logical_position {
                        scroll.logical_position = next_logical;
                        scroll.source = ScrollSource::Preview;
                    }
                }
            }
        }

        PreviewHeader::new(document.is_some(), toc_visible, show_toc, action).show(ui);

        download_req
    }
}

impl<'a> PreviewHeader<'a> {
    pub fn new(
        has_doc: bool,
        toc_visible: bool,
        show_toc: bool,
        action: &'a mut AppAction,
    ) -> Self {
        Self {
            has_doc,
            toc_visible,
            show_toc,
            action,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let has_doc = self.has_doc;
        let action = self.action;
        let button_size = egui::vec2(ui.spacing().interact_size.y, ui.spacing().interact_size.y);
        let margin = f32::from(PREVIEW_CONTENT_PADDING);
        let spacing = ui.spacing().item_spacing.x;
        let mut button_count = 2.0; // WHY: Export, Slideshow
        if self.toc_visible {
            button_count += 1.0;
        }
        let total_width = (button_size.x * button_count) + (spacing * (button_count - 1.0));

        let button_rect = egui::Rect::from_min_size(
            egui::pos2(
                ui.max_rect().right() - margin - total_width,
                ui.max_rect().top() + margin,
            ),
            egui::vec2(total_width, button_size.y),
        );
        let mut overlay_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(button_rect)
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
        );

        let icon_bg = if ui.visuals().dark_mode {
            crate::theme_bridge::TRANSPARENT
        } else {
            crate::theme_bridge::ThemeBridgeOps::from_gray(LIGHT_MODE_ICON_BG)
        };

        let export_img = egui::Image::new(crate::icon::Icon::Export.uri())
            .tint(overlay_ui.visuals().text_color());
        overlay_ui.scope(|ui| {
            ui.visuals_mut().widgets.inactive.bg_fill = icon_bg;

            if ui
                .add_enabled(
                    has_doc,
                    egui::Button::image_and_text(
                        crate::Icon::Preview.ui_image(ui, crate::icon::IconSize::Medium),
                        crate::shell_ui::ShellUiOps::invisible_label("toggle_slideshow"),
                    )
                    .min_size(button_size)
                    .fill(icon_bg),
                )
                .on_hover_text(crate::i18n::I18nOps::get().action.toggle_slideshow.clone())
                .clicked()
            {
                *action = AppAction::ToggleSlideshow;
            }

            ui.menu_image_button(export_img, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                if ui
                    .button(crate::i18n::I18nOps::get().menu.export_html.clone())
                    .clicked()
                {
                    *action = AppAction::ExportDocument(crate::app_state::ExportFormat::Html);
                    ui.close();
                }
                if ui
                    .button(crate::i18n::I18nOps::get().menu.export_pdf.clone())
                    .clicked()
                {
                    *action = AppAction::ExportDocument(crate::app_state::ExportFormat::Pdf);
                    ui.close();
                }
                if ui
                    .button(crate::i18n::I18nOps::get().menu.export_png.clone())
                    .clicked()
                {
                    *action = AppAction::ExportDocument(crate::app_state::ExportFormat::Png);
                    ui.close();
                }
                if ui
                    .button(crate::i18n::I18nOps::get().menu.export_jpg.clone())
                    .clicked()
                {
                    *action = AppAction::ExportDocument(crate::app_state::ExportFormat::Jpg);
                    ui.close();
                }
            });
        });

        if self.toc_visible {
            let toc_bg = if self.show_toc {
                if ui.visuals().dark_mode {
                    ui.visuals().selection.bg_fill
                } else {
                    crate::theme_bridge::ThemeBridgeOps::from_gray(LIGHT_MODE_ICON_ACTIVE_BG)
                }
            } else {
                icon_bg
            };
            if overlay_ui
                .add_enabled(
                    has_doc,
                    egui::Button::image_and_text(
                        crate::Icon::Toc.ui_image(ui, crate::icon::IconSize::Medium),
                        crate::shell_ui::ShellUiOps::invisible_label("toggle_toc"),
                    )
                    .min_size(button_size)
                    .fill(toc_bg),
                )
                .on_hover_text(crate::i18n::I18nOps::get().action.toggle_toc.clone())
                .clicked()
            {
                *action = AppAction::ToggleToc;
            }
        }
    }
}
