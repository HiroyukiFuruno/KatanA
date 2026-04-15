use super::side_panels::{
    PreviewSidePanels, PANEL_ANIM_SPEED, PANEL_HEAD_SPACE, PANEL_HOVER_MARGIN, PANEL_WIDTH,
};
use crate::app_state::{AppAction, ViewMode};
use crate::icon::IconSize;
use eframe::egui;
use katana_platform::{PaneOrder, SplitDirection};

use super::tangochou::TangochouWidget;

const SPLIT_SECTION_SPACE: f32 = 8.0;
const ICON_BTN_SIZE: f32 = 32.0;

impl<'a> PreviewSidePanels<'a> {
    pub(super) fn render_tools(&mut self, ui: &mut egui::Ui) {
        let anim = ui.ctx().animate_bool_with_time(
            egui::Id::new("tools_panel_anim"),
            self.app.state.layout.show_tools_panel,
            PANEL_ANIM_SPEED,
        );

        if anim == 0.0 {
            return;
        }

        let mut keep_open = false;
        if self.app.state.layout.show_tools_panel
            && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
            && let Some(btn_rect) = self.tools_btn_rect
            && btn_rect.expand(PANEL_HOVER_MARGIN).contains(pos)
        {
            keep_open = true;
        }

        let panel_resp = egui::SidePanel::right("preview_tools_panel")
            .resizable(false)
            .exact_width(PANEL_WIDTH * anim)
            .show_inside(ui, |ui| {
                ui.set_min_width(PANEL_WIDTH);
                ui.vertical(|ui| {
                    ui.add_space(PANEL_HEAD_SPACE);

                    ui.indent("tools_content", |ui| {
                        /* WHY: Reserve right margin so LabeledToggle doesn't clip. */
                        ui.set_max_width(ui.available_width() - 8.0);
                        let i18n = crate::i18n::I18nOps::get();
                        let view_mode = self.app.state.active_view_mode();
                        let is_split = view_mode == ViewMode::Split;

                        /* ── 分割トグル ── */
                        let mut split_on = is_split;
                        let split_resp = ui.add(
                            crate::widgets::LabeledToggle::new(
                                i18n.view_mode.split.clone(),
                                &mut split_on,
                            )
                            .position(crate::widgets::TogglePosition::Right)
                            .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
                        );
                        if split_resp.changed() {
                            if split_on {
                                self.app.pending_action =
                                    AppAction::SetViewMode(ViewMode::Split);
                            } else {
                                self.app.pending_action =
                                    AppAction::SetViewMode(ViewMode::PreviewOnly);
                            }
                        }

                        ui.add_space(SPLIT_SECTION_SPACE);

                        if is_split {
                            /* ── 分割ON時: スクロール同期 ── */
                            let config_sync = self
                                .app
                                .state
                                .config
                                .settings
                                .settings()
                                .behavior
                                .scroll_sync_enabled;
                            let mut sync = self
                                .app
                                .state
                                .scroll
                                .sync_override
                                .unwrap_or(config_sync);
                            let sync_resp = ui.add(
                                crate::widgets::LabeledToggle::new(
                                    i18n.settings.behavior.scroll_sync.clone(),
                                    &mut sync,
                                )
                                .position(crate::widgets::TogglePosition::Right)
                                .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
                            );
                            if sync_resp.changed() {
                                self.app.pending_action = AppAction::ToggleScrollSync(sync);
                            }

                            ui.add_space(SPLIT_SECTION_SPACE);

                            ui.horizontal(|ui| {
                                let dir = self.app.state.active_split_direction();
                                let order = self.app.state.active_pane_order();

                                /* 左右/上下 切り替えアイコン（現在の方向を表示、クリックでトグル） */
                                let (dir_icon, dir_tip, next_dir) =
                                    if dir == SplitDirection::Horizontal {
                                        (
                                            crate::Icon::SplitHorizontal,
                                            i18n.split_toggle.vertical.clone(),
                                            SplitDirection::Vertical,
                                        )
                                    } else {
                                        (
                                            crate::Icon::SplitVertical,
                                            i18n.split_toggle.horizontal.clone(),
                                            SplitDirection::Horizontal,
                                        )
                                    };
                                let dir_img = dir_icon.ui_image(ui, IconSize::Medium);
                                let btn_dir = egui::Button::image(dir_img)
                                    .min_size(egui::vec2(ICON_BTN_SIZE, ICON_BTN_SIZE));
                                if ui.add(btn_dir).on_hover_text(dir_tip).clicked() {
                                    self.app.pending_action =
                                        AppAction::SetSplitDirection(next_dir);
                                }

                                /* コード/プレビュー 入れ替えアイコン */
                                let swap_icon = if dir == SplitDirection::Horizontal {
                                    crate::Icon::SwapHorizontal
                                } else {
                                    crate::Icon::SwapVertical
                                };
                                let swap_tip = if order == PaneOrder::EditorFirst {
                                    i18n.split_toggle.preview_first.clone()
                                } else {
                                    i18n.split_toggle.editor_first.clone()
                                };
                                let swap_img = swap_icon.ui_image(ui, IconSize::Medium);
                                let btn_swap = egui::Button::image(swap_img)
                                    .min_size(egui::vec2(ICON_BTN_SIZE, ICON_BTN_SIZE));
                                if ui.add(btn_swap).on_hover_text(swap_tip).clicked() {
                                    let new_order = match order {
                                        PaneOrder::EditorFirst => PaneOrder::PreviewFirst,
                                        PaneOrder::PreviewFirst => PaneOrder::EditorFirst,
                                    };
                                    self.app.pending_action = AppAction::SetPaneOrder(new_order);
                                }
                            });
                        } else {
                            /* ── 分割OFF時: Tangochou（コード/プレビュー切替） ── */
                            let i18n2 = crate::i18n::I18nOps::get();
                            if let Some(action) = (TangochouWidget {
                                view_mode: self.app.state.active_view_mode(),
                                label_front: &i18n2.view_mode.code,
                                label_back: &i18n2.view_mode.preview,
                            })
                            .show(ui)
                            {
                                self.app.pending_action = action;
                            }
                        }
                    });
                });
            });

        if self.app.state.layout.show_tools_panel {
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos())
                && panel_resp.response.rect.expand(PANEL_HOVER_MARGIN).contains(pos)
            {
                keep_open = true;
            }

            if !keep_open && ui.input(|i| i.pointer.hover_pos().is_some()) {
                self.app.state.layout.show_tools_panel = false;
            }
        }
    }
}
