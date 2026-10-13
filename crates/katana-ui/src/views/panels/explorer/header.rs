use crate::app_state::AppAction;
use eframe::egui;

pub(crate) struct ExplorerHeader<'a> {
    pub workspace: &'a mut crate::app_state::WorkspaceState,
    pub search: &'a mut crate::app_state::SearchState,
    pub action: &'a mut AppAction,
}

impl<'a> ExplorerHeader<'a> {
    pub fn new(
        workspace: &'a mut crate::app_state::WorkspaceState,
        search: &'a mut crate::app_state::SearchState,
        action: &'a mut AppAction,
    ) -> Self {
        Self {
            workspace,
            search,
            action,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let (workspace, search, action) = (self.workspace, self.search, self.action);

        let ws_root = workspace
            .data
            .as_ref()
            .map(|w| w.root.clone())
            .unwrap_or_default();
        let is_flat = workspace.is_flat_view(&ws_root);

        let icon_btn_size =
            crate::icon::IconSize::Small.to_vec2() + ui.spacing().button_padding * 2.0;
        let square_size = icon_btn_size.max_elem();
        let icon_min_size = egui::vec2(square_size, square_size);

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), icon_min_size.y),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                if workspace.data.is_some() {
                    if workspace.is_temporary_root(&ws_root) {
                        ui.add(crate::Icon::Hourglass.ui_image(ui, crate::icon::IconSize::Small))
                            .on_hover_text(
                                crate::i18n::I18nOps::get()
                                    .workspace
                                    .temporary_workspace_tooltip
                                    .clone(),
                            );
                    }
                    ui.add_enabled_ui(!is_flat, |ui| {
                        let btn_resp = ui
                            .add(Self::panel_icon_button(
                                ui,
                                crate::Icon::ExpandAll,
                                crate::icon::IconSize::Small,
                            ))
                            .on_hover_text(crate::i18n::I18nOps::get().action.expand_all.clone());
                        btn_resp.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "+")
                        });
                        if btn_resp.clicked()
                            && let Some(ws) = &workspace.data
                        {
                            workspace
                                .expanded_directories
                                .extend(ws.collect_all_directory_paths());
                        }

                        let btn_resp = ui
                            .add(Self::panel_icon_button(
                                ui,
                                crate::Icon::CollapseAll,
                                crate::icon::IconSize::Small,
                            ))
                            .on_hover_text(crate::i18n::I18nOps::get().action.collapse_all.clone());
                        btn_resp.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "-")
                        });
                        if btn_resp.clicked() {
                            workspace.force_tree_open = Some(false);
                        }
                    });
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.scope(|ui| {
                        super::header_menu::ExplorerHeaderMenu {
                            workspace,
                            action,
                            ws_root: ws_root.clone(),
                            is_flat,
                        }
                        .show(ui);
                    });

                    if workspace.data.is_some() {
                        let refresh_resp = ui
                            .add(Self::panel_icon_button(
                                ui,
                                crate::Icon::Refresh,
                                crate::icon::IconSize::Small,
                            ))
                            .on_hover_text(
                                crate::i18n::I18nOps::get().action.refresh_explorer.clone(),
                            );
                        refresh_resp.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "⟳")
                        });
                        if refresh_resp.clicked() {
                            *action = AppAction::RefreshExplorer;
                        }

                        let filter_resp = ui
                            .add(
                                Self::panel_icon_button(
                                    ui,
                                    crate::Icon::Filter,
                                    crate::icon::IconSize::Small,
                                )
                                .selected(search.filter_enabled),
                            )
                            .on_hover_text(
                                crate::i18n::I18nOps::get().action.toggle_filter.clone(),
                            );
                        filter_resp.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "\u{2207}")
                        });
                        if filter_resp.clicked() {
                            *action = AppAction::ToggleExplorerFilter;
                        }

                        let new_directory_resp = ui
                            .add(Self::panel_icon_button(
                                ui,
                                crate::Icon::FolderPlus,
                                crate::icon::IconSize::Small,
                            ))
                            .on_hover_text(
                                crate::i18n::I18nOps::get().action.new_directory.clone(),
                            );
                        new_directory_resp.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "folder+")
                        });
                        if new_directory_resp.clicked() {
                            *action = Self::new_directory_action(&ws_root);
                        }

                        let new_file_resp = ui
                            .add(Self::panel_icon_button(
                                ui,
                                crate::Icon::FilePlus,
                                crate::icon::IconSize::Small,
                            ))
                            .on_hover_text(crate::i18n::I18nOps::get().action.new_file.clone());
                        new_file_resp.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "file+")
                        });
                        if new_file_resp.clicked() {
                            *action = Self::new_file_action(&ws_root);
                        }
                    }
                });
            },
        );

        super::header_filter::ExplorerHeaderFilter::show(ui, workspace, search);
    }

    fn new_file_action(ws_root: &std::path::Path) -> AppAction {
        AppAction::RequestNewFile(ws_root.to_path_buf())
    }

    fn new_directory_action(ws_root: &std::path::Path) -> AppAction {
        AppAction::RequestNewDirectory(ws_root.to_path_buf())
    }

    fn panel_icon_button(
        ui: &egui::Ui,
        icon: crate::Icon,
        size: crate::icon::IconSize,
    ) -> egui::Button<'static> {
        icon.button_on_fill(ui, size, ui.visuals().window_fill())
    }
}
