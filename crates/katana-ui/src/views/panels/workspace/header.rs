use crate::app_state::AppAction;
use eframe::egui;

pub(crate) struct WorkspaceHeader<'a> {
    pub workspace: &'a mut crate::app_state::WorkspaceState,
    pub search: &'a mut crate::app_state::SearchState,
    pub recent_paths: &'a [String],
    pub action: &'a mut AppAction,
}

impl<'a> WorkspaceHeader<'a> {
    pub fn new(
        workspace: &'a mut crate::app_state::WorkspaceState,
        search: &'a mut crate::app_state::SearchState,
        recent_paths: &'a [String],
        action: &'a mut AppAction,
    ) -> Self {
        Self {
            workspace,
            search,
            recent_paths,
            action,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let (workspace, search, _recent_paths, action) =
            (self.workspace, self.search, self.recent_paths, self.action);

        if workspace.data.is_some() {
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

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.scope(|ui| {
                            let more_img =
                                crate::Icon::More.ui_image(ui, crate::icon::IconSize::Small);
                            ui.menu_image_button(more_img, |ui| {
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
                            });
                        });

                        let refresh_resp = ui
                            .add(crate::Icon::Refresh.button(ui, crate::icon::IconSize::Small))
                            .on_hover_text(
                                crate::i18n::I18nOps::get().action.refresh_workspace.clone(),
                            );
                        refresh_resp.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "⟳")
                        });
                        if refresh_resp.clicked() {
                            *action = AppAction::RefreshWorkspace;
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
                            *action = AppAction::ToggleWorkspaceFilter;
                        }
                    });
                },
            );

            if search.filter_enabled {
                let mut is_valid_regex = true;
                if !search.filter_query.is_empty() {
                    is_valid_regex = regex::Regex::new(&search.filter_query).is_ok();
                }
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .content(|ui| {
                        let text_color = if is_valid_regex {
                            ui.visuals().text_color()
                        } else {
                            ui.ctx()
                                .data(|d| {
                                    d.get_temp::<katana_platform::theme::ThemeColors>(
                                        egui::Id::new("katana_theme_colors"),
                                    )
                                })
                                .map_or(crate::theme_bridge::WHITE, |tc| {
                                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(
                                        tc.system.error_text,
                                    )
                                })
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut search.filter_query)
                                .text_color(text_color)
                                .hint_text(&crate::i18n::I18nOps::get().workspace.filter_regex_hint)
                                .desired_width(f32::INFINITY),
                        );
                    })
                    .show(ui);
            }
        }
    }
}
