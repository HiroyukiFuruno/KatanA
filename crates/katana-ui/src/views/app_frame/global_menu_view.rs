use super::global_menu_context::GlobalMenuContext;
use crate::app_state::AppAction;
use eframe::egui;

pub(super) struct GlobalViewMenu;

impl GlobalViewMenu {
    pub(super) fn render(ui: &mut egui::Ui, context: &mut GlobalMenuContext<'_>) {
        let menu_label = context.i18n().menu.view.clone();
        crate::widgets::MenuButtonOps::show(ui, &menu_label, |ui| {
            let command_palette = context.i18n().menu.command_palette.clone();
            context.shortcut_action_item(
                ui,
                "view.command_palette",
                &command_palette,
                "open_palette",
                AppAction::ToggleCommandPalette,
            );
            ui.separator();
            Self::render_workspace_view_items(ui, context);
            ui.separator();
            Self::render_zoom_items(ui, context);
            ui.separator();
            let close_all = context.i18n().search.command_close_all.clone();
            context.action_item(
                ui,
                "view.close_all",
                &close_all,
                AppAction::CloseAllDocuments,
            );
        });
    }

    fn render_workspace_view_items(ui: &mut egui::Ui, context: &mut GlobalMenuContext<'_>) {
        let explorer = context.i18n().search.command_explorer.clone();
        context.shortcut_action_item(
            ui,
            "view.explorer",
            &explorer,
            "toggle_sidebar",
            AppAction::ToggleExplorer,
        );
        let refresh_explorer = context.i18n().search.command_refresh_explorer.clone();
        context.action_item(
            ui,
            "view.refresh_explorer",
            &refresh_explorer,
            AppAction::RefreshExplorer,
        );
        let refresh_document = context.i18n().action.refresh_document.clone();
        context.shortcut_action_item(
            ui,
            "view.refresh_document",
            &refresh_document,
            "refresh_document",
            AppAction::RefreshDocument { is_manual: true },
        );
    }

    fn render_zoom_items(ui: &mut egui::Ui, context: &mut GlobalMenuContext<'_>) {
        let zoom_in = context.i18n().menu.zoom_in.clone();
        context.shortcut_action_item(ui, "view.zoom_in", &zoom_in, "zoom_in", AppAction::ZoomIn);
        let zoom_out = context.i18n().menu.zoom_out.clone();
        context.shortcut_action_item(
            ui,
            "view.zoom_out",
            &zoom_out,
            "zoom_out",
            AppAction::ZoomOut,
        );
    }
}
