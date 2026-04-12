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
                    ui.add_enabled_ui(!is_flat, |ui| {
                        let btn_resp = ui
                            .add(crate::Icon::ExpandAll.button(ui, crate::icon::IconSize::Small))
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
                            .add(crate::Icon::CollapseAll.button(ui, crate::icon::IconSize::Small))
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
                        let more_img = crate::Icon::More.ui_image(ui, crate::icon::IconSize::Small);
                        ui.menu_image_button(more_img, |ui| {
                            if ui
                                .button(crate::i18n::I18nOps::get().menu.open_workspace.clone())
                                .clicked()
                            {
                                *action = crate::shell_ui::ShellUiOps::pick_open_workspace();
                                ui.close();
                            }

                            if workspace.data.is_some() {
                                ui.separator();
                                let mut is_flat_mut = is_flat;
                                if ui
                                    .checkbox(
                                        &mut is_flat_mut,
                                        crate::i18n::I18nOps::get().workspace.flat_view.clone(),
                                    )
                                    .clicked()
                                {
                                    workspace.set_flat_view(ws_root, is_flat_mut);
                                    ui.close();
                                }

                                ui.separator();
                                if ui
                                    .button(
                                        crate::i18n::I18nOps::get().menu.close_workspace.clone(),
                                    )
                                    .clicked()
                                {
                                    *action = AppAction::CloseWorkspace;
                                    ui.close();
                                }
                            }
                        });
                    });

                    if workspace.data.is_some() {
                        let refresh_resp = ui
                            .add(crate::Icon::Refresh.button(ui, crate::icon::IconSize::Small))
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
                                crate::Icon::Filter
                                    .button(ui, crate::icon::IconSize::Small)
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
                    }
                });
            },
        );

        if workspace.data.is_some() && search.filter_enabled {
            let mut is_valid_regex = true;
            if !search.filter_query.is_empty() {
                is_valid_regex = regex::RegexBuilder::new(&search.filter_query)
                    .case_insensitive(true)
                    .build()
                    .is_ok();
            }
            let text_color = if is_valid_regex {
                ui.visuals().text_color()
            } else {
                ui.ctx()
                    .data(|d| {
                        d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                            "katana_theme_colors",
                        ))
                    })
                    .map_or(crate::theme_bridge::WHITE, |tc| {
                        crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.system.error_text)
                    })
            };
            let resp = ui.add(
                egui::TextEdit::singleline(&mut search.filter_query)
                    .text_color(text_color)
                    .hint_text(&crate::i18n::I18nOps::get().workspace.filter_regex_hint)
                    .desired_width(ui.available_width()),
            );
            if ui
                .ctx()
                .memory_mut(|m| m.data.get_temp(egui::Id::new("filter_newly_enabled")))
                .unwrap_or(false)
            {
                resp.request_focus();
                ui.ctx()
                    .memory_mut(|m| m.data.remove::<bool>(egui::Id::new("filter_newly_enabled")));
            }
        }
    }
}
