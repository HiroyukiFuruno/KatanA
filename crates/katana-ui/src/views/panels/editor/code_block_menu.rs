use crate::app_action::{AppAction, CodeBlockKind, MarkdownAuthoringOp};
use crate::i18n::I18nOps;
use crate::icon::{Icon, IconSize};
use eframe::egui;

const CODE_BLOCK_MENU_MIN_WIDTH: f32 = 140.0;
const CODE_BLOCK_MENU_OPEN_ID: &str = "editor_code_block_kind_menu_open";

pub(crate) struct CodeBlockMenuOps;

impl CodeBlockMenuOps {
    pub(crate) fn show(ui: &mut egui::Ui, action: &mut AppAction) -> bool {
        ui.scope(|ui| {
            Self::apply_menu_item_visuals(ui);
            Self::show_items(ui, action)
        })
        .inner
    }

    fn show_items(ui: &mut egui::Ui, action: &mut AppAction) -> bool {
        let mut clicked = false;
        for kind in CodeBlockKind::all() {
            if ui.button(kind.display_label()).clicked() {
                *action = AppAction::AuthorMarkdown(MarkdownAuthoringOp::CodeBlock(*kind));
                clicked = true;
                ui.close();
            }
        }
        clicked
    }

    fn apply_menu_item_visuals(ui: &mut egui::Ui) {
        crate::widgets::StyledComboBox::apply_popup_visuals(ui);
    }
}

pub(crate) struct CodeBlockMenuPopupOps;

impl CodeBlockMenuPopupOps {
    pub(crate) fn show(ui: &mut egui::Ui, action: &mut AppAction) {
        let label = I18nOps::get().editor.toolbar.code_block.clone();
        let button_response = ui
            .add(Icon::Code.button(ui, IconSize::Small))
            .on_hover_text(label.clone());
        if button_response.clicked() {
            Self::set_open(ui, true);
        }
        Self::show_menu(ui, action, &button_response);
        button_response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label.clone())
        });
    }

    fn show_menu(ui: &mut egui::Ui, action: &mut AppAction, button_response: &egui::Response) {
        let menu_clicked = egui::Popup::from_response(button_response)
            .open(Self::is_open(ui))
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.set_min_width(CODE_BLOCK_MENU_MIN_WIDTH);
                CodeBlockMenuOps::show(ui, action)
            })
            .is_some_and(|response| response.inner);
        if menu_clicked || (!button_response.clicked() && button_response.clicked_elsewhere()) {
            Self::set_open(ui, false);
        }
    }

    pub(crate) fn is_open(ui: &egui::Ui) -> bool {
        ui.memory(|mem| {
            mem.data
                .get_temp::<bool>(egui::Id::new(CODE_BLOCK_MENU_OPEN_ID))
                .unwrap_or(false)
        })
    }

    pub(crate) fn set_open(ui: &egui::Ui, open: bool) {
        ui.memory_mut(|mem| {
            mem.data
                .insert_temp(egui::Id::new(CODE_BLOCK_MENU_OPEN_ID), open)
        });
    }

    #[cfg(test)]
    fn close(ui: &egui::Ui) {
        Self::set_open(ui, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_block_menu_state_defaults_closed() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                assert!(!CodeBlockMenuPopupOps::is_open(ui));
            });
        });
    }

    #[test]
    fn code_block_menu_state_can_be_closed() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                CodeBlockMenuPopupOps::set_open(ui, true);
                assert!(CodeBlockMenuPopupOps::is_open(ui));
                CodeBlockMenuPopupOps::close(ui);
                assert!(!CodeBlockMenuPopupOps::is_open(ui));
            });
        });
    }

    #[test]
    fn code_block_menu_applies_transparent_inactive_item_background() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                CodeBlockMenuOps::apply_menu_item_visuals(ui);

                assert_eq!(
                    ui.visuals().widgets.inactive.bg_fill,
                    crate::theme_bridge::TRANSPARENT
                );
                assert_eq!(
                    ui.visuals().widgets.inactive.weak_bg_fill,
                    crate::theme_bridge::TRANSPARENT
                );
            });
        });
    }
}
