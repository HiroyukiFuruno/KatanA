use crate::shell::KatanaApp;

/* WHY: Implementation of sidebar popups using foreground areas for zero layout impact. */
impl crate::views::app_frame::types::ExplorerSidebar<'_> {
    /* WHY: Renders the active rail popup with smooth animation and i18n support. */
    pub(crate) fn render_rail_popup(ui: &mut egui::Ui, app: &mut KatanaApp) {
        let Some(active) = app.state.layout.active_rail_popup else {
            return;
        };

        let rail_width =
            crate::shell::SIDEBAR_COLLAPSED_TOGGLE_WIDTH + crate::shell::ACTIVITY_RAIL_PADDING;
        let anchor_y = match active {
            crate::state::layout::RailPopup::Search => {
                crate::shell::ACTIVITY_RAIL_PADDING + crate::shell::RAIL_POPUP_Y_OFFSET_SEARCH
            }
            crate::state::layout::RailPopup::History => {
                ui.max_rect().bottom() - crate::shell::RAIL_POPUP_Y_OFFSET_HISTORY
            }
            crate::state::layout::RailPopup::AddWorkspace => {
                crate::shell::RAIL_POPUP_Y_OFFSET_WORKSPACE
            }
            crate::state::layout::RailPopup::Help => {
                ui.max_rect().bottom() - crate::shell::RAIL_POPUP_Y_OFFSET_HELP
            }
        };

        let area_id = egui::Id::new("rail_popup");
        let is_open = app.state.layout.active_rail_popup.is_some();
        let animation_val = ui.ctx().animate_bool_with_time(
            area_id,
            is_open,
            crate::shell::RAIL_POPUP_ANIMATION_TIME,
        );

        if animation_val <= 0.0 {
            return;
        }

        let area_resp = egui::Area::new(area_id)
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(
                rail_width + crate::shell::RAIL_POPUP_MARGIN,
                anchor_y,
            ))
            .show(ui.ctx(), |ui| {
                ui.set_max_width(crate::shell::RAIL_POPUP_WIDTH);
                ui.set_max_height(crate::shell::RAIL_POPUP_HEIGHT);

                let animation_f32 = animation_val;

                ui.scope(|ui| {
                    let mut window_fill = ui.visuals().window_fill();
                    window_fill = window_fill.gamma_multiply(animation_f32);
                    let mut window_stroke = ui.visuals().window_stroke();
                    window_stroke.color = window_stroke.color.gamma_multiply(animation_f32);

                    let frame = egui::Frame::window(ui.style())
                        .fill(window_fill)
                        .stroke(window_stroke)
                        .shadow(egui::Shadow {
                            color: crate::theme_bridge::ThemeBridgeOps::from_black_alpha(
                                (animation_f32 * (crate::shell::RAIL_POPUP_SHADOW_ALPHA as f32))
                                    as u8,
                            ),
                            ..Default::default()
                        })
                        .inner_margin(egui::Margin::same(crate::shell::RAIL_POPUP_PADDING))
                        .rounding(crate::shell::RAIL_POPUP_ROUNDING);

                    frame.show(ui, |ui| {
                        match active {
                            crate::state::layout::RailPopup::Search => {
                                ui.heading(crate::i18n::I18nOps::get().search.modal_title.clone());
                            }
                            crate::state::layout::RailPopup::History => {
                                super::history_popup::HistoryPopup::render(ui, app);
                            }
                            crate::state::layout::RailPopup::AddWorkspace => {
                                ui.heading(
                                    crate::i18n::I18nOps::get()
                                        .workspace
                                        .open_workspace_button
                                        .clone(),
                                );
                            }
                            crate::state::layout::RailPopup::Help => {
                                ui.heading(crate::i18n::I18nOps::get().menu.help.clone());
                                ui.separator();
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    /* WHY: Display common shortcuts with i18n support */
                                    ui.strong(
                                        crate::i18n::I18nOps::get().help.section_general.clone(),
                                    );
                                    crate::os_command::OsCommandOps::render_shortcut_row(
                                        ui,
                                        &crate::i18n::I18nOps::get().help.shortcut_command_palette,
                                    );
                                    crate::os_command::OsCommandOps::render_shortcut_row(
                                        ui,
                                        &crate::i18n::I18nOps::get().help.shortcut_search,
                                    );
                                    crate::os_command::OsCommandOps::render_shortcut_row(
                                        ui,
                                        &crate::i18n::I18nOps::get().help.shortcut_sidebar,
                                    );
                                    ui.add_space(crate::shell::RAIL_POPUP_SPACING_LARGE);
                                    ui.strong(
                                        crate::i18n::I18nOps::get().help.section_editor.clone(),
                                    );
                                    crate::os_command::OsCommandOps::render_shortcut_row(
                                        ui,
                                        &crate::i18n::I18nOps::get().help.shortcut_save,
                                    );
                                    crate::os_command::OsCommandOps::render_shortcut_row(
                                        ui,
                                        &crate::i18n::I18nOps::get().help.shortcut_refresh,
                                    );
                                    ui.add_space(crate::shell::RAIL_POPUP_SPACING_LARGE);
                                    ui.strong(
                                        crate::i18n::I18nOps::get().help.section_behavior.clone(),
                                    );
                                    crate::os_command::OsCommandOps::render_shortcut_row(
                                        ui,
                                        &crate::i18n::I18nOps::get().help.shortcut_tab_prev,
                                    );
                                    crate::os_command::OsCommandOps::render_shortcut_row(
                                        ui,
                                        &crate::i18n::I18nOps::get().help.shortcut_tab_next,
                                    );
                                    crate::os_command::OsCommandOps::render_shortcut_row(
                                        ui,
                                        &crate::i18n::I18nOps::get().help.shortcut_toggle_split,
                                    );
                                    crate::os_command::OsCommandOps::render_shortcut_row(
                                        ui,
                                        &crate::i18n::I18nOps::get()
                                            .help
                                            .shortcut_toggle_code_preview,
                                    );
                                    crate::os_command::OsCommandOps::render_shortcut_row(
                                        ui,
                                        &crate::i18n::I18nOps::get().help.shortcut_toggle_slideshow,
                                    );
                                    ui.add_space(crate::shell::RAIL_POPUP_SPACING_LARGE);
                                    if ui
                                        .button(
                                            crate::i18n::I18nOps::get().menu.check_updates.clone(),
                                        )
                                        .clicked()
                                    {
                                        app.pending_action =
                                            crate::app_state::AppAction::CheckForUpdates;
                                    }
                                });
                            }
                        }
                    });
                });
            });

        /* WHY: Close popup when clicking outside or clicking the trigger again */
        #[allow(clippy::collapsible_if)]
        if ui.input(|i| i.pointer.any_pressed()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                if !area_resp.response.rect.contains(pos) && pos.x > rail_width {
                    ui.ctx().memory_mut(|m| m.close_popup(area_id));
                }
            }
        }
    }
}
