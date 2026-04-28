use crate::app_action::{AppAction, CodeBlockKind, MarkdownAuthoringOp};
use crate::i18n::I18nOps;
use crate::icon::{Icon, IconSize};
use eframe::egui;

const CODE_BLOCK_MENU_MIN_WIDTH: f32 = 140.0;
const CODE_BLOCK_MENU_GAP: f32 = 4.0;
const CODE_BLOCK_MENU_ID: &str = "editor_code_block_kind_menu";
const CODE_BLOCK_MENU_OPEN_ID: &str = "editor_code_block_kind_menu_open";
const CODE_BLOCK_MENU_POS_ID: &str = "editor_code_block_kind_menu_pos";

pub(crate) struct CodeBlockMenuOps;

impl CodeBlockMenuOps {
    pub(crate) fn show(ui: &mut egui::Ui, action: &mut AppAction) -> bool {
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
}

pub(crate) struct CodeBlockMenuPopupOps;

impl CodeBlockMenuPopupOps {
    pub(crate) fn show(ui: &mut egui::Ui, action: &mut AppAction) {
        let label = I18nOps::get().editor.toolbar.code_block.clone();
        let button_response = ui
            .add(Icon::Code.button(ui, IconSize::Small))
            .on_hover_text(label.clone());
        button_response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label.clone())
        });
        if button_response.clicked() {
            Self::set_open(ui, true);
            Self::store_pos(ui, button_response.rect.left_bottom());
        }

        if Self::is_open(ui) {
            Self::show_menu(ui, action);
        }
    }

    fn show_menu(ui: &mut egui::Ui, action: &mut AppAction) {
        let pos = Self::pos(ui) + egui::vec2(0.0, CODE_BLOCK_MENU_GAP);
        let area_response = egui::Area::new(egui::Id::new(CODE_BLOCK_MENU_ID))
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_min_width(CODE_BLOCK_MENU_MIN_WIDTH);
                    CodeBlockMenuOps::show(ui, action)
                })
            });

        crate::widgets::InteractionFacade::consume_rect(
            ui,
            "editor_code_block_kind_menu_input_blocker",
            area_response.response.rect,
        );
        if area_response.inner.inner {
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

    fn pos(ui: &egui::Ui) -> egui::Pos2 {
        ui.memory(|mem| {
            mem.data
                .get_temp::<egui::Pos2>(egui::Id::new(CODE_BLOCK_MENU_POS_ID))
                .unwrap_or(egui::Pos2::ZERO)
        })
    }

    fn store_pos(ui: &egui::Ui, pos: egui::Pos2) {
        ui.memory_mut(|mem| {
            mem.data
                .insert_temp(egui::Id::new(CODE_BLOCK_MENU_POS_ID), pos);
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
}
