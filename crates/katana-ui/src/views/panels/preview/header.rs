use super::types::*;
use crate::app_state::AppAction;
use crate::shell_ui::{LIGHT_MODE_ICON_ACTIVE_BG, LIGHT_MODE_ICON_BG, PREVIEW_CONTENT_PADDING};
use eframe::egui;

impl<'a> PreviewHeader<'a> {
    pub fn new(
        has_doc: bool,
        toc_visible: bool,
        show_toc: bool,
        show_export: bool,
        show_story: bool,
        action: &'a mut AppAction,
    ) -> Self {
        Self {
            has_doc,
            toc_visible,
            show_toc,
            show_export,
            show_story,
            action,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let has_doc = self.has_doc;
        let action = self.action;
        let button_size = egui::vec2(ui.spacing().interact_size.y, ui.spacing().interact_size.y);
        let margin = f32::from(PREVIEW_CONTENT_PADDING);
        let spacing = ui.spacing().item_spacing.x;
        /* WHY: Story, Export, TOC (if visible) */
        let mut button_count = 2.0;
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

        let active_bg = if ui.visuals().dark_mode {
            ui.visuals().selection.bg_fill
        } else {
            crate::theme_bridge::ThemeBridgeOps::from_gray(LIGHT_MODE_ICON_ACTIVE_BG)
        };

        overlay_ui.scope(|ui| {
            ui.visuals_mut().widgets.inactive.bg_fill = icon_bg;

            /* WHY: Story Panel Toggle */
            let story_bg = if self.show_story { active_bg } else { icon_bg };
            if ui
                .add_enabled(
                    has_doc,
                    egui::Button::image(
                        crate::Icon::Preview.ui_image(ui, crate::icon::IconSize::Medium),
                    )
                    .min_size(button_size)
                    .fill(story_bg),
                )
                .on_hover_text(
                    crate::i18n::I18nOps::get()
                        .preview
                        .slideshow_settings
                        .clone(),
                )
                .clicked()
            {
                *action = AppAction::ToggleStoryPanel;
            }

            /* WHY: Export Panel Toggle */
            let export_bg = if self.show_export { active_bg } else { icon_bg };
            if ui
                .add_enabled(
                    has_doc,
                    egui::Button::image(
                        crate::Icon::Export.ui_image(ui, crate::icon::IconSize::Medium),
                    )
                    .min_size(button_size)
                    .fill(export_bg),
                )
                .on_hover_text(crate::i18n::I18nOps::get().menu.export.clone())
                .clicked()
            {
                *action = AppAction::ToggleExportPanel;
            }
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
            let resp_toc = overlay_ui
                .add_enabled(
                    has_doc,
                    egui::Button::image(
                        crate::Icon::Toc.ui_image(ui, crate::icon::IconSize::Medium),
                    )
                    .min_size(button_size)
                    .fill(toc_bg),
                )
                .on_hover_text(crate::i18n::I18nOps::get().action.toggle_toc.clone());
            resp_toc.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    true,
                    crate::i18n::I18nOps::get().action.toggle_toc.clone(),
                )
            });

            if resp_toc.clicked() {
                *action = AppAction::ToggleToc;
            }
        }
    }
}
